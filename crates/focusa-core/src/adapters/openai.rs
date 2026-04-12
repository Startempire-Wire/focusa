//! OpenAI-compatible HTTP proxy adapter.
//!
//! Source: 09-proxy-adapter.md, G1-detail-04-proxy-adapter.md
//!
//! Mode B — HTTP proxy between harness and model provider.
//!
//! Flow:
//!   1. Accept OpenAI chat completion request
//!   2. Extract user messages
//!   3. Assemble Focusa-enhanced prompt via Expression Engine
//!   4. Inject as system message (prepend or replace)
//!   5. Forward to upstream provider
//!   6. Return response unchanged
//!   7. Emit turn events to daemon
//!
//! Failure: passthrough raw request (fail-safe).
//! Performance: <20ms overhead target.

use crate::expression::budget::{available_tokens, estimate_tokens};
use crate::expression::engine::AssembledPrompt;
use crate::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Adapter capability declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    pub streaming: bool,
    pub tool_output_capture: bool,
    pub structured_messages: bool,
}

impl Default for AdapterCapabilities {
    fn default() -> Self {
        Self {
            streaming: false,           // MVP: non-streaming only
            tool_output_capture: false, // best-effort
            structured_messages: true,  // OpenAI chat format
        }
    }
}

/// OpenAI chat message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI chat completion request (subset of fields we need).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Pass through any additional fields we don't explicitly model.
    #[serde(flatten)]
    pub extra: Value,
}

/// Result of proxy processing.
pub struct ProxyResult {
    /// The modified request to send upstream.
    pub request: ChatCompletionRequest,
    /// Prompt assembly metadata.
    pub assembly: AssembledPrompt,
    /// The original user input (for event emission).
    pub user_input: String,
}

/// Process an incoming chat completion request through Focusa.
///
/// Extracts user messages, assembles a Focusa-enhanced system prompt,
/// and returns the modified request ready to forward upstream.
///
/// If assembly fails, returns None (caller should passthrough).
pub fn process_request(
    mut request: ChatCompletionRequest,
    state: &FocusaState,
    config: &FocusaConfig,
) -> Option<ProxyResult> {
    // Extract user input from messages.
    let user_input = extract_user_input(&request.messages);
    if user_input.is_empty() {
        return None; // Nothing to enhance — passthrough.
    }

    let assembly = build_operator_first_slice(state, config, &user_input)?;

    // Inject assembled prompt as system message.
    inject_system_message(&mut request.messages, &assembly.content);

    Some(ProxyResult {
        request,
        assembly,
        user_input,
    })
}

#[derive(Debug, Clone, Copy)]
struct OperatorIntent {
    direct_question: bool,
    steering_change: bool,
    focus_relevant: bool,
}

fn classify_operator_intent(text: &str) -> OperatorIntent {
    let t = text.trim().to_lowercase();
    OperatorIntent {
        direct_question: t.ends_with('?')
            || ["what", "why", "how", "when", "where", "who", "can", "could", "should", "is", "are", "do", "does", "did"]
                .iter()
                .any(|p| t.starts_with(&format!("{} ", p))),
        steering_change: [
            "no ", "stop", "instead", "actually", "wait", "wrong", "not what i asked", "new task", "switch to", "different",
        ]
        .iter()
        .any(|needle| t.contains(needle)),
        focus_relevant: [
            "focusa", "focus state", "stack", "constraint", "decision", "thread", "ontology", "mission", "frame", "checkpoint", "context",
        ]
        .iter()
        .any(|needle| t.contains(needle)),
    }
}

fn unique_items(items: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        let trimmed = item.trim();
        if !trimmed.is_empty() && !out.iter().any(|existing: &String| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

pub(crate) fn build_operator_first_slice(
    state: &FocusaState,
    config: &FocusaConfig,
    user_input: &str,
) -> Option<AssembledPrompt> {
    let intent = classify_operator_intent(user_input);
    if !intent.focus_relevant && (intent.direct_question || intent.steering_change) {
        return None;
    }

    let active_frame = state
        .focus_stack
        .active_id
        .and_then(|aid| state.focus_stack.frames.iter().find(|f| f.id == aid))?;
    let fs = &active_frame.focus_state;
    let mission = if !active_frame.goal.trim().is_empty() {
        active_frame.goal.trim().to_string()
    } else if !active_frame.title.trim().is_empty() {
        active_frame.title.trim().to_string()
    } else {
        fs.intent.trim().to_string()
    };
    let active_focus = fs.current_state.trim().to_string();
    let constraints = unique_items(&fs.constraints).into_iter().take(4).collect::<Vec<_>>();
    let decisions = unique_items(&fs.decisions).into_iter().take(4).collect::<Vec<_>>();
    let next_steps = unique_items(&fs.next_steps).into_iter().take(3).collect::<Vec<_>>();
    let blockers = fs
        .failures
        .iter()
        .chain(fs.open_questions.iter())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .fold(Vec::<String>::new(), |mut acc, s| {
            if !acc.iter().any(|existing| existing == &s) {
                acc.push(s);
            }
            acc
        })
        .into_iter()
        .take(3)
        .collect::<Vec<_>>();
    let recent = unique_items(&fs.recent_results).into_iter().take(2).collect::<Vec<_>>();

    let session_id = state.session.as_ref().map(|s| s.session_id);
    let artifact_handles = state
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned)
        .map(|h| format!("{}:{}", format_handle_kind(h.kind), h.label))
        .take(3)
        .collect::<Vec<_>>();

    let lower = user_input.to_lowercase();
    let mut sections: Vec<String> = Vec::new();
    if !mission.is_empty() {
        sections.push(format!("MISSION:\n  - {}", mission));
    }
    if !active_focus.is_empty() && intent.focus_relevant {
        sections.push(format!("ACTIVE_FOCUS:\n  - {}", active_focus));
    }
    if !constraints.is_empty() {
        sections.push(format!(
            "APPLICABLE_CONSTRAINTS:\n{}",
            constraints.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }
    if !decisions.is_empty() && (intent.focus_relevant || lower.contains("why") || lower.contains("decision") || lower.contains("already") || lower.contains("earlier") || lower.contains("reuse")) {
        sections.push(format!(
            "RELEVANT_DECISIONS:\n{}",
            decisions.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }
    if !next_steps.is_empty() && !intent.direct_question {
        sections.push(format!(
            "OPEN_LOOPS:\n{}",
            next_steps.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }
    if !blockers.is_empty() && ["blocked", "failing", "error", "issue", "problem", "why"].iter().any(|needle| lower.contains(needle)) {
        sections.push(format!(
            "BLOCKERS:\n{}",
            blockers.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }
    if !recent.is_empty() && ["latest", "recent", "changed", "what happened", "status"].iter().any(|needle| lower.contains(needle)) {
        sections.push(format!(
            "RECENT_VERIFIED_DELTAS:\n{}",
            recent.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }
    if !artifact_handles.is_empty() && ["file", "artifact", "handle", "output", "evidence"].iter().any(|needle| lower.contains(needle)) {
        sections.push(format!(
            "ARTIFACT_HANDLES:\n{}",
            artifact_handles.iter().map(|x| format!("  - {}", x)).collect::<Vec<_>>().join("\n")
        ));
    }

    if sections.is_empty() {
        return None;
    }

    let available = available_tokens(config.max_prompt_tokens, config.reserve_for_response);
    let max_tokens = available.clamp(120, 600);
    let mut warnings = Vec::new();
    let mut content = format!("[Focusa Minimal Applicable Slice]\n{}", sections.join("\n"));
    if estimate_tokens(&content) > max_tokens {
        let mut trimmed = Vec::new();
        for section in &sections {
            let candidate = format!("[Focusa Minimal Applicable Slice]\n{}", [trimmed.clone(), vec![section.clone()]].concat().join("\n"));
            if estimate_tokens(&candidate) > max_tokens {
                break;
            }
            trimmed.push(section.clone());
        }
        if trimmed.is_empty() {
            return None;
        }
        content = format!("[Focusa Minimal Applicable Slice]\n{}", trimmed.join("\n"));
        warnings.push("Degradation step 1: trimmed minimal applicable slice to fit proxy budget".to_string());
    }
    let token_estimate = estimate_tokens(&content);
    Some(AssembledPrompt {
        content,
        token_estimate,
        handles_used: Vec::new(),
        degraded: !warnings.is_empty(),
        warnings,
    })
}

fn format_handle_kind(kind: HandleKind) -> &'static str {
    match kind {
        HandleKind::Log => "log",
        HandleKind::Diff => "diff",
        HandleKind::Text => "text",
        HandleKind::Json => "json",
        HandleKind::Url => "url",
        HandleKind::FileSnapshot => "file",
        HandleKind::Other => "other",
    }
}

/// Forward a chat completion request to the upstream provider.
pub async fn forward_request(
    client: &Client,
    upstream_url: &str,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> anyhow::Result<Value> {
    let resp = client
        .post(upstream_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Upstream returned HTTP {}: {}", status, body);
    }

    Ok(resp.json().await?)
}

/// Extract user input from chat messages.
///
/// Concatenates all user-role messages (there's usually one).
fn extract_user_input(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .filter(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Inject the assembled Focusa prompt as the system message.
///
/// Strategy: prepend a system message with the assembled prompt.
/// If a system message already exists, replace its content.
fn inject_system_message(messages: &mut Vec<ChatMessage>, content: &str) {
    if let Some(sys) = messages.iter_mut().find(|m| m.role == "system") {
        // Prepend Focusa context to existing system message.
        sys.content = format!("{}\n\n---\n\n{}", content, sys.content);
    } else {
        // Insert system message at the front.
        messages.insert(
            0,
            ChatMessage {
                role: "system".into(),
                content: content.into(),
            },
        );
    }
}

/// Extract assistant response text from a completion response.
pub fn extract_assistant_output(response: &Value) -> Option<String> {
    response
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(String::from)
}

/// Extract token usage from a completion response.
pub fn extract_usage(response: &Value) -> (u32, u32) {
    let usage = response.get("usage");
    let prompt = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let completion = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    (prompt, completion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn test_state() -> FocusaState {
        let frame_id = Uuid::now_v7();
        let mut state = FocusaState::default();
        state.focus_stack.active_id = Some(frame_id);
        state.focus_stack.root_id = Some(frame_id);
        state.focus_stack.frames.push(FrameRecord {
            id: frame_id,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FrameStatus::Active,
            title: "Refactor proxy adapter".into(),
            goal: "Align proxy context injection with operator-first minimal slices".into(),
            beads_issue_id: "focusa-test".into(),
            tags: vec!["project:focusa".into()],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: FocusState {
                intent: "Proxy parity".into(),
                current_state: "Working in proxy adapter".into(),
                decisions: vec!["Use operator-first minimal slices for proxy mode".into()],
                artifacts: vec![],
                constraints: vec!["Do not inject full ontology blobs".into()],
                open_questions: vec![],
                next_steps: vec!["Patch both adapters".into()],
                recent_results: vec!["Trace and behavioral gates green".into()],
                failures: vec!["Proxy still used full assembly".into()],
                notes: vec!["internal note should not leak".into()],
            },
            completed_at: None,
            completion_reason: None,
        });
        state
    }

    #[test]
    fn direct_question_passthroughs_when_focus_irrelevant() {
        let request = ChatCompletionRequest {
            model: "gpt-test".into(),
            messages: vec![ChatMessage { role: "user".into(), content: "What is Rust?".into() }],
            temperature: None,
            max_tokens: None,
            extra: Value::Null,
        };
        let result = process_request(request, &test_state(), &FocusaConfig::default());
        assert!(result.is_none());
    }

    #[test]
    fn focus_relevant_request_gets_minimal_slice_not_full_focus_dump() {
        let request = ChatCompletionRequest {
            model: "gpt-test".into(),
            messages: vec![ChatMessage { role: "user".into(), content: "Why did we already choose this decision for the proxy context?".into() }],
            temperature: None,
            max_tokens: None,
            extra: Value::Null,
        };
        let result = process_request(request, &test_state(), &FocusaConfig::default()).expect("slice injected");
        let system = &result.request.messages[0];
        assert_eq!(system.role, "system");
        assert!(system.content.contains("[Focusa Minimal Applicable Slice]"));
        assert!(system.content.contains("MISSION:"));
        assert!(system.content.contains("APPLICABLE_CONSTRAINTS:"));
        assert!(system.content.contains("RELEVANT_DECISIONS:"));
        assert!(!system.content.contains("FOCUS FRAME:"));
        assert!(!system.content.contains("NOTES"));
    }
}

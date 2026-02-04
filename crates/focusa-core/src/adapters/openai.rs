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

use crate::expression::engine::{assemble, AssembledPrompt};
use crate::memory::procedural;
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
            streaming: false,          // MVP: non-streaming only
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

/// OpenAI chat completion response (passed through unchanged).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    #[serde(flatten)]
    pub raw: Value,
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

    // Get active frame's focus state (or default).
    let focus_state = state
        .focus_stack
        .active_id
        .and_then(|aid| state.focus_stack.frames.iter().find(|f| f.id == aid))
        .map(|f| &f.focus_state)
        .cloned()
        .unwrap_or_default();

    // Select procedural rules.
    let rules = procedural::select_for_prompt(
        &state.memory,
        state.focus_stack.active_id,
        5,
    );
    let rules_owned: Vec<RuleRecord> = rules.into_iter().cloned().collect();

    // Collect artifact handles from active frame.
    let handles: Vec<&HandleRef> = state
        .focus_stack
        .active_id
        .and_then(|aid| state.focus_stack.frames.iter().find(|f| f.id == aid))
        .map(|f| f.handles.iter().collect())
        .unwrap_or_default();
    let handles_owned: Vec<HandleRef> = handles.into_iter().cloned().collect();

    // Assemble prompt.
    let assembly = assemble(
        &focus_state,
        None, // No ASCC checkpoint in MVP proxy
        &rules_owned,
        &handles_owned,
        &user_input,
        config,
    );

    // Inject assembled prompt as system message.
    inject_system_message(&mut request.messages, &assembly.content);

    Some(ProxyResult {
        request,
        assembly,
        user_input,
    })
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
        messages.insert(0, ChatMessage {
            role: "system".into(),
            content: content.into(),
        });
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

//! Anthropic API proxy adapter.
//!
//! Mode B — HTTP proxy for Claude models via Anthropic's /v1/messages API.
//!
//! Flow:
//!   1. Accept Anthropic messages request
//!   2. Extract user messages
//!   3. Assemble Focusa-enhanced prompt via Expression Engine
//!   4. Inject into system prompt field
//!   5. Forward to upstream Anthropic
//!   6. Return response unchanged
//!   7. Emit turn events
//!
//! Failure: passthrough raw request (fail-safe).

use crate::expression::engine::{assemble, AssembledPrompt};
use crate::memory::procedural;
use crate::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Anthropic message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Value, // Can be string or array of content blocks
}

/// Anthropic messages request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Pass through any additional fields.
    #[serde(flatten)]
    pub extra: Value,
}

/// Result of proxy processing.
pub struct ProxyResult {
    pub request: MessagesRequest,
    pub assembly: AssembledPrompt,
    pub user_input: String,
}

/// Process an incoming Anthropic messages request through Focusa.
pub fn process_request(
    mut request: MessagesRequest,
    state: &FocusaState,
    config: &FocusaConfig,
) -> Option<ProxyResult> {
    let user_input = extract_user_input(&request.messages);
    if user_input.is_empty() {
        return None;
    }

    // Get active frame's focus state.
    let focus_state = state
        .focus_stack
        .active_id
        .and_then(|aid| state.focus_stack.frames.iter().find(|f| f.id == aid))
        .map(|f| &f.focus_state)
        .cloned()
        .unwrap_or_default();

    // Select procedural rules.
    let rules = procedural::select_for_prompt(&state.memory, state.focus_stack.active_id, 5);
    let rules_owned: Vec<RuleRecord> = rules.into_iter().cloned().collect();

    // Collect artifact handles.
    let session_id = state.session.as_ref().map(|s| s.session_id);
    let handles_owned: Vec<HandleRef> = state
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned)
        .cloned()
        .collect();

    // Assemble prompt.
    let assembly = assemble(
        &focus_state,
        None,
        &rules_owned,
        &handles_owned,
        &user_input,
        config,
    );

    // Inject into system prompt.
    inject_system_prompt(&mut request, &assembly.content);

    Some(ProxyResult {
        request,
        assembly,
        user_input,
    })
}

/// Forward request to Anthropic API.
pub async fn forward_request(
    client: &Client,
    upstream_url: &str,
    api_key: &str,
    request: &MessagesRequest,
) -> anyhow::Result<Value> {
    let resp = client
        .post(upstream_url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Anthropic returned HTTP {}: {}", status, body);
    }

    Ok(resp.json().await?)
}

/// Extract user input from messages (public for proxy use).
pub fn extract_user_input(messages: &[AnthropicMessage]) -> String {
    messages
        .iter()
        .filter(|m| m.role == "user")
        .filter_map(|m| extract_text_content(&m.content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract text from content (handles string or content block array).
fn extract_text_content(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let texts: Vec<&str> = blocks
                .iter()
                .filter_map(|b| {
                    if b.get("type")?.as_str()? == "text" {
                        b.get("text")?.as_str()
                    } else {
                        None
                    }
                })
                .collect();
            if texts.is_empty() {
                None
            } else {
                Some(texts.join("\n"))
            }
        }
        _ => None,
    }
}

/// Inject Focusa context into the system prompt.
fn inject_system_prompt(request: &mut MessagesRequest, content: &str) {
    if let Some(ref mut sys) = request.system {
        // Prepend Focusa context.
        *sys = format!("{}\n\n---\n\n{}", content, sys);
    } else {
        // Set as new system prompt.
        request.system = Some(content.to_string());
    }
}

/// Extract assistant response text from Anthropic response.
pub fn extract_assistant_output(response: &Value) -> Option<String> {
    let content = response.get("content")?.as_array()?;
    let texts: Vec<&str> = content
        .iter()
        .filter_map(|b| {
            if b.get("type")?.as_str()? == "text" {
                b.get("text")?.as_str()
            } else {
                None
            }
        })
        .collect();
    if texts.is_empty() {
        None
    } else {
        Some(texts.join(""))
    }
}

/// Extract token usage from Anthropic response.
pub fn extract_usage(response: &Value) -> (u32, u32) {
    let usage = response.get("usage");
    let input = usage
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output = usage
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    (input, output)
}

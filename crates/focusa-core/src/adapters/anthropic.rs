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

use crate::expression::engine::{AssembledPrompt, assemble};
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_system"
    )]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Pass through any additional fields.
    #[serde(flatten)]
    pub extra: Value,
}

/// Deserialize `system` from either a string or an array of content blocks.
/// Anthropic API allows both formats; upstream Kimi only accepts strings.
fn deserialize_system<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Array(arr)) => {
            // Extract text from content blocks: [{"type":"text","text":"..."},...]
            let mut parts = Vec::new();
            for item in &arr {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    parts.push(text);
                } else if let Some(s) = item.as_str() {
                    parts.push(s);
                }
            }
            if parts.is_empty() {
                Ok(None)
            } else {
                Ok(Some(parts.join("\n\n")))
            }
        }
        Some(other) => Ok(Some(other.to_string())),
    }
}

/// Result of proxy processing.
pub struct ProxyResult {
    pub request: MessagesRequest,
    pub assembly: AssembledPrompt,
    pub user_input: String,
}

/// Auth header for Anthropic-compatible upstreams.
#[derive(Debug, Clone)]
pub enum AnthropicAuth {
    ApiKey(String),
    Bearer(String),
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
    let project_id = state
        .focus_stack
        .active_id
        .and_then(|fid| state.focus_stack.frames.iter().find(|f| f.id == fid))
        .and_then(|frame| {
            frame
                .tags
                .iter()
                .find(|t| t.starts_with("project:"))
                .map(|t| t.trim_start_matches("project:").to_string())
        });

    let rules = procedural::select_for_prompt(
        &state.memory,
        state.focus_stack.active_id,
        project_id.as_deref(),
        5,
    );
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

    // Build ASCC sections from FocusState (G1-07 §Prompt Serialization).
    let ascc = crate::types::AsccSections::from(&focus_state);
    let ascc_ref = if ascc.is_empty() { None } else { Some(&ascc) };

    // Assemble prompt.
    let assembly = assemble(
        &focus_state,
        ascc_ref,
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
    auth: &AnthropicAuth,
    request: &MessagesRequest,
) -> anyhow::Result<Value> {
    let mut req = client
        .post(upstream_url)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json");

    match auth {
        AnthropicAuth::ApiKey(key) => {
            req = req.header("x-api-key", key);
        }
        AnthropicAuth::Bearer(token) => {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
    }

    let resp = req.json(request).send().await?;

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

/// Best-effort compatibility sanitizer for Anthropic-shaped providers.
///
/// Important: keep `tools` / `tool_choice` so upstream can still perform
/// actual tool-calling; only strip known incompatible extras and normalize
/// legacy blocks in message history.
pub fn sanitize_for_compat(request: &mut MessagesRequest) {
    if let Some(obj) = request.extra.as_object_mut() {
        for key in ["thinking", "mcp_servers", "metadata"] {
            obj.remove(key);
        }
    }

    for msg in &mut request.messages {
        msg.content = coerce_content_to_text(&msg.content);
    }
}

fn coerce_content_to_text(content: &Value) -> Value {
    match content {
        Value::String(s) => Value::String(strip_tool_markup_text(s)),
        Value::Array(blocks) => {
            let mut parts: Vec<String> = Vec::new();
            for b in blocks {
                let kind = b.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match kind {
                    "text" => {
                        if let Some(t) = b.get("text").and_then(|v| v.as_str()) {
                            parts.push(t.to_string());
                        }
                    }
                    "tool_result" => {
                        let inner = b.get("content").unwrap_or(&Value::Null);
                        let rendered = match inner {
                            Value::String(s) => s.clone(),
                            Value::Array(arr) => arr
                                .iter()
                                .filter_map(|it| it.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            other => other.to_string(),
                        };
                        if !rendered.trim().is_empty() {
                            parts.push(rendered);
                        }
                    }
                    "tool_use" => {
                        // Drop raw tool_use blocks from historical context to avoid
                        // leaking tool markup into assistant natural-language output.
                    }
                    "thinking" => {
                        // Drop thinking blocks for compatibility.
                    }
                    _ => {
                        if let Some(t) = b.get("text").and_then(|v| v.as_str()) {
                            parts.push(t.to_string());
                        }
                    }
                }
            }
            Value::String(parts.join("\n"))
        }
        other => Value::String(other.to_string()),
    }
}

pub fn strip_tool_markup_text(input: &str) -> String {
    let mut s = input.to_string();

    // Remove XML-style tool wrappers.
    s = remove_between_all(&s, "<tool>", "</tool>");
    s = remove_between_all(&s, "<parameter>", "</parameter>");

    // Remove bracket-style tool wrappers that sometimes leak from compatibility shims.
    s = remove_bracket_tool_inline(&s, "[tool_use:");
    s = remove_bracket_tool_inline(&s, "[tool_result:");

    // Safety net for orphan tags.
    s = s
        .replace("<tool>", "")
        .replace("</tool>", "")
        .replace("<parameter>", "")
        .replace("</parameter>", "");

    // Normalize trailing whitespace per line.
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

fn remove_between_all(input: &str, start: &str, end: &str) -> String {
    let mut out = input.to_string();
    loop {
        let Some(sidx) = out.find(start) else { break };
        let Some(eidx_rel) = out[sidx + start.len()..].find(end) else {
            out.replace_range(sidx.., "");
            break;
        };
        let eidx = sidx + start.len() + eidx_rel + end.len();
        out.replace_range(sidx..eidx, "");
    }
    out
}

fn remove_bracket_tool_inline(input: &str, marker: &str) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    for mut line in input.lines().map(|l| l.to_string()) {
        while let Some(idx) = line.find(marker) {
            let end_idx = if let Some(close_obj_rel) = line[idx..].find('}') {
                idx + close_obj_rel + 1
            } else {
                line.len()
            };
            line.replace_range(idx..end_idx, "");
        }
        out_lines.push(line);
    }
    out_lines.join("\n")
}

//! ACP Proxy Adapter (Mode A: Observation + Mode B: Active Cognitive Proxy)
//!
//! Source: docs/33-acp-proxy-spec.md
//!
//! Mode A: Passive observation — subprocess wrapping, telemetry only.
//! Mode B: Active cognitive proxy — full Focusa cognition applied to ACP traffic.
//!
//! Protocol: ACP (Agent Client Protocol) — JSON-RPC 2.0 over stdio/HTTP.
//! Integration: Zed, Cursor, any ACP-compliant editor.

use crate::types::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ACP integration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpMode {
    /// Passive observation — telemetry only.
    Observation,
    /// Active cognitive proxy — full Focus Gate, Prompt Assembly, CLT.
    Proxy,
    /// Disabled.
    Off,
}

/// ACP session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpSession {
    pub session_id: String,
    pub mode: AcpMode,
    pub agent_command: String,
    pub started_at: chrono::DateTime<Utc>,
    pub message_count: u64,
    pub bytes_forwarded: u64,
}

/// ACP JSON-RPC message (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

/// ACP telemetry event (generated for every observed message).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpTelemetryEvent {
    pub event_id: Uuid,
    pub session_id: String,
    pub direction: AcpDirection,
    pub method: Option<String>,
    pub timestamp: chrono::DateTime<Utc>,
    pub latency_ms: Option<u64>,
    pub token_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpDirection {
    ClientToAgent,
    AgentToClient,
}

/// Parse a JSON-RPC message from raw bytes.
pub fn parse_message(data: &[u8]) -> Result<AcpMessage, String> {
    serde_json::from_slice(data).map_err(|e| format!("ACP parse error: {}", e))
}

/// Create a telemetry event from an observed ACP message.
pub fn observe_message(
    session_id: &str,
    msg: &AcpMessage,
    direction: AcpDirection,
) -> AcpTelemetryEvent {
    AcpTelemetryEvent {
        event_id: Uuid::now_v7(),
        session_id: session_id.into(),
        direction,
        method: msg.method.clone(),
        timestamp: Utc::now(),
        latency_ms: None,
        token_count: None,
    }
}

/// Check if a message is a prompt/completion request.
pub fn is_completion_request(msg: &AcpMessage) -> bool {
    msg.method.as_deref() == Some("completions/create")
        || msg.method.as_deref() == Some("chat/completions")
}

/// Check if a message is a tool call.
pub fn is_tool_call(msg: &AcpMessage) -> bool {
    msg.method.as_deref() == Some("tools/call") || msg.method.as_deref() == Some("tool/execute")
}

/// Extract token stats from a completion response (if available).
pub fn extract_token_stats(msg: &AcpMessage) -> Option<(u64, u64)> {
    let result = msg.result.as_ref()?;
    let usage = result.get("usage")?;
    let prompt = usage.get("prompt_tokens")?.as_u64()?;
    let completion = usage.get("completion_tokens")?.as_u64()?;
    Some((prompt, completion))
}

/// Mode A: Build the observer wrapper command.
///
/// Returns the command line to launch the agent as a subprocess with
/// Focusa observing stdin/stdout traffic.
pub fn build_observer_command(agent_command: &str) -> Vec<String> {
    // The wrapper launches the agent and pipes through Focusa.
    vec![
        "focusa-daemon".into(),
        "--acp-observe".into(),
        "--".into(),
        agent_command.into(),
    ]
}

/// Mode B: Apply Focus Gate filtering to an ACP completion request.
///
/// Injects ASCC context, applies gate decisions, tracks CLT node.
pub fn apply_cognition(
    msg: &mut AcpMessage,
    focus_state: &FocusState,
    _gate_state: &FocusGateState,
) {
    if !is_completion_request(msg) {
        return;
    }

    // Inject ASCC context as a system message prefix.
    let ascc_summary = crate::expression::serializer::to_string_compact(focus_state);
    if !ascc_summary.is_empty()
        && let Some(ref mut params) = msg.params
        && let Some(messages) = params.get_mut("messages")
        && let Some(arr) = messages.as_array_mut()
    {
        let system_msg = serde_json::json!({
            "role": "system",
            "content": format!("[Focusa Context]\n{}", ascc_summary),
        });
        arr.insert(0, system_msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jsonrpc() {
        let data = br#"{"jsonrpc":"2.0","method":"completions/create","id":1,"params":{}}"#;
        let msg = parse_message(data).unwrap();
        assert_eq!(msg.method.as_deref(), Some("completions/create"));
        assert!(is_completion_request(&msg));
    }

    #[test]
    fn test_observe_message() {
        let msg = AcpMessage {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: Some("tools/call".into()),
            params: None,
            result: None,
            error: None,
        };
        let event = observe_message("session-1", &msg, AcpDirection::ClientToAgent);
        assert_eq!(event.session_id, "session-1");
        assert!(is_tool_call(&msg));
    }

    #[test]
    fn test_extract_tokens() {
        let msg = AcpMessage {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "usage": { "prompt_tokens": 500, "completion_tokens": 200 }
            })),
            error: None,
        };
        let (p, c) = extract_token_stats(&msg).unwrap();
        assert_eq!(p, 500);
        assert_eq!(c, 200);
    }
}

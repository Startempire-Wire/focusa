//! `GET /v1/agent/prompt` — surfaces a structured reminder to agents (e.g. Pi
//! running the focusa-pi-extension) telling them the canonical interaction
//! layer is the focusa_* tool suite, not direct curl/fetch.
//!
//! Two surfaces are emitted:
//!   1. JSON body (when called explicitly): full structured prompt with
//!      tool_count, tool_families, reason, next_tools, operator_reminder,
//!      registry_url, and the agent's perceived "client".
//!   2. Response header `X-Focusa-Agent-Prompt: focusa_*` (on EVERY response
//!      to a detected Pi client): a low-overhead grep hint that surfaces the
//!      reminder passively without an extra request.
//!
//! Detection (any one of):
//!   - `X-Focusa-Client: pi` header (set by the focusa-pi-extension client)
//!   - `X-Extension-Token: focusa-pi*` header (set by the focusa-pi-extension)
//!   - `User-Agent` contains `focusa-pi`
//!
//! When no detection, the endpoint returns a minimal `{ is_agent: false }`
//! and the response header is omitted.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

/// Header names — kept public so tests + middleware can reference them.
pub const HEADER_CLIENT: &str = "x-focusa-client";
pub const HEADER_EXTENSION_TOKEN: &str = "x-extension-token";
pub const HEADER_AGENT_PROMPT: &str = "x-focusa-agent-prompt";
pub const HEADER_USER_AGENT: &str = "user-agent";
pub const DETECTED_CLIENT: &str = "pi";
pub const PROMPT_HINT: &str = "focusa_*";

/// Tool families registered in the focusa_* tool surface (Spec 90).
/// Keep in sync with apps/pi-extension/src/tool-contracts.ts FocusaToolFamily.
const TOOL_FAMILIES: &[&str] = &[
    "focus_state",
    "workpoint",
    "work_loop",
    "metacognition",
    "tree_lineage",
    "diagnostics_hygiene",
    "trajectory",
    "project_identity",
    "traversal",
    "session_transfer",
];

const TOOL_CONTRACTS_JSON: &str =
    include_str!("../../../../docs/current/focusa-tool-contracts.json");

fn tool_contract_count() -> usize {
    serde_json::from_str::<Value>(TOOL_CONTRACTS_JSON)
        .ok()
        .and_then(|registry| {
            registry
                .get("contracts")
                .and_then(Value::as_array)
                .map(|arr| arr.len())
        })
        .unwrap_or(0)
}

/// True iff the request looks like it's from a Pi agent client.
pub fn detect_pi_agent(headers: &HeaderMap) -> bool {
    if let Some(v) = headers.get(HEADER_CLIENT)
        && v.to_str()
            .map(|s| s.eq_ignore_ascii_case(DETECTED_CLIENT))
            .unwrap_or(false)
        {
            return true;
        }
    if let Some(v) = headers.get(HEADER_EXTENSION_TOKEN)
        && v.to_str()
            .map(|s| s.to_ascii_lowercase().starts_with("focusa-pi"))
            .unwrap_or(false)
        {
            return true;
        }
    if let Some(v) = headers.get(HEADER_USER_AGENT)
        && v.to_str()
            .map(|s| s.to_ascii_lowercase().contains("focusa-pi"))
            .unwrap_or(false)
        {
            return true;
        }
    false
}

/// Build the structured prompt body for a detected Pi agent.
fn build_prompt() -> Value {
    json!({
        "is_agent": true,
        "client": DETECTED_CLIENT,
        "preferred_layer": "focusa_* tools",
        "tool_registry_path": "/v1/ontology/tool-contracts",
        "tool_count": tool_contract_count(),
        "tool_families": TOOL_FAMILIES,
        "reason": "Direct curl/fetch bypasses the tool_result_v1 envelope, evidence linking, failure_class recovery, next_tools choreography, and metacog loop. The focusa_* tool layer is canonical for daemon interactions in Pi (and any Focusa-aware editor).",
        "next_tools": [
            "focusa_agent_prompt",
            "focusa_tool_doctor",
            "focusa_trajectory_view",
            "focusa_workpoint_resume",
            "focusa_workpoint_checkpoint",
            "focusa_evidence_capture",
            "focusa_metacog_capture",
            "focusa_metacog_retrieve"
        ],
        "operator_reminder": "Decide MVP UI scope (menubar / PWA / TUI in-MVP or v0.2) so the next workpoint can lock it in.",
        "active_trajectory_hint": "HLT = Build Focusa and go to market soon with an MVP; menubar is an MLG subordinate, not the HLT.",
        "rule": "every daemon interaction -> focusa_* tool. UIAI pretest is a separate verification surface and remains raw."
    })
}

/// Minimal response when the client is not detected as a Pi agent.
fn build_non_agent_prompt() -> Value {
    json!({
        "is_agent": false,
        "preferred_layer": "HTTP (any client)",
        "hint": format!("set {}=pi or {}=focusa-pi... to receive the agent-layer prompt", HEADER_CLIENT, HEADER_EXTENSION_TOKEN)
    })
}

/// `GET /v1/agent/prompt` — returns the agent reminder when the request
/// looks like a Pi agent; minimal response otherwise.
pub async fn agent_prompt(State(_state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let is_pi = detect_pi_agent(&headers);
    let mut response = if is_pi {
        (StatusCode::OK, Json(build_prompt())).into_response()
    } else {
        (StatusCode::OK, Json(build_non_agent_prompt())).into_response()
    };
    if is_pi
        && let Ok(v) = HeaderValue::from_str(PROMPT_HINT) {
            response
                .headers_mut()
                .insert(HeaderName::from_static(HEADER_AGENT_PROMPT), v);
        }
    response
}

/// Response-header + body middleware: when the request is from a Pi agent,
/// add `X-Focusa-Agent-Prompt: focusa_*` to the response AND inject reminder hints
/// into the body when possible, so the instruction is visible outside JSON payloads.
///
/// For JSON responses, inject a top-level `_agent_prompt` object.
/// For plain-text responses, append a short reminder trailer.
/// Streaming bodies or large payloads are passed through unmodified to avoid latency.
pub const MAX_INJECT_BYTES: usize = 256 * 1024;

pub async fn agent_prompt_response_header_mw(
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let is_pi = detect_pi_agent(&headers);
    if !is_pi {
        return next.run(request).await;
    }
    let is_prompt_route = request.uri().path() == "/v1/agent/prompt";
    let mut response = next.run(request).await;
    if let Ok(v) = HeaderValue::from_str(PROMPT_HINT) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(HEADER_AGENT_PROMPT), v);
    }
    if !is_prompt_route {
        inject_agent_prompt_into_body(&mut response).await;
    }
    response
}

/// Try to inject reminder content into response bodies so Pi clients always have
/// a visible hint. JSON payloads get `_agent_prompt`; plain-text responses get
/// a short trailer to avoid silent drift when JSON parsing is unavailable.
async fn inject_agent_prompt_into_body(response: &mut Response) {
    use axum::body::Body;

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_json = content_type.contains("application/json");
    let is_plain = content_type.starts_with("text/") || content_type.is_empty();

    let body = std::mem::replace(response.body_mut(), Body::empty());
    let bytes = match axum::body::to_bytes(body, MAX_INJECT_BYTES + 1).await {
        Ok(b) => b,
        Err(_) => return,
    };
    if bytes.len() > MAX_INJECT_BYTES {
        return;
    }

    if is_json {
        let mut value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let serde_json::Value::Object(ref mut map) = value {
            if !map.contains_key("_agent_prompt") {
                map.insert("_agent_prompt".to_string(), build_inject_prompt());
            }
        } else {
            return;
        }
        let new_bytes = match serde_json::to_vec(&value) {
            Ok(b) => b,
            Err(_) => return,
        };
        let new_len = new_bytes.len();
        *response.body_mut() = Body::from(new_bytes);
        if let Ok(len) = HeaderValue::from_str(&new_len.to_string()) {
            response
                .headers_mut()
                .insert(axum::http::header::CONTENT_LENGTH, len);
        }
        return;
    }

    if !is_plain {
        *response.body_mut() = Body::from(bytes);
        return;
    }

    let text = match std::str::from_utf8(&bytes) {
        Ok(v) => v,
        Err(_) => {
            *response.body_mut() = Body::from(bytes);
            return;
        }
    };
    let marker = "\n\nfocusa reminder: use focusa_* tool calls over raw curl/fetch\n";
    let with_hint = format!("{}{}", text, marker);
    let new_len = with_hint.len();
    *response.body_mut() = Body::from(with_hint);
    if let Ok(len) = HeaderValue::from_str(&new_len.to_string()) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_LENGTH, len);
    }
}

/// The injected `_agent_prompt` payload. Compact but impossible to miss:
/// starts with a literal alarm marker, names the rule in plain English,
/// and lists the next canonical tools.
fn build_inject_prompt() -> Value {
    json!({
        "marker": "🔔🔔🔔 AGENT PROMPT — READ THIS 🔔🔔🔔",
        "rule": "You are an agent running in Pi (or another Focusa-aware editor). PREFER the focusa_* tool layer over direct curl/fetch for every daemon interaction.",
        "why": "Direct curl/fetch bypasses the tool_result_v1 envelope, evidence linking, failure_class recovery, next_tools choreography, and the metacog loop. The focusa_* tool layer is the canonical interaction surface for Focusa-aware agents.",
        "preferred_layer": "focusa_* tools",
        "next_tools": [
            "focusa_agent_prompt",
            "focusa_tool_doctor",
            "focusa_trajectory_view",
            "focusa_workpoint_resume",
            "focusa_workpoint_checkpoint",
            "focusa_evidence_capture",
            "focusa_metacog_capture",
            "focusa_metacog_retrieve"
        ],
        "tool_registry": "/v1/ontology/tool-contracts",
        "full_prompt_endpoint": "/v1/agent/prompt",
        "do_not_drift": [
            "do not call curl/fetch directly when a focusa_* tool exists",
            "do not skip workpoint_checkpoint before long-running slices",
            "do not skip evidence_capture when proof is available",
            "do not skip metacog_capture when a reusable lesson is learned"
        ],
        "active_trajectory_hint": "HLT=Build Focusa and go to market soon with an MVP. The Mac menubar is an MLG subordinate, not the HLT."
    })
}

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/agent/prompt", axum::routing::get(agent_prompt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    fn h() -> HeaderMap {
        HeaderMap::new()
    }

    #[test]
    fn no_headers_is_not_pi() {
        assert!(!detect_pi_agent(&h()));
    }

    #[test]
    fn x_focusa_client_pi_is_pi() {
        let mut hm = h();
        hm.insert(HEADER_CLIENT, HeaderValue::from_static("pi"));
        assert!(detect_pi_agent(&hm));
        hm.insert(HEADER_CLIENT, HeaderValue::from_static("PI"));
        assert!(detect_pi_agent(&hm));
    }

    #[test]
    fn x_focusa_client_random_is_not_pi() {
        let mut hm = h();
        hm.insert(HEADER_CLIENT, HeaderValue::from_static("curl"));
        assert!(!detect_pi_agent(&hm));
    }

    #[test]
    fn x_extension_token_focusa_pi_is_pi() {
        let mut hm = h();
        hm.insert(
            HEADER_EXTENSION_TOKEN,
            HeaderValue::from_static("focusa-pi-abc123"),
        );
        assert!(detect_pi_agent(&hm));
    }

    #[test]
    fn user_agent_focusa_pi_is_pi() {
        let mut hm = h();
        hm.insert(
            HEADER_USER_AGENT,
            HeaderValue::from_static("focusa-pi/0.9.14-dev"),
        );
        assert!(detect_pi_agent(&hm));
    }

    #[test]
    fn prompt_body_lists_tool_families() {
        let body = build_prompt();
        let fams = body
            .get("tool_families")
            .and_then(|v| v.as_array())
            .unwrap();
        assert!(fams.iter().any(|v| v.as_str() == Some("workpoint")));
        assert!(fams.iter().any(|v| v.as_str() == Some("trajectory")));
    }
}

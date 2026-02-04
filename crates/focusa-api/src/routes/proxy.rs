//! Proxy routes — OpenAI-compatible HTTP proxy.
//!
//! POST /proxy/v1/chat/completions — proxied chat completion
//!
//! Configure harness to use: http://127.0.0.1:8787/proxy/v1/chat/completions
//! Set FOCUSA_UPSTREAM_URL and FOCUSA_API_KEY env vars.

use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::post};
use focusa_core::adapters::openai::{
    self, ChatCompletionRequest, extract_assistant_output, extract_usage,
};
use focusa_core::adapters::passthrough;
use focusa_core::types::Action;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

/// Default upstream URL (OpenAI).
const DEFAULT_UPSTREAM: &str = "https://api.openai.com/v1/chat/completions";

/// Lazy-initialized HTTP client for upstream requests.
static UPSTREAM_CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

fn get_client() -> &'static Client {
    UPSTREAM_CLIENT.get_or_init(Client::new)
}

/// Get upstream URL from env or default.
fn upstream_url() -> String {
    std::env::var("FOCUSA_UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM.into())
}

/// Get API key from env (or from request Authorization header).
fn api_key(headers: &HeaderMap) -> Option<String> {
    // Check env first, then request header.
    if let Ok(key) = std::env::var("FOCUSA_API_KEY") {
        return Some(key);
    }
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
}

/// POST /proxy/v1/chat/completions
///
/// 1. Read Focusa state
/// 2. Assemble enhanced prompt via Expression Engine
/// 3. Forward to upstream
/// 4. Return response unchanged
/// 5. Emit turn events
///
/// On failure: passthrough raw request (fail-safe).
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key = api_key(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No API key — set FOCUSA_API_KEY or pass Authorization header"})),
        )
    })?;

    let url = upstream_url();
    let client = get_client();

    // Try to enhance the request with Focusa state.
    let focusa_state = state.focusa.read().await;
    let result = openai::process_request(request.clone(), &focusa_state, &state.config);
    drop(focusa_state); // Release read lock before HTTP call.

    let (response, _user_input, _assembly_info) = match result {
        Some(proxy_result) => {
            // Enhanced request — forward with Focusa context.
            let user_input = proxy_result.user_input.clone();
            let degraded = proxy_result.assembly.degraded;
            let token_est = proxy_result.assembly.token_estimate;

            match openai::forward_request(client, &url, &key, &proxy_result.request).await {
                Ok(resp) => (resp, user_input, Some((degraded, token_est))),
                Err(e) => {
                    tracing::error!("Upstream request failed: {}", e);
                    return Err((
                        StatusCode::BAD_GATEWAY,
                        Json(json!({"error": format!("Upstream error: {}", e)})),
                    ));
                }
            }
        }
        None => {
            // Passthrough — no enhancement possible.
            match passthrough::passthrough(client, &url, &key, &request).await {
                Ok(resp) => {
                    let user_input = request
                        .messages
                        .iter()
                        .filter(|m| m.role == "user")
                        .map(|m| m.content.clone())
                        .collect::<Vec<_>>()
                        .join("\n");
                    (resp, user_input, None)
                }
                Err(e) => {
                    return Err((
                        StatusCode::BAD_GATEWAY,
                        Json(json!({"error": format!("Upstream error: {}", e)})),
                    ));
                }
            }
        }
    };

    // Emit turn events to daemon (fire-and-forget).
    let assistant_output = extract_assistant_output(&response);
    let (prompt_tokens, completion_tokens) = extract_usage(&response);

    // Notify the intuition engine about this turn.
    if assistant_output.is_some() {
        let frame_id = {
            let focusa = state.focusa.read().await;
            focusa.focus_stack.active_id
        };

        // Update frame stats.
        if let Some(fid) = frame_id {
            let delta = focusa_core::types::FocusStateDelta {
                current_state: Some(format!(
                    "Last turn: {} prompt tokens, {} completion tokens",
                    prompt_tokens, completion_tokens,
                )),
                ..Default::default()
            };
            let _ = state.command_tx.send(Action::UpdateCheckpointDelta {
                frame_id: fid,
                turn_id: uuid::Uuid::now_v7().to_string(),
                delta,
            }).await;
        }
    }

    Ok(Json(response))
}

/// POST /proxy/acp — ACP JSON-RPC proxy endpoint.
///
/// ACP clients can point to this endpoint for Mode B (Active Cognitive Proxy).
/// Focusa applies Focus Gate + Prompt Assembly + CLT tracking.
async fn acp_proxy(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use focusa_core::adapters::acp;

    // Parse ACP message.
    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    let mut msg = acp::parse_message(&bytes).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": e })))
    })?;

    // Record telemetry (logged; persistence via telemetry subsystem).
    let s = state.focusa.read().await;
    let session_id = s.session.as_ref().map(|s| s.session_id.to_string()).unwrap_or_default();
    let telemetry = acp::observe_message(&session_id, &msg, acp::AcpDirection::ClientToAgent);
    tracing::debug!(method = ?telemetry.method, direction = ?telemetry.direction, "ACP message observed");

    // Apply cognition (Mode B) — use active frame, not first frame.
    if let Some(active_id) = s.focus_stack.active_id
        && let Some(frame_record) = s.focus_stack.frames.iter().find(|f| f.id == active_id)
    {
        acp::apply_cognition(&mut msg, &frame_record.focus_state, &s.focus_gate);
    }
    drop(s);

    // Forward to upstream ACP server.
    let upstream = std::env::var("FOCUSA_ACP_UPSTREAM").unwrap_or_else(|_| "http://127.0.0.1:4000".into());
    let client = Client::new();
    let resp = client.post(&upstream)
        .json(&msg)
        .send()
        .await
        .map_err(|e| {
            (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("ACP upstream error: {}", e) })))
        })?;

    let response: Value = resp.json().await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, Json(json!({ "error": format!("ACP response parse error: {}", e) })))
    })?;

    Ok(Json(response))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/proxy/v1/chat/completions", post(chat_completions))
        .route("/proxy/acp", post(acp_proxy))
}

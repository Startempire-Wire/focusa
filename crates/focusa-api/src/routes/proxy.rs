//! Proxy routes — HTTP proxy for AI providers with full turn tracking.
//!
//! POST /proxy/v1/chat/completions — OpenAI-compatible
//! POST /proxy/v1/messages — Anthropic (Claude)
//!
//! Per spec G1-detail-04-proxy-adapter.md:
//! 1. TurnStart — record turn beginning
//! 2. PromptAssemble — enhance with Focusa context
//! 3. Forward to upstream
//! 4. TurnComplete — record result (success or failure)
//! 5. Emit signals to Focus Gate

use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::post};
use focusa_core::adapters::anthropic::{self, MessagesRequest};
use focusa_core::adapters::openai::{
    self, ChatCompletionRequest, extract_assistant_output, extract_usage,
};
use focusa_core::adapters::passthrough;
use focusa_core::types::*;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use chrono::Utc;

const DEFAULT_UPSTREAM: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_ANTHROPIC_UPSTREAM: &str = "https://api.anthropic.com/v1/messages";

static UPSTREAM_CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();

fn get_client() -> &'static Client {
    UPSTREAM_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client")
    })
}

fn upstream_url() -> String {
    std::env::var("FOCUSA_UPSTREAM_URL").unwrap_or_else(|_| DEFAULT_UPSTREAM.into())
}

fn anthropic_upstream_url() -> String {
    std::env::var("FOCUSA_ANTHROPIC_UPSTREAM").unwrap_or_else(|_| DEFAULT_ANTHROPIC_UPSTREAM.into())
}

fn anthropic_api_key(headers: &HeaderMap) -> Option<String> {
    if let Ok(key) = std::env::var("FOCUSA_ANTHROPIC_KEY") {
        return Some(key);
    }
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

fn api_key(headers: &HeaderMap) -> Option<String> {
    if let Ok(key) = std::env::var("FOCUSA_API_KEY") {
        return Some(key);
    }
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
}

/// Create a signal for the Focus Gate.
fn create_signal(kind: SignalKind, summary: impl Into<String>) -> Signal {
    Signal {
        id: uuid::Uuid::now_v7(),
        ts: Utc::now(),
        origin: SignalOrigin::Adapter,
        kind,
        frame_context: None,
        summary: summary.into(),
        payload_ref: None,
        tags: vec![],
    }
}

/// POST /proxy/v1/chat/completions — OpenAI proxy with turn tracking.
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

    let turn_id = uuid::Uuid::now_v7().to_string();

    // 1. TURN START — Record in state.
    let user_input = request.messages.iter()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn = Some(ActiveTurn {
            turn_id: turn_id.clone(),
            adapter_id: "openai-proxy".to_string(),
            harness_name: "openai".to_string(),
            started_at: Utc::now(),
            raw_user_input: Some(user_input.clone()),
            assembled_prompt: None,
        });
    }
    tracing::info!(turn_id = %turn_id, harness = "openai", "Turn started");

    // Emit user_input signal to Focus Gate (summary max 200 chars).
    if !user_input.is_empty() {
        let summary: String = user_input.chars().take(200).collect();
        let signal = create_signal(SignalKind::UserInput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    let url = upstream_url();
    let client = get_client();

    // 2. PROMPT ASSEMBLY — Enhance with Focusa context.
    let focusa_state = state.focusa.read().await;
    let result = openai::process_request(request.clone(), &focusa_state, &state.config);
    drop(focusa_state);

    // Update active turn with assembled prompt if enhancement occurred.
    if let Some(ref proxy_result) = result {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn {
            // Verify turn_id hasn't changed (prevent race with concurrent requests).
            if turn.turn_id == turn_id {
                turn.assembled_prompt = Some(proxy_result.assembly.content.clone());
            }
        }
    }

    // 3. FORWARD TO UPSTREAM.
    let (response, error_str) = match result {
        Some(proxy_result) => {
            match openai::forward_request(client, &url, &key, &proxy_result.request).await {
                Ok(resp) => (resp, None),
                Err(e) => {
                    tracing::error!("Upstream failed: {}", e);
                    (json!({"error": e.to_string()}), Some(e.to_string()))
                }
            }
        }
        None => {
            match passthrough::passthrough(client, &url, &key, &request).await {
                Ok(resp) => (resp, None),
                Err(e) => (json!({"error": e.to_string()}), Some(e.to_string()))
            }
        }
    };

    // 4. TURN COMPLETE — Record result.
    let assistant_output = extract_assistant_output(&response).unwrap_or_default();
    let (prompt_tokens, completion_tokens) = extract_usage(&response);

    // Emit error signal if failed (summary max 200 chars).
    if let Some(ref err) = error_str {
        let summary: String = err.chars().take(200).collect();
        let signal = create_signal(SignalKind::Error, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    // Emit assistant_output signal if success (summary max 200 chars).
    if error_str.is_none() && !assistant_output.is_empty() {
        let summary: String = assistant_output.chars().take(200).collect();
        let signal = create_signal(SignalKind::AssistantOutput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    tracing::info!(
        turn_id = %turn_id,
        success = error_str.is_none(),
        output_len = assistant_output.len(),
        "Turn completed"
    );

    // Clear active_turn to prevent stale data.
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == turn_id
        {
            focusa.active_turn.take();
        }
    }

    // 5. UPDATE FRAME CHECKPOINT.
    if error_str.is_none() {
        let frame_id = {
            let focusa = state.focusa.read().await;
            focusa.focus_stack.active_id
        };

        if let Some(fid) = frame_id {
            let delta = FocusStateDelta {
                current_state: Some(format!(
                    "Turn {}: {} prompt + {} completion tokens",
                    turn_id, prompt_tokens, completion_tokens
                )),
                ..Default::default()
            };
            let _ = state.command_tx.send(Action::UpdateCheckpointDelta {
                frame_id: fid,
                turn_id: turn_id.clone(),
                delta,
            }).await;
        }
    }

    // Return error if upstream failed.
    if let Some(err) = error_str {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": err})),
        ));
    }

    Ok(Json(response))
}

/// POST /proxy/v1/messages — Anthropic proxy with turn tracking.
async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<MessagesRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let key = anthropic_api_key(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No API key — set FOCUSA_ANTHROPIC_KEY or pass x-api-key header"})),
        )
    })?;

    let turn_id = uuid::Uuid::now_v7().to_string();

    // 1. TURN START.
    let user_input = anthropic::extract_user_input(&request.messages);

    {
        let mut focusa = state.focusa.write().await;
        focusa.active_turn = Some(ActiveTurn {
            turn_id: turn_id.clone(),
            adapter_id: "anthropic-proxy".to_string(),
            harness_name: "anthropic".to_string(),
            started_at: Utc::now(),
            raw_user_input: Some(user_input.clone()),
            assembled_prompt: None,
        });
    }
    tracing::info!(turn_id = %turn_id, harness = "anthropic", "Turn started");

    // Emit user_input signal (summary max 200 chars).
    if !user_input.is_empty() {
        let summary: String = user_input.chars().take(200).collect();
        let signal = create_signal(SignalKind::UserInput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    let url = anthropic_upstream_url();
    let client = get_client();

    // 2. PROMPT ASSEMBLY.
    let focusa_state = state.focusa.read().await;
    let result = anthropic::process_request(request.clone(), &focusa_state, &state.config);
    drop(focusa_state);

    // Update active turn with assembled prompt if enhancement occurred.
    if let Some(ref proxy_result) = result {
        let mut focusa = state.focusa.write().await;
        if let Some(ref mut turn) = focusa.active_turn {
            // Verify turn_id hasn't changed (prevent race with concurrent requests).
            if turn.turn_id == turn_id {
                turn.assembled_prompt = Some(proxy_result.assembly.content.clone());
            }
        }
    }

    // 3. FORWARD.
    let (response, error_str) = match result {
        Some(proxy_result) => {
            match anthropic::forward_request(client, &url, &key, &proxy_result.request).await {
                Ok(resp) => (resp, None),
                Err(e) => {
                    tracing::error!("Anthropic upstream failed: {}", e);
                    (json!({"error": e.to_string()}), Some(e.to_string()))
                }
            }
        }
        None => {
            match anthropic::forward_request(client, &url, &key, &request).await {
                Ok(resp) => (resp, None),
                Err(e) => (json!({"error": e.to_string()}), Some(e.to_string()))
            }
        }
    };

    // 4. TURN COMPLETE.
    let assistant_output = anthropic::extract_assistant_output(&response).unwrap_or_default();
    let (input_tokens, output_tokens) = anthropic::extract_usage(&response);

    // Emit error or assistant signal (summary max 200 chars).
    if let Some(ref err) = error_str {
        let summary: String = err.chars().take(200).collect();
        let signal = create_signal(SignalKind::Error, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    } else if !assistant_output.is_empty() {
        let summary: String = assistant_output.chars().take(200).collect();
        let signal = create_signal(SignalKind::AssistantOutput, summary);
        let _ = state.command_tx.send(Action::IngestSignal { signal }).await;
    }

    tracing::info!(
        turn_id = %turn_id,
        success = error_str.is_none(),
        output_len = assistant_output.len(),
        "Turn completed"
    );

    // Clear active_turn to prevent stale data.
    {
        let mut focusa = state.focusa.write().await;
        if let Some(ref turn) = focusa.active_turn
            && turn.turn_id == turn_id
        {
            focusa.active_turn.take();
        }
    }

    // 5. UPDATE FRAME.
    if error_str.is_none() {
        let frame_id = {
            let focusa = state.focusa.read().await;
            focusa.focus_stack.active_id
        };

        if let Some(fid) = frame_id {
            let delta = FocusStateDelta {
                current_state: Some(format!(
                    "Turn {}: {} input + {} output tokens",
                    turn_id, input_tokens, output_tokens
                )),
                ..Default::default()
            };
            let _ = state.command_tx.send(Action::UpdateCheckpointDelta {
                frame_id: fid,
                turn_id: turn_id.clone(),
                delta,
            }).await;
        }
    }

    if let Some(err) = error_str {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": err})),
        ));
    }

    Ok(Json(response))
}

/// POST /proxy/acp — ACP JSON-RPC proxy.
async fn acp_proxy(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use focusa_core::adapters::acp;

    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    let mut msg = acp::parse_message(&bytes).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": e })))
    })?;

    let s = state.focusa.read().await;
    let session_id = s.session.as_ref().map(|s| s.session_id.to_string()).unwrap_or_default();
    let telemetry = acp::observe_message(&session_id, &msg, acp::AcpDirection::ClientToAgent);
    tracing::debug!(method = ?telemetry.method, direction = ?telemetry.direction, "ACP message observed");

    if let Some(active_id) = s.focus_stack.active_id
        && let Some(frame_record) = s.focus_stack.frames.iter().find(|f| f.id == active_id)
    {
        acp::apply_cognition(&mut msg, &frame_record.focus_state, &s.focus_gate);
    }
    drop(s);

    let upstream = std::env::var("FOCUSA_ACP_UPSTREAM").unwrap_or_else(|_| "http://127.0.0.1:4000".into());
    let client = get_client();
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
        .route("/proxy/v1/messages", post(anthropic_messages))
        .route("/proxy/acp", post(acp_proxy))
}

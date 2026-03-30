//! Focus Gate routes.
//!
//! GET  /v1/focus-gate/candidates     — list candidates
//! POST /v1/focus-gate/suppress       — suppress a candidate
//! POST /v1/focus-gate/pin            — pin a candidate
//! POST /v1/focus-gate/surface        — surface a candidate (increase pressure)
//! POST /v1/focus-gate/ingest-signal  — emit signal from adapter
//! POST /v1/gate/signal               — alias for ingest-signal

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::types::{Action, Signal, SignalKind, SignalOrigin};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

async fn candidates(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("gate:read") {
        return Err(forbid("gate:read"));
    }
    let focusa = state.focusa.read().await;
    let threshold = state.config.gate_surface_threshold;
    let surfaced =
        focusa_core::gate::focus_gate::surfaced_candidates(&focusa.focus_gate, threshold);
    Ok(Json(json!({
        "candidates": focusa.focus_gate.candidates,
        "surfaced_count": surfaced.len(),
    })))
}

#[derive(Deserialize)]
struct SuppressBody {
    candidate_id: uuid::Uuid,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_scope() -> String {
    "session".into()
}

async fn suppress(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SuppressBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("commands:submit") {
        return Err(StatusCode::FORBIDDEN);
    }
    state
        .command_tx
        .send(Action::SuppressCandidate {
            candidate_id: body.candidate_id,
            scope: body.scope,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

#[derive(Deserialize)]
struct PinBody {
    candidate_id: uuid::Uuid,
}

async fn pin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<PinBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("commands:submit") {
        return Err(StatusCode::FORBIDDEN);
    }
    state
        .command_tx
        .send(Action::PinCandidate {
            candidate_id: body.candidate_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

/// POST /v1/focus-gate/surface — surface a candidate.
///
/// Increases the candidate's pressure to bring it to attention.
#[derive(Deserialize)]
struct SurfaceBody {
    candidate_id: uuid::Uuid,
    #[serde(default)]
    boost: Option<f32>,
}

async fn surface(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SurfaceBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("commands:submit") {
        return Err(StatusCode::FORBIDDEN);
    }
    state
        .command_tx
        .send(Action::SurfaceCandidate {
            candidate_id: body.candidate_id,
            boost: body.boost.unwrap_or(1.0),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

/// POST /v1/focus-gate/ingest-signal — emit signal from adapter.
///
/// Per spec: adapters emit signals for user input, tool output, errors.
#[derive(Deserialize)]
struct SignalBody {
    kind: String,
    summary: String,
    #[serde(default)]
    frame_context: Option<Uuid>,
}

async fn emit_signal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SignalBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("commands:submit") {
        return Err(StatusCode::FORBIDDEN);
    }
    let kind = match body.kind.as_str() {
        "user_input_received" => SignalKind::UserInput,
        "tool_output_captured" => SignalKind::ToolOutput,
        "error_observed" => SignalKind::Error,
        "repeated_warning" => SignalKind::Warning,
        _ => SignalKind::UserInput,
    };

    let signal = Signal {
        id: Uuid::now_v7(),
        ts: Utc::now(),
        origin: SignalOrigin::Adapter,
        kind,
        frame_context: body.frame_context,
        summary: body.summary,
        payload_ref: None,
        tags: vec![],
    };

    state
        .command_tx
        .send(Action::IngestSignal { signal })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus-gate/candidates", get(candidates))
        .route("/v1/focus-gate/suppress", post(suppress))
        .route("/v1/focus-gate/pin", post(pin))
        .route("/v1/focus-gate/surface", post(surface))
        .route("/v1/focus-gate/ingest-signal", post(emit_signal))
        .route("/v1/gate/signal", post(emit_signal)) // Alias
}

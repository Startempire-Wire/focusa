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
/// Accepts two formats:
///   1. Canonical: { kind, summary, frame_context }
///   2. Extension (§36.2/§36.3): { signal_type, surface, frame_id, payload }
#[derive(Deserialize)]
struct SignalBody {
    // Canonical format
    kind: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    frame_context: Option<Uuid>,
    // Extension format (§36.2/§36.3)
    #[serde(alias = "kind", default)]
    signal_type: Option<String>,
    #[serde(default)]
    surface: Option<String>,
    #[serde(default)]
    frame_id: Option<String>,
    #[serde(default)]
    payload: Option<serde_json::Value>,
}

impl SignalBody {
    fn resolved(&self) -> (String, String, Option<Uuid>) {
        // kind or signal_type → kind string
        let raw = self.kind
            .as_deref()
            .or(self.signal_type.as_deref())
            .unwrap_or("user_input");
        let kind = match raw {
            "tool_error" | "tool_output_captured" => "tool_output_captured",
            "model_error" => "error_observed",
            "user_input" | "user_input_received" => "user_input_received",
            "steering" => "user_input_received",
            "error_observed" => "error_observed",
            "blocker" => "error_observed",
            "failure" => "error_observed",
            "model_change" => "user_input_received",
            "long_session" => "user_input_received",
            "error_rate_high" => "error_observed",
            "file_churn" => "user_input_received",
            "correction" => "user_input_received",
            _ => "user_input_received",
        };
        // Build summary from payload if no explicit summary
        let summary = if let Some(ref s) = self.summary {
            s.clone()
        } else if let Some(ref p) = self.payload {
            if let Some(ref tool) = p.get("tool") {
                format!("Tool error: {}", tool.as_str().unwrap_or("unknown"))
            } else if let Some(ref msg) = p.get("error") {
                format!("Error: {}", msg.as_str().unwrap_or("unknown")[..200].to_string())
            } else if let Some(ref model) = p.get("model_id") {
                format!("Model: {}", model.as_str().unwrap_or("unknown"))
            } else if let Some(ref cnt) = p.get("count") {
                format!("Error rate: {} errors", cnt)
            } else if let Some(ref cnt) = p.get("minutes") {
                format!("Long session: {} min", cnt)
            } else {
                p.as_str().unwrap_or("Signal").to_string()
            }
        } else {
            "Signal".to_string()
        };
        let fc = self.frame_context.or_else(|| {
            self.frame_id.as_ref().and_then(|s| Uuid::parse_str(s).ok())
        });
        (kind.to_string(), summary, fc)
    }
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
    let (kind_str, summary, frame_context) = body.resolved();
    let kind = match kind_str.as_str() {
        "tool_output_captured" => SignalKind::ToolOutput,
        "error_observed" => SignalKind::Error,
        "user_input_received" => SignalKind::UserInput,
        _ => SignalKind::UserInput,
    };

    let signal = Signal {
        id: Uuid::now_v7(),
        ts: Utc::now(),
        origin: SignalOrigin::Adapter,
        kind,
        frame_context,
        summary,
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

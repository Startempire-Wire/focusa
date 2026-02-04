//! Focus Gate routes.
//!
//! GET  /v1/focus-gate/candidates     — list candidates
//! POST /v1/focus-gate/suppress       — suppress a candidate
//! POST /v1/focus-gate/pin            — pin a candidate

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::Action;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn candidates(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    let threshold = state.config.gate_surface_threshold;
    let surfaced = focusa_core::gate::focus_gate::surfaced_candidates(
        &focusa.focus_gate,
        threshold,
    );
    Json(json!({
        "candidates": focusa.focus_gate.candidates,
        "surfaced_count": surfaced.len(),
    }))
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
    Json(body): Json<SuppressBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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
    Json(body): Json<PinBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // CandidatePinned goes directly through the reducer — emit the event.
    state
        .command_tx
        .send(Action::SurfaceCandidate {
            candidate_id: body.candidate_id,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"status": "accepted"})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus-gate/candidates", get(candidates))
        .route("/v1/focus-gate/suppress", post(suppress))
        .route("/v1/focus-gate/pin", post(pin))
}

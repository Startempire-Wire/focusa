//! Focus Gate routes.
//!
//! GET  /v1/focus-gate/candidates
//! POST /v1/focus-gate/ingest-signal
//! POST /v1/focus-gate/suppress
//! POST /v1/focus-gate/surface

use crate::server::AppState;
use axum::{Json, Router, routing::{get, post}};
use serde_json::json;
use std::sync::Arc;

async fn candidates(state: axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "candidates": focusa.focus_gate.candidates,
    }))
}

async fn ingest_signal() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

async fn suppress() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

async fn surface() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus-gate/candidates", get(candidates))
        .route("/v1/focus-gate/ingest-signal", post(ingest_signal))
        .route("/v1/focus-gate/suppress", post(suppress))
        .route("/v1/focus-gate/surface", post(surface))
}

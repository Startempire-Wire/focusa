//! Training Dataset Export & Contribution routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/training/status — export pipeline status.
async fn export_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "contribution_enabled": s.contribution.enabled,
        "queue_size": s.contribution.queue.len(),
        "total_contributed": s.contribution.total_contributed,
        "policy": s.contribution.policy,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/training/status", get(export_status))
}

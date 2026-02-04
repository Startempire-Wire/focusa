//! Constitution routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/constitution/active — active constitution.
async fn active(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    match focusa_core::constitution::active(&s.constitution) {
        Some(c) => Json(serde_json::to_value(c).unwrap_or(json!({}))),
        None => Json(json!({ "error": "No active constitution" })),
    }
}

/// GET /v1/constitution/versions — version history.
async fn versions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    let versions = focusa_core::constitution::version_history(&s.constitution);
    Json(json!({
        "versions": versions,
        "active": s.constitution.active_version,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/constitution/active", get(active))
        .route("/v1/constitution/versions", get(versions))
}

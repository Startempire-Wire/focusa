//! Session routes.
//!
//! GET  /v1/status
//! POST /v1/turn/start
//! POST /v1/turn/complete

use crate::server::AppState;
use axum::{Json, Router, routing::{get, post}};
use serde_json::json;
use std::sync::Arc;

async fn status(state: axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "session": focusa.session,
        "stack_depth": focusa.focus_stack.frames.len(),
        "active_frame_id": focusa.focus_stack.active_id,
        "version": focusa.version,
    }))
}

async fn turn_start() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

async fn turn_complete() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/turn/start", post(turn_start))
        .route("/v1/turn/complete", post(turn_complete))
}

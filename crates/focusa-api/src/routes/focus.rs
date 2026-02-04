//! Focus stack routes.
//!
//! GET  /v1/focus/stack
//! POST /v1/focus/push
//! POST /v1/focus/pop
//! POST /v1/focus/set-active

use crate::server::AppState;
use axum::{Json, Router, routing::{get, post}};
use serde_json::json;
use std::sync::Arc;

async fn get_stack(state: axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "stack": focusa.focus_stack,
        "active_frame_id": focusa.focus_stack.active_id,
    }))
}

async fn push_frame() -> Json<serde_json::Value> {
    // TODO: Parse body, dispatch PushFrame action
    Json(json!({"status": "not_implemented"}))
}

async fn pop_frame() -> Json<serde_json::Value> {
    // TODO: Parse body, dispatch PopFrame action
    Json(json!({"status": "not_implemented"}))
}

async fn set_active() -> Json<serde_json::Value> {
    // TODO: Parse body, dispatch SetActiveFrame action
    Json(json!({"status": "not_implemented"}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/stack", get(get_stack))
        .route("/v1/focus/push", post(push_frame))
        .route("/v1/focus/pop", post(pop_frame))
        .route("/v1/focus/set-active", post(set_active))
}

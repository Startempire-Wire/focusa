//! Memory routes.
//!
//! GET  /v1/memory/semantic
//! POST /v1/memory/semantic/upsert
//! GET  /v1/memory/procedural
//! POST /v1/memory/procedural/reinforce

use crate::server::AppState;
use axum::{Json, Router, routing::{get, post}};
use serde_json::json;
use std::sync::Arc;

async fn semantic(state: axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "semantic": focusa.memory.semantic,
    }))
}

async fn upsert_semantic() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

async fn procedural(state: axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let focusa = state.focusa.read().await;
    Json(json!({
        "procedural": focusa.memory.procedural,
    }))
}

async fn reinforce_rule() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/memory/semantic", get(semantic))
        .route("/v1/memory/semantic/upsert", post(upsert_semantic))
        .route("/v1/memory/procedural", get(procedural))
        .route("/v1/memory/procedural/reinforce", post(reinforce_rule))
}

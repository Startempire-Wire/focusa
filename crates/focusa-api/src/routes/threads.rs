//! Thread routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/threads — list threads (placeholder — threads stored separately).
async fn list_threads(State(_state): State<Arc<AppState>>) -> Json<Value> {
    // Threads are persisted in ~/.focusa/threads/. For now return structure.
    Json(json!({
        "threads": [],
        "note": "Thread listing requires persistence layer — use focusa-cli thread commands"
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/threads", get(list_threads))
}

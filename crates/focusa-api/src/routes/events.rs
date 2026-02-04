//! Event routes.
//!
//! GET /v1/events/recent?limit=200
//! GET /v1/events/stream (SSE — optional)

use crate::server::AppState;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn recent() -> Json<serde_json::Value> {
    Json(json!({
        "events": [],
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/events/recent", get(recent))
}

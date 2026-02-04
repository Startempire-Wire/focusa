//! Event routes.
//!
//! GET /v1/events/recent?limit=200

use crate::server::AppState;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn recent() -> Json<serde_json::Value> {
    // TODO: Read from event log file when persistence is wired in.
    Json(json!({
        "events": [],
        "note": "Event log reading not yet implemented",
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/events/recent", get(recent))
}

//! GET /v1/health

use crate::server::AppState;
use axum::{Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/health", get(health))
}

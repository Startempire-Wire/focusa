//! ECS routes.
//!
//! POST /v1/ecs/store
//! GET  /v1/ecs/resolve/:handle_id

use crate::server::AppState;
use axum::{Json, Router, routing::{get, post}};
use serde_json::json;
use std::sync::Arc;

async fn store_artifact() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

async fn resolve_handle() -> Json<serde_json::Value> {
    Json(json!({"status": "not_implemented"}))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ecs/store", post(store_artifact))
        .route("/v1/ecs/resolve/{handle_id}", get(resolve_handle))
}

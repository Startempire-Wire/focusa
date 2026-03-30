//! Constitution routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/constitution/active — active constitution.
async fn active(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("constitution:read") {
        return Err(forbid("constitution:read"));
    }
    let s = state.focusa.read().await;
    match focusa_core::constitution::active(&s.constitution) {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or(json!({})))),
        None => Ok(Json(json!({ "error": "No active constitution" }))),
    }
}

/// GET /v1/constitution/versions — version history.
async fn versions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (axum::http::StatusCode, axum::Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("constitution:read") {
        return Err(forbid("constitution:read"));
    }
    let s = state.focusa.read().await;
    let versions = focusa_core::constitution::version_history(&s.constitution);
    Ok(Json(json!({
        "versions": versions,
        "active": s.constitution.active_version,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/constitution/active", get(active))
        .route("/v1/constitution/versions", get(versions))
}

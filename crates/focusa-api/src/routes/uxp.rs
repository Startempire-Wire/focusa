//! UXP / UFI routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/uxp — UXP profile.
async fn uxp_profile(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(serde_json::to_value(&s.uxp).unwrap_or(json!({})))
}

/// GET /v1/ufi — UFI state.
async fn ufi_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "aggregate": s.ufi.aggregate,
        "signal_count": s.ufi.signals.len(),
        "signals": s.ufi.signals,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/uxp", get(uxp_profile))
        .route("/v1/ufi", get(ufi_state))
}

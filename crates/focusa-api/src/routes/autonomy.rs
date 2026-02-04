//! Autonomy Calibration routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/autonomy — autonomy state.
async fn autonomy_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "level": s.autonomy.level,
        "ari_score": s.autonomy.ari_score,
        "dimensions": s.autonomy.dimensions,
        "sample_count": s.autonomy.sample_count,
        "granted_scope": s.autonomy.granted_scope,
        "granted_ttl": s.autonomy.granted_ttl,
        "recommendation": focusa_core::autonomy::should_recommend_promotion(&s.autonomy),
        "history_count": s.autonomy.history.len(),
    }))
}

/// GET /v1/autonomy/history — autonomy event history.
async fn history(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({ "history": s.autonomy.history }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/autonomy", get(autonomy_state))
        .route("/v1/autonomy/history", get(history))
}

//! RFM (Reliability Focus Mode) routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

/// GET /v1/rfm — RFM state.
async fn rfm_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "level": s.rfm.level,
        "ais_score": s.rfm.ais_score,
        "needs_regeneration": focusa_core::rfm::needs_regeneration(&s.rfm),
        "needs_ensemble": focusa_core::rfm::needs_ensemble(&s.rfm),
        "validator_results": s.rfm.validator_results,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/rfm", get(rfm_state))
}

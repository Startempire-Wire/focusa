//! Telemetry routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/telemetry/tokens — token usage metrics.
async fn tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "total_events": s.telemetry.total_events,
        "total_prompt_tokens": s.telemetry.total_prompt_tokens,
        "total_completion_tokens": s.telemetry.total_completion_tokens,
        "tokens_per_task": s.telemetry.tokens_per_task,
    }))
}

/// GET /v1/telemetry/cost — cost estimate.
async fn cost(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    // Default pricing: $0.003/1K prompt, $0.015/1K completion (Claude-class).
    let estimated = focusa_core::telemetry::estimate_cost(&s.telemetry, 0.003, 0.015);
    Json(json!({
        "estimated_cost_usd": estimated,
        "prompt_tokens": s.telemetry.total_prompt_tokens,
        "completion_tokens": s.telemetry.total_completion_tokens,
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/telemetry/tokens", get(tokens))
        .route("/v1/telemetry/cost", get(cost))
}

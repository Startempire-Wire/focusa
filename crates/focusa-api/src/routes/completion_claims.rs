//! Completion claim evaluation route (#276 slice 2): every harness
//! evaluates completion claims through the SAME deterministic core —
//! no per-surface verdict logic.

use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use focusa_core::completion_authority::{
    evaluate_completion_claim, CompletionClaim,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/completion-claims/evaluate", post(evaluate))
}

async fn evaluate(
    State(_state): State<Arc<AppState>>,
    Json(claim): Json<CompletionClaim>,
) -> Json<Value> {
    let verdict = evaluate_completion_claim(&claim);
    Json(json!({
        "status": "evaluated",
        "verdict": verdict,
    }))
}

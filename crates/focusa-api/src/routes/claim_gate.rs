use axum::{Json, Router, routing::post};
use focusa_core::claim_gate::{CompletionClaimRequest, completion_claim_gate};
use serde_json::json;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/claim/preclose", post(preclose))
}

async fn preclose(Json(body): Json<CompletionClaimRequest>) -> Json<serde_json::Value> {
    Json(json!(completion_claim_gate(body)))
}

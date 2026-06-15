use axum::{Json, Router, routing::get};
use focusa_core::utility_card::utility_card;
use serde_json::json;
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/utility/card", get(card))
        .route("/v1/utility/bootstrap", get(bootstrap))
        .route("/v1/utility/post-compaction", get(post_compaction))
}

async fn card() -> Json<serde_json::Value> {
    Json(json!(utility_card()))
}

async fn bootstrap() -> Json<serde_json::Value> {
    let card = utility_card();
    Json(json!({
        "schema": "focusa.utility_bootstrap.v1",
        "status": card.status,
        "authority_boundary": card.authority_boundary,
        "usefulness_bar": card.usefulness_bar,
        "scope_gate": card.scope_gate,
        "bootstrap_card": card.bootstrap_card,
        "exact_next_actions": card.exact_next_actions,
        "do_not_drift": card.do_not_drift,
        "next_tools": card.next_tools,
    }))
}

async fn post_compaction() -> Json<serde_json::Value> {
    let card = utility_card();
    Json(json!({
        "schema": "focusa.utility_post_compaction.v1",
        "status": card.status,
        "authority_boundary": card.authority_boundary,
        "usefulness_bar": card.usefulness_bar,
        "post_compaction_card": card.post_compaction_card,
        "exact_next_actions": card.exact_next_actions,
        "do_not_drift": card.do_not_drift,
        "evidence_policy": card.evidence_policy,
        "recovery_order": card.recovery_order,
        "next_tools": card.next_tools,
    }))
}

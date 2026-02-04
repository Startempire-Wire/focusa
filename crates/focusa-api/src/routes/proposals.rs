//! PRE (Proposal Resolution Engine) routes.

use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/proposals — list pending proposals.
async fn list_proposals(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "proposals": s.pre.proposals,
        "pending": focusa_core::pre::pending_count(&s.pre),
    }))
}

/// POST /v1/proposals — submit a proposal.
async fn submit_proposal(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let kind_str = body.get("kind").and_then(|v| v.as_str()).unwrap_or("focus_change");
    let source = body.get("source").and_then(|v| v.as_str()).unwrap_or("api");
    let payload = body.get("payload").cloned().unwrap_or(json!({}));
    let deadline_ms = body.get("deadline_ms").and_then(|v| v.as_u64()).unwrap_or(5000);

    let kind = match kind_str {
        "focus_change" => focusa_core::types::ProposalKind::FocusChange,
        "thesis_update" => focusa_core::types::ProposalKind::ThesisUpdate,
        "autonomy_adjustment" => focusa_core::types::ProposalKind::AutonomyAdjustment,
        "constitution_revision" => focusa_core::types::ProposalKind::ConstitutionRevision,
        "memory_write" => focusa_core::types::ProposalKind::MemoryWrite,
        _ => focusa_core::types::ProposalKind::FocusChange,
    };

    let mut s = state.focusa.write().await;
    let id = focusa_core::pre::submit(&mut s.pre, kind, source, payload, deadline_ms);
    Json(json!({ "status": "accepted", "proposal_id": id }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/proposals", get(list_proposals).post(submit_proposal))
}

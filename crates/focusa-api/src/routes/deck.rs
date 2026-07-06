//! Mission Deck read-first API routes (Spec 117 §17.3).
//!
//! These routes are intentionally read-only launch surfaces. They expose Deck
//! metadata, walkthrough catalog hints, Recall card schema, proof meter states,
//! and next-safe-action model hints without creating canonical Workpoint
//! authority or mutating daemon state.

use crate::server::AppState;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

pub const DECK_SCHEMA: &str = "focusa.deck.v1";
pub const WALKTHROUGH_SCHEMA: &str = "focusa.walkthrough.v1";
pub const RECALL_CARD_SCHEMA: &str = "focusa.recall_deck_card.v1";
pub const PROOF_METER_STATES: &[&str] = &["none:[-----]", "linked:[##---]", "verified:[#####]"];
pub const SCOPE_BADGE_STATES: &[&str] = &["canonical", "advisory", "blocked", "unbound"];
pub const NEXT_SAFE_ACTION_STATES: &[&str] = &[
    "disconnected:start_daemon",
    "unbound:bind_project",
    "no_workpoint:create_workpoint",
    "no_evidence:attach_evidence",
    "resumable:resume_mission",
    "blocked:review_scope_before_acting",
];

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/deck/home", get(home))
        .route("/v1/deck/walkthroughs", get(walkthroughs))
        .route("/v1/deck/recall/schema", get(recall_schema))
        .route("/v1/deck/proof-meter", get(proof_meter))
        .route("/v1/deck/next-safe-action", get(next_safe_action))
}

async fn home() -> Json<Value> {
    Json(json!({
        "schema": DECK_SCHEMA,
        "title": "Focusa Mission Deck",
        "default_tab": "DeckHome",
        "read_only": true,
        "surfaces": [
            "/v1/deck/home",
            "/v1/deck/walkthroughs",
            "/v1/deck/recall/schema",
            "/v1/deck/proof-meter",
            "/v1/deck/next-safe-action"
        ],
        "launch_focus": [
            "bind project",
            "create/resume Workpoint",
            "attach evidence",
            "show next safe action",
            "teach authority boundaries"
        ]
    }))
}

async fn walkthroughs() -> Json<Value> {
    Json(json!({
        "schema": WALKTHROUGH_SCHEMA,
        "read_only": true,
        "catalog": [
            "first-mission",
            "agent-handoff",
            "no-proof-no-done"
        ],
        "storage": "~/.focusa/deck/walkthroughs/{project_hash}.jsonl",
        "event_types": ["started", "advanced", "completed", "reset", "blocked"]
    }))
}

async fn recall_schema() -> Json<Value> {
    Json(json!({
        "schema": RECALL_CARD_SCHEMA,
        "read_only": true,
        "authority": "advisory_only",
        "full_spec_expansion_bead": "focusa-117-arch.29",
        "fields": [
            "result_id",
            "provider",
            "source_session_id",
            "project_root",
            "continuity_id",
            "timestamp",
            "span_type",
            "memory_status",
            "scope_status",
            "proof_status",
            "allowed_use",
            "safe_excerpt",
            "evidence_refs",
            "next_action"
        ],
        "memory_status_values": ["active", "stale", "superseded", "contradicted", "noise", "quarantined"],
        "scope_status_values": ["current", "same_project_other_continuity", "other_project", "global_advisory"],
        "proof_status_values": ["none", "linked", "verified"],
        "allowed_use_values": ["include", "inspect_only", "verify_first", "exclude"],
        "forbidden": ["recall_direct_canonical_write", "promotion_without_operator_approval"]
    }))
}

async fn proof_meter() -> Json<Value> {
    Json(json!({
        "schema": DECK_SCHEMA,
        "read_only": true,
        "proof_meter_states": PROOF_METER_STATES,
        "scope_badge_states": SCOPE_BADGE_STATES,
        "recovery": "attach proof or declare an explicit proof gap before claiming done"
    }))
}

async fn next_safe_action() -> Json<Value> {
    Json(json!({
        "schema": DECK_SCHEMA,
        "read_only": true,
        "states": NEXT_SAFE_ACTION_STATES,
        "rule": "show one primary next safe action, with why before commands"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_cover_market_deck_surfaces() {
        assert_eq!(DECK_SCHEMA, "focusa.deck.v1");
        assert!(PROOF_METER_STATES.contains(&"verified:[#####]"));
        assert!(SCOPE_BADGE_STATES.contains(&"blocked"));
        assert!(NEXT_SAFE_ACTION_STATES.contains(&"resumable:resume_mission"));
    }
}

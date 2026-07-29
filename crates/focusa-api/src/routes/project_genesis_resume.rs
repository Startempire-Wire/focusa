//! No-fork Project Genesis resume flow.

use super::super::project_genesis_support::{
    ProjectGenesisRequest, build_staged_packet, canonical_root, clean, packet_path, read_json,
    reject, write_json_atomic,
};
use super::crist::record_crist_transition;
use super::{continuity_access, enrich_from_existing_trajectory, start};
use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::Value;
use std::sync::Arc;

pub(super) async fn resume(
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if clean(Some(&req.continuity_id)).is_none() || clean(Some(&req.idempotency_key)).is_none() {
        return Err(reject(
            StatusCode::BAD_REQUEST,
            "missing_authority",
            "continuity_id and idempotency_key are required",
        ));
    }
    let root = canonical_root(&req.project_root)?;
    let Some(existing) = read_json(&packet_path(&root)) else {
        return start(State(state), Json(req)).await;
    };
    let existing_continuity_id = existing["ownership"]["continuity_id"]
        .as_str()
        .unwrap_or_default();
    let same_owner = continuity_access(
        &req,
        existing_continuity_id,
        "Another project workstream currently owns the staged Project Genesis state.",
    )?;
    if !same_owner {
        return start(State(state), Json(req)).await;
    }
    if existing["status"] == "ready" {
        return Ok(Json(existing));
    }

    let existing_hlt = enrich_from_existing_trajectory(&state, &root, &mut req).await;
    let mut packet = build_staged_packet(&root, &req, existing_hlt);
    let target_stage = packet["crist_stage"]
        .as_str()
        .unwrap_or("project_scope_verified")
        .to_string();
    for key in [
        "genesis_id",
        "created_at",
        "ownership",
        "crist_stage",
        "revision",
        "transition_receipts",
        "receipts",
        "role",
        "interview",
        "spec",
    ] {
        if !existing[key].is_null() {
            packet[key] = existing[key].clone();
        }
    }
    packet["resolved_project_operating_profile"]["crist_state"] =
        existing["resolved_project_operating_profile"]["crist_state"].clone();
    if packet["crist_stage"] == "project_scope_verified" && target_stage == "context_collecting" {
        record_crist_transition(
            &root,
            &mut packet,
            "context_collecting",
            "resume_context_collection",
        )
        .map_err(|receipt| (StatusCode::CONFLICT, Json(receipt)))?;
    }
    write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_journal_write_failed",
            error,
        )
    })?;
    Ok(Json(packet))
}

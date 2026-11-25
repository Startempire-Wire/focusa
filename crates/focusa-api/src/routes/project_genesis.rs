//! Spec143 §6-7 Project Genesis transaction.
//! Identity, Trajectory, task path, first Workpoint, and coordination become ready together.

use crate::routes::project::project_identity_payload_for_scope;
use crate::routes::workpoint::materialize_workpoint_events;
use crate::scope::{ScopeContext, ScopeSource};
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::scoped_state::ScopeRef;
use focusa_core::types::{HltLedgerEntry, WorkpointStatus};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;

use super::project_genesis_support::*;
#[path = "project_genesis_records.rs"]
mod records;
use records::{compile_genesis_records, genesis_events};

async fn enrich_from_existing_trajectory(
    state: &Arc<AppState>,
    root: &Path,
    req: &mut ProjectGenesisRequest,
) -> Option<String> {
    let focusa = state.focusa.read().await;
    let active = focusa.trajectory.active_trajectory_id.as_deref();
    let record = focusa.trajectory.records.iter().rev().find(|record| {
        record.canonical
            && record.hlt_status.is_action_ready()
            && record.project_root.as_deref() == Some(root.to_string_lossy().as_ref())
            && record.continuity_id.as_deref() == Some(req.continuity_id.as_str())
            && active.map(|id| id == record.trajectory_id).unwrap_or(true)
    })?;
    req.desired_end_state
        .get_or_insert_with(|| record.desired_end_state.clone());
    if req.current_state.is_none() {
        req.current_state = record.current_state.clone();
    }
    if req.mid_level_goal.is_none() {
        req.mid_level_goal = record.mid_level_goal.clone();
    }
    if req.short_term_goal.is_none() {
        req.short_term_goal = record.short_term_goal.clone();
    }
    if req.waypoints.is_empty() {
        req.waypoints = record
            .waypoints
            .iter()
            .map(|waypoint| waypoint.title.clone())
            .collect();
    }
    if req.specification_ref.is_none() {
        req.specification_ref = record
            .source_refs
            .get("specification_ref")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    if req.acceptance_criteria.is_empty()
        && let Some(definition) = record.definition_of_done.as_ref()
    {
        req.acceptance_criteria = definition.criteria.clone();
    }
    Some(record.long_term_goal.clone())
}

fn existing_readiness_gate(
    root: &Path,
    req: &ProjectGenesisRequest,
) -> Result<Option<Value>, (StatusCode, Json<Value>)> {
    let marker = read_json(&root.join(".focusa-project.json")).unwrap_or(Value::Null);
    let binding = &marker["genesis_binding"];
    if binding["status"] != "ready" {
        return Ok(None);
    }
    let same_continuity = binding["continuity_id"].as_str() == Some(req.continuity_id.as_str());
    let packet = read_json(&packet_path(root)).filter(|packet| packet["status"] == "ready");
    let replay = packet.as_ref().is_some_and(|packet| {
        packet["idempotency_key"].as_str() == Some(req.idempotency_key.as_str())
    });
    // A new, explicitly confirmed reconciliation may replace a historical
    // ready receipt in the SAME continuity, using the atomic constructor.
    // Exact replays still return the original receipt without new events.
    if req.takeover == Some(true) && req.confirm == Some(true) && !(same_continuity && replay) {
        return Ok(None);
    }
    if same_continuity {
        return Ok(packet);
    }
    if req.takeover == Some(true) {
        return Err(reject(
            StatusCode::PRECONDITION_REQUIRED,
            "takeover_confirmation_required",
            "Take over requires explicit confirmation",
        ));
    }
    Err((
        StatusCode::CONFLICT,
        Json(json!({
            "schema": GENESIS_SCHEMA,
            "status": "coordination_conflict",
            "code": "active_agent_coordination_conflict",
            "message": "Another project workstream currently owns the active first Workpoint.",
            "choices": [
                "View current work",
                "Coordinate with that agent",
                "Take over with confirmation",
                "Continue read-only"
            ],
            "next_action": "choose one plain-language coordination option",
            "existing_continuity_id": binding["continuity_id"],
            "requested_continuity_id": req.continuity_id,
        })),
    ))
}

pub(super) async fn start(
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
    if let Some(packet) = existing_readiness_gate(&root, &req)? {
        return Ok(Json(packet));
    }
    let existing_hlt = enrich_from_existing_trajectory(&state, &root, &mut req).await;
    let packet = build_staged_packet(&root, &req, existing_hlt);
    write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_journal_write_failed",
            error,
        )
    })?;
    Ok(Json(packet))
}

#[path = "project_genesis_status.rs"]
mod status_projection;

#[path = "project_genesis_remote.rs"]
mod remote;

async fn status(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ProjectGenesisStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if query.remote_binding_id.is_some() {
        return remote::status(&state, &headers, &query, None).await;
    }
    status_projection::read(scope, query).await
}

async fn resume(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.remote_binding_id.is_some() {
        return remote::resume(&state, &headers, req).await;
    }
    // Reuse the existing marker/continuity readiness owner: a ready journal
    // alone is not the final commit point and may belong to another Workstream.
    start(State(state), Json(req)).await
}

pub(super) async fn commit(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(mut req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.confirm != Some(true) {
        return Err(reject(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "commit requires confirm=true",
        ));
    }
    if req.remote_binding_id.is_some() {
        return remote::commit(&state, &headers, req).await;
    }
    let root = canonical_root(&req.project_root)?;
    if let Some(packet) = existing_readiness_gate(&root, &req)? {
        return Ok(Json(packet));
    }
    let existing_hlt = enrich_from_existing_trajectory(&state, &root, &mut req).await;
    let mut packet = build_staged_packet(&root, &req, existing_hlt);
    if packet["status"] != "staged" {
        write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
            reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "genesis_journal_write_failed",
                error,
            )
        })?;
        return Err((StatusCode::PRECONDITION_FAILED, Json(packet)));
    }
    packet["status"] = json!("preparing");
    write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_journal_write_failed",
            error,
        )
    })?;

    let project_identity = project_identity_payload_for_scope(
        Some(root.to_string_lossy().as_ref()),
        Some(root.to_string_lossy().as_ref()),
        None,
    );
    let scope_ref: ScopeRef = project_identity
        .pointer("/project_identity/scope_ref")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| {
            reject(
                StatusCode::PRECONDITION_FAILED,
                "verified_project_scope_required",
                "Project Genesis requires a verified typed ProjectIdentity ScopeRef",
            )
        })?;
    let (trajectory_id, workpoint_id) =
        commit_genesis_records(&state, &root, &req, &packet, scope_ref).await?;
    let owner_id = stable_id("coordination", &root, &req.idempotency_key);
    packet["status"] = json!("ready");
    packet["first_workpoint"] =
        json!({"workpoint_id": workpoint_id, "status": "active", "canonical": true});
    packet["coordination_owner"] = json!({"owner_id": owner_id, "workpoint_id": workpoint_id, "status": "active", "internal": true});
    packet["readiness_receipt"] = json!({"receipt_id": stable_id("ready", &root, &req.idempotency_key), "trajectory_id": trajectory_id, "workpoint_id": workpoint_id, "marker_guard": "verified", "recorded_at": Utc::now().to_rfc3339()});
    packet["missing_links"] = json!([]);
    packet["next_action"] = json!("continue from the active first Workpoint");
    write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_ready_write_failed",
            error,
        )
    })?;

    let marker_path = root.join(".focusa-project.json");
    let mut marker = read_json(&marker_path).ok_or_else(|| {
        reject(
            StatusCode::PRECONDITION_FAILED,
            "project_marker_unreadable",
            "cannot verify project marker",
        )
    })?;
    marker["genesis_binding"] = json!({"schema": GENESIS_SCHEMA, "status": "ready", "trajectory_id": trajectory_id, "workpoint_id": workpoint_id, "receipt_id": packet["readiness_receipt"]["receipt_id"], "continuity_id": req.continuity_id});
    write_json_atomic(&marker_path, &marker).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_marker_commit_failed",
            error,
        )
    })?;
    Ok(Json(packet))
}
/// Materialize the canonical records from verified scope and staged inventory.
/// The caller owns onboarding admission, journaling and readiness publication.
async fn commit_genesis_records(
    state: &Arc<AppState>,
    root: &Path,
    req: &ProjectGenesisRequest,
    packet: &Value,
    scope_ref: ScopeRef,
) -> Result<(String, Uuid), (StatusCode, Json<Value>)> {
    let (trajectory, workpoint) = compile_genesis_records(root, req, packet, scope_ref);
    let trajectory_id = trajectory.trajectory_id.clone();
    let workpoint_id = workpoint.workpoint_id;
    let (already_committed, previous_trajectory) = {
        let focusa = state.focusa.read().await;
        let committed = focusa.workpoint.records.iter().any(|record| {
            record.workpoint_id == workpoint_id && record.status == WorkpointStatus::Active
        });
        let previous = focusa
            .trajectory
            .records
            .iter()
            .rev()
            .find(|record| {
                record.canonical
                    && record.project_root.as_deref() == Some(root.to_string_lossy().as_ref())
                    && record.continuity_id.as_deref() == Some(req.continuity_id.as_str())
            })
            .cloned();
        (committed, previous)
    };
    let hlt_entry = HltLedgerEntry::new(
        root.to_string_lossy().to_string(),
        trajectory.long_term_goal.clone(),
        "project_genesis_commit",
        state
            .external_mutation_epoch
            .load(std::sync::atomic::Ordering::Acquire)
            + 1,
    )
    .with_old_hlt(
        previous_trajectory
            .as_ref()
            .map(|record| record.long_term_goal.clone()),
    )
    .with_scope(Some(req.continuity_id.clone()), None)
    .with_reason(Some("project_genesis_commit".into()));
    let ladder_events = super::trajectory::trajectory_commit_events(
        &trajectory,
        previous_trajectory.as_ref(),
        &hlt_entry,
        &[],
        Some(req.idempotency_key.as_str()),
    );
    if !already_committed {
        let scope = ScopeContext {
            project_root: Some(root.to_string_lossy().to_string()),
            continuity_id: Some(req.continuity_id.clone()),
            source: ScopeSource::Query,
            ..ScopeContext::default()
        };
        materialize_workpoint_events(
            scope,
            state,
            genesis_events(trajectory, workpoint),
            "project_genesis_commit",
        )
        .await?;
    }

    // A canonical projection without its scoped HLT ladder is not ready.
    // Reuse the goal-definition event compiler and fail before publishing
    // readiness if either append fails. Retries use the same replay key.
    state
        .persistence
        .append_trajectory_ladder_events(&ladder_events)
        .map_err(|error| {
            reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "genesis_ladder_commit_failed",
                error.to_string(),
            )
        })?;
    state
        .persistence
        .append_hlt_ledger_entry(&hlt_entry)
        .map_err(|error| {
            reject(
                StatusCode::INTERNAL_SERVER_ERROR,
                "genesis_hlt_commit_failed",
                error.to_string(),
            )
        })?;
    Ok((trajectory_id, workpoint_id))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/genesis/start", post(remote::start_or_preview))
        .route("/v1/project/genesis/resume", post(resume))
        .route("/v1/project/genesis/status", get(status))
        .route("/v1/project/genesis/commit", post(commit))
}

#[cfg(test)]
#[path = "project_genesis_tests.rs"]
mod tests;

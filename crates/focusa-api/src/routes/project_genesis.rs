//! Spec143 §6-7 Project Genesis transaction.
//! Identity, Trajectory, task path, first Workpoint, and coordination become ready together.

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
use focusa_core::types::{
    FocusaEvent, HltStatus, TrajectoryConfidence, TrajectoryDefinitionStatus,
    TrajectoryGoalProvenanceRecord, TrajectoryProjectionRecord, TrajectoryRootGoalStability,
    TrajectoryWaypointRecord, TrajectoryWaypointStatus, WorkpointActionIntentRecord,
    WorkpointCheckpointReason, WorkpointConfidence, WorkpointRecord, WorkpointStatus,
};
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

async fn existing_hlt(state: &Arc<AppState>, root: &Path, continuity_id: &str) -> Option<String> {
    let focusa = state.focusa.read().await;
    let active = focusa.trajectory.active_trajectory_id.as_deref();
    focusa
        .trajectory
        .records
        .iter()
        .rev()
        .find(|record| {
            record.canonical
                && record.hlt_status.is_action_ready()
                && record.project_root.as_deref() == Some(root.to_string_lossy().as_ref())
                && record.continuity_id.as_deref() == Some(continuity_id)
                && active.map(|id| id == record.trajectory_id).unwrap_or(true)
        })
        .map(|record| record.long_term_goal.clone())
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
    if same_continuity {
        return Ok(read_json(&packet_path(root)).filter(|packet| packet["status"] == "ready"));
    }
    if req.takeover == Some(true) && req.confirm == Some(true) {
        return Ok(None);
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

async fn start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProjectGenesisRequest>,
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
    let packet = build_staged_packet(
        &root,
        &req,
        existing_hlt(&state, &root, &req.continuity_id).await,
    );
    write_json_atomic(&packet_path(&root), &packet).map_err(|error| {
        reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "genesis_journal_write_failed",
            error,
        )
    })?;
    Ok(Json(packet))
}

async fn status(
    Query(query): Query<ProjectGenesisStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = canonical_root(&query.project_root)?;
    read_json(&packet_path(&root)).map(Json).ok_or_else(|| {
        reject(
            StatusCode::NOT_FOUND,
            "genesis_not_started",
            "call project genesis start",
        )
    })
}

async fn resume(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let root = canonical_root(&req.project_root)?;
    if let Some(packet) = read_json(&packet_path(&root))
        && packet.get("idempotency_key").and_then(Value::as_str)
            == Some(req.idempotency_key.as_str())
        && packet.get("status").and_then(Value::as_str) == Some("ready")
    {
        return Ok(Json(packet));
    }
    start(State(state), Json(req)).await
}

async fn commit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProjectGenesisRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.confirm != Some(true) {
        return Err(reject(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "commit requires confirm=true",
        ));
    }
    let root = canonical_root(&req.project_root)?;
    if let Some(packet) = existing_readiness_gate(&root, &req)? {
        return Ok(Json(packet));
    }
    let mut packet = build_staged_packet(
        &root,
        &req,
        existing_hlt(&state, &root, &req.continuity_id).await,
    );
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

    let trajectory_id = stable_id("trajectory", &root, &req.idempotency_key);
    let workpoint_id = stable_uuid(&root, &req.idempotency_key);
    let hlt = packet["hlt"].as_str().unwrap_or_default().to_string();
    let mlg = packet["mlg"].as_str().unwrap_or_default().to_string();
    let stg = packet["stg"].as_str().unwrap_or_default().to_string();
    let waypoint_titles = packet["waypoints"].as_array().cloned().unwrap_or_default();
    let first_task = packet["first_workpoint_candidate"].clone();
    let now = Utc::now();
    let trajectory = TrajectoryProjectionRecord {
        trajectory_id: trajectory_id.clone(),
        project_root: Some(root.to_string_lossy().to_string()),
        continuity_id: Some(req.continuity_id.clone()),
        root_long_term_goal: hlt.clone(),
        long_term_goal: hlt.clone(),
        desired_end_state: req.desired_end_state.clone().unwrap_or_default(),
        mid_level_goal: Some(mlg.clone()),
        short_term_goal: Some(stg.clone()),
        waypoints: waypoint_titles
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value.as_str().map(|title| TrajectoryWaypointRecord {
                    waypoint_id: format!("{trajectory_id}:waypoint:{}", index + 1),
                    title: title.to_string(),
                    desired_state_delta: title.to_string(),
                    status: if index == 0 {
                        TrajectoryWaypointStatus::Active
                    } else {
                        TrajectoryWaypointStatus::NotStarted
                    },
                    next_workpoint_candidate: if index == 0 {
                        first_task.clone()
                    } else {
                        Value::Null
                    },
                    ..TrajectoryWaypointRecord::default()
                })
            })
            .collect(),
        current_state: req.current_state.clone(),
        root_goal_stability: TrajectoryRootGoalStability::Stable,
        session_clarity_status: TrajectoryDefinitionStatus::Clear,
        gap_summary: Some(format!("{mlg} → {stg}")),
        active_waypoint_id: Some(format!("{trajectory_id}:waypoint:1")),
        active_workpoint_id: Some(workpoint_id),
        source_refs: json!({"specification_ref": req.specification_ref, "genesis_receipt": packet["bootstrap_receipt"]["receipt_id"]}),
        blockers: Vec::new(),
        open_questions: Vec::new(),
        definition_status: TrajectoryDefinitionStatus::Clear,
        hlt_status: HltStatus::CanonicalExplicit,
        confidence: TrajectoryConfidence::High,
        goal_provenance: vec![
            TrajectoryGoalProvenanceRecord {
                field: "hlt".into(),
                source: "project_genesis".into(),
                source_ref: req.specification_ref.clone(),
                inferred: false,
                confidence: TrajectoryConfidence::High,
            },
            TrajectoryGoalProvenanceRecord {
                field: "mlg_stg_waypoints".into(),
                source: "spec143_deliberate_inference".into(),
                source_ref: req.specification_ref.clone(),
                inferred: req.mid_level_goal.is_none(),
                confidence: TrajectoryConfidence::Medium,
            },
        ],
        canonical: true,
        created_at: Some(now),
        updated_at: Some(now),
        ..TrajectoryProjectionRecord::default()
    };
    let work_item_id = first_task
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mission = first_task
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&stg)
        .to_string();
    let workpoint = WorkpointRecord {
        workpoint_id,
        work_item_id,
        continuity_id: Some(req.continuity_id.clone()),
        project_root: Some(root.to_string_lossy().to_string()),
        status: WorkpointStatus::Proposed,
        checkpoint_reason: WorkpointCheckpointReason::SessionStart,
        confidence: WorkpointConfidence::High,
        canonical: true,
        mission: Some(mission.clone()),
        action_intent: Some(WorkpointActionIntentRecord {
            action_type: "project_genesis_first_workpoint".into(),
            target_ref: req.specification_ref.clone(),
            verification_hooks: req.acceptance_criteria.clone(),
            status: Some("ready".into()),
        }),
        next_slice: Some(mission),
        idempotency_key: Some(req.idempotency_key.clone()),
        ..WorkpointRecord::default()
    };
    let already_committed = {
        let focusa = state.focusa.read().await;
        focusa.workpoint.records.iter().any(|record| {
            record.workpoint_id == workpoint_id && record.status == WorkpointStatus::Active
        })
    };
    if !already_committed {
        let scope = ScopeContext {
            project_root: Some(root.to_string_lossy().to_string()),
            continuity_id: Some(req.continuity_id.clone()),
            source: ScopeSource::Query,
            ..ScopeContext::default()
        };
        materialize_workpoint_events(
            scope,
            &state,
            vec![
                FocusaEvent::TrajectoryGoalDefined { trajectory },
                FocusaEvent::WorkpointCheckpointProposed { workpoint },
                FocusaEvent::WorkpointCheckpointPromoted {
                    workpoint_id,
                    confidence: WorkpointConfidence::High,
                    reason: "Project Genesis atomic first Workpoint".into(),
                },
            ],
            "project_genesis_commit",
        )
        .await?;
    }

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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/project/genesis/start", post(start))
        .route("/v1/project/genesis/resume", post(resume))
        .route("/v1/project/genesis/status", get(status))
        .route("/v1/project/genesis/commit", post(commit))
}

#[cfg(test)]
#[path = "project_genesis_tests.rs"]
mod tests;

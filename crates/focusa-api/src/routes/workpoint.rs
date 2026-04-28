//! Spec88 Workpoint continuity API routes.

use crate::routes::permissions::{forbid, permission_context};
use crate::server::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::{get, post}};
use focusa_core::types::{
    Action, FocusaEvent, WorkpointActionIntentRecord, WorkpointCheckpointReason,
    WorkpointConfidence, WorkpointDriftSeverity, WorkpointRecord, WorkpointStatus,
    WorkpointVerificationRecord,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointCheckpointRequest {
    pub workpoint_id: Option<Uuid>,
    pub work_item_id: Option<String>,
    pub session_id: Option<String>,
    pub frame_id: Option<Uuid>,
    pub checkpoint_reason: Option<WorkpointCheckpointReason>,
    pub confidence: Option<WorkpointConfidence>,
    pub canonical: Option<bool>,
    pub mission: Option<String>,
    pub active_object_refs: Option<Vec<String>>,
    pub action_intent: Option<WorkpointActionIntentRecord>,
    pub verification_records: Option<Vec<WorkpointVerificationRecord>>,
    pub next_slice: Option<String>,
    pub source_turn_id: Option<String>,
    pub promote: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointResumeRequest {
    pub workpoint_id: Option<Uuid>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointDriftCheckRequest {
    pub workpoint_id: Option<Uuid>,
    pub latest_action: Option<String>,
    pub expected_action_type: Option<String>,
    pub emit: Option<bool>,
}

fn active_workpoint<'a>(state: &'a focusa_core::types::FocusaState) -> Option<&'a WorkpointRecord> {
    state
        .workpoint
        .active_workpoint_id
        .and_then(|id| state.workpoint.records.iter().find(|record| record.workpoint_id == id))
}

fn workpoint_packet(record: &WorkpointRecord) -> Value {
    json!({
        "workpoint_id": record.workpoint_id,
        "work_item_id": record.work_item_id,
        "session_id": record.session_id,
        "frame_id": record.frame_id,
        "status": record.status,
        "checkpoint_reason": record.checkpoint_reason,
        "confidence": record.confidence,
        "canonical": record.canonical,
        "mission": record.mission,
        "active_object_refs": record.active_object_refs,
        "action_intent": record.action_intent,
        "verification_records": record.verification_records,
        "blockers": record.blockers,
        "next_slice": record.next_slice,
        "source_turn_id": record.source_turn_id,
        "updated_at": record.updated_at,
    })
}

fn resume_summary(record: &WorkpointRecord) -> String {
    let action = record
        .action_intent
        .as_ref()
        .map(|intent| intent.action_type.as_str())
        .unwrap_or("unknown_action");
    let next = record.next_slice.as_deref().unwrap_or("continue from active workpoint");
    format!(
        "WORKPOINT {}: mission={}; action={}; next={}; canonical={}",
        record.workpoint_id,
        record.mission.as_deref().unwrap_or("unknown"),
        action,
        next,
        record.canonical
    )
}

async fn dispatch_event(
    state: &Arc<AppState>,
    event: FocusaEvent,
) -> Result<(), (StatusCode, Json<Value>)> {
    state
        .command_tx
        .send(Action::EmitEvent { event })
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "rejected",
                    "canonical": false,
                    "error": format!("dispatch failed: {error}"),
                    "next_step_hint": "retry after daemon command channel recovers"
                })),
            )
        })
}

async fn checkpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointCheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    if req.mission.as_deref().unwrap_or("").trim().is_empty()
        && req.next_slice.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "rejected",
                "canonical": false,
                "error": "mission or next_slice is required",
                "next_step_hint": "provide typed continuation content before checkpointing"
            })),
        ));
    }

    let workpoint_id = req.workpoint_id.unwrap_or_else(Uuid::now_v7);
    let promote = req.promote.unwrap_or(true);
    let record = WorkpointRecord {
        workpoint_id,
        work_item_id: req.work_item_id,
        session_id: req.session_id,
        frame_id: req.frame_id,
        status: WorkpointStatus::Proposed,
        checkpoint_reason: req.checkpoint_reason.unwrap_or(WorkpointCheckpointReason::Manual),
        confidence: req.confidence.unwrap_or(WorkpointConfidence::High),
        canonical: req.canonical.unwrap_or(true),
        mission: req.mission,
        active_object_refs: req.active_object_refs.unwrap_or_default(),
        action_intent: req.action_intent,
        verification_records: req.verification_records.unwrap_or_default(),
        next_slice: req.next_slice,
        source_turn_id: req.source_turn_id,
        ..WorkpointRecord::default()
    };
    let canonical = record.canonical;

    dispatch_event(
        &state,
        FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
    )
    .await?;
    if promote && canonical {
        dispatch_event(
            &state,
            FocusaEvent::WorkpointCheckpointPromoted {
                workpoint_id,
                confidence: req.confidence.unwrap_or(WorkpointConfidence::High),
                reason: "checkpoint API promote=true".to_string(),
            },
        )
        .await?;
    }

    Ok(Json(json!({
        "status": if promote && canonical { "accepted" } else { "partial" },
        "workpoint_id": workpoint_id,
        "canonical": canonical,
        "warnings": if promote && !canonical { vec!["non-canonical checkpoint was proposed but not promoted"] } else { vec![] },
        "next_step_hint": "call /v1/workpoint/resume to render the packet for Pi continuation"
    })))
}

async fn current(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    let focusa = state.focusa.read().await;
    let Some(record) = active_workpoint(&focusa) else {
        return Ok(Json(json!({
            "status": "not_found",
            "canonical": false,
            "workpoint_id": null,
            "warnings": ["no active workpoint"],
            "next_step_hint": "POST /v1/workpoint/checkpoint before compacting or resuming"
        })));
    };
    Ok(Json(json!({
        "status": "completed",
        "workpoint_id": record.workpoint_id,
        "canonical": record.canonical,
        "workpoint": workpoint_packet(record),
        "warnings": [],
        "next_step_hint": record.next_slice,
    })))
}

async fn resume(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointResumeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    let focusa = state.focusa.read().await;
    let record = req
        .workpoint_id
        .and_then(|id| focusa.workpoint.records.iter().find(|record| record.workpoint_id == id))
        .or_else(|| active_workpoint(&focusa));
    let Some(record) = record else {
        return Ok(Json(json!({
            "status": "not_found",
            "canonical": false,
            "workpoint_id": null,
            "warnings": ["no workpoint available to resume"],
            "next_step_hint": "checkpoint the current mission/action before retrying resume"
        })));
    };
    let workpoint_id = record.workpoint_id;
    let canonical = record.canonical;
    let packet = workpoint_packet(record);
    let summary = resume_summary(record);
    drop(focusa);

    dispatch_event(
        &state,
        FocusaEvent::WorkpointResumeRendered {
            workpoint_id: Some(workpoint_id),
            mode: req.mode.unwrap_or_else(|| "compact_prompt".to_string()),
            rendered_summary: summary.clone(),
        },
    )
    .await?;

    Ok(Json(json!({
        "status": "completed",
        "workpoint_id": workpoint_id,
        "canonical": canonical,
        "resume_packet": packet,
        "rendered_summary": summary,
        "warnings": if canonical { vec![] } else { vec!["resume packet is non-canonical degraded fallback"] },
        "next_step_hint": "inject rendered_summary plus resume_packet before the next Pi turn"
    })))
}

async fn drift_check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointDriftCheckRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    let focusa = state.focusa.read().await;
    let record = req
        .workpoint_id
        .and_then(|id| focusa.workpoint.records.iter().find(|record| record.workpoint_id == id))
        .or_else(|| active_workpoint(&focusa));
    let Some(record) = record else {
        return Ok(Json(json!({
            "status": "not_found",
            "canonical": false,
            "warnings": ["no active workpoint for drift check"],
            "next_step_hint": "resume/checkpoint a workpoint first"
        })));
    };

    let expected = req.expected_action_type.or_else(|| {
        record
            .action_intent
            .as_ref()
            .map(|intent| intent.action_type.clone())
    });
    let latest = req.latest_action.unwrap_or_default();
    let drift_detected = expected
        .as_ref()
        .map(|expected| !latest.is_empty() && !latest.contains(expected))
        .unwrap_or(false);
    let workpoint_id = record.workpoint_id;
    drop(focusa);

    if req.emit.unwrap_or(false) && drift_detected {
        dispatch_event(
            &state,
            FocusaEvent::WorkpointDriftDetected {
                workpoint_id: Some(workpoint_id),
                severity: WorkpointDriftSeverity::High,
                reason: format!(
                    "latest action did not match expected action {}",
                    expected.clone().unwrap_or_else(|| "unknown".to_string())
                ),
                recovery_hint: Some("resume from active WorkpointResumePacket".to_string()),
            },
        )
        .await?;
    }

    Ok(Json(json!({
        "status": if drift_detected { "drift_detected" } else { "no_drift" },
        "workpoint_id": workpoint_id,
        "canonical": true,
        "drift_detected": drift_detected,
        "expected_action_type": expected,
        "warnings": [],
        "next_step_hint": if drift_detected { "call /v1/workpoint/resume and realign before continuing" } else { "continue current action" }
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/workpoint/checkpoint", post(checkpoint))
        .route("/v1/workpoint/current", get(current))
        .route("/v1/workpoint/resume", post(resume))
        .route("/v1/workpoint/drift-check", post(drift_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_summary_is_bounded_and_action_oriented() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            mission: Some("Keep Pi on typed workpoint".to_string()),
            canonical: true,
            next_slice: Some("Patch compaction hook".to_string()),
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "patch_component_binding".to_string(),
                target_ref: Some("apps/pi-extension/src/compaction.ts".to_string()),
                verification_hooks: vec![],
                status: Some("ready".to_string()),
            }),
            ..WorkpointRecord::default()
        };
        let summary = resume_summary(&record);
        assert!(summary.contains("patch_component_binding"));
        assert!(summary.contains("Patch compaction hook"));
    }

    #[test]
    fn workpoint_packet_contains_next_slice_and_canonical_flag() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            canonical: true,
            next_slice: Some("Resume from packet".to_string()),
            ..WorkpointRecord::default()
        };
        let packet = workpoint_packet(&record);
        assert_eq!(packet.get("canonical").and_then(Value::as_bool), Some(true));
        assert_eq!(
            packet.get("next_slice").and_then(Value::as_str),
            Some("Resume from packet")
        );
    }
}

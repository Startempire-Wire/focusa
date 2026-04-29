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
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

static WORKPOINT_IDEMPOTENCY_CACHE: LazyLock<Mutex<HashMap<String, WorkpointRecord>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointCheckpointRequest {
    pub workpoint_id: Option<Uuid>,
    pub work_item_id: Option<String>,
    pub session_id: Option<String>,
    pub frame_id: Option<Uuid>,
    pub checkpoint_reason: Option<String>,
    pub confidence: Option<WorkpointConfidence>,
    pub canonical: Option<bool>,
    pub mission: Option<String>,
    pub active_object_refs: Option<Vec<String>>,
    pub action_intent: Option<WorkpointActionIntentRecord>,
    pub verification_records: Option<Vec<WorkpointVerificationRecord>>,
    pub next_slice: Option<String>,
    pub source_turn_id: Option<String>,
    pub promote: Option<bool>,
    pub idempotency_key: Option<String>,
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
    pub active_object_refs: Option<Vec<String>>,
    pub do_not_drift: Option<Vec<String>>,
    pub emit: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointEvidenceLinkRequest {
    pub workpoint_id: Option<Uuid>,
    pub target_ref: String,
    pub result: String,
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ActiveObjectResolveRequest {
    pub hint: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct DriftDecision {
    drift_detected: bool,
    severity: WorkpointDriftSeverity,
    reason: String,
    recovery_hint: String,
    drift_classes: Vec<String>,
}

fn normalize_for_match(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn object_tokens(value: &str) -> Vec<String> {
    normalize_for_match(value)
        .split_whitespace()
        .filter(|token| token.len() >= 4)
        .map(ToString::to_string)
        .collect()
}

fn latest_tokens(value: &str) -> Vec<String> {
    normalize_for_match(value)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn latest_mentions_object(latest: &str, object_ref: &str) -> bool {
    let latest_norm = normalize_for_match(latest);
    if latest_norm.is_empty() {
        return false;
    }
    let object_norm = normalize_for_match(object_ref);
    if !object_norm.is_empty() && latest_norm.contains(&object_norm) {
        return true;
    }
    let latest_tokens = latest_tokens(latest);
    object_tokens(object_ref)
        .iter()
        .any(|token| latest_tokens.iter().any(|latest_token| latest_token == token))
}

fn classify_drift(
    record: &WorkpointRecord,
    latest_action: &str,
    expected_action_type: Option<&str>,
    request_objects: &[String],
    request_boundaries: &[String],
) -> DriftDecision {
    let latest_norm = normalize_for_match(latest_action);
    let action = expected_action_type
        .or_else(|| record.action_intent.as_ref().map(|intent| intent.action_type.as_str()))
        .unwrap_or("");
    let action_norm = normalize_for_match(action);
    let mut classes = Vec::new();
    let mut reasons = Vec::new();

    if latest_norm.is_empty() {
        return DriftDecision {
            drift_detected: false,
            severity: WorkpointDriftSeverity::Info,
            reason: "latest action is empty; no drift decision".to_string(),
            recovery_hint: "continue current action".to_string(),
            drift_classes: vec![],
        };
    }

    let notes_only_markers = ["note", "notes", "document", "docs", "breadcrumb", "summary", "handoff"];
    let implementation_markers = ["implement", "patch", "edit", "verify", "test", "run", "inspect", "fix"];
    let action_requires_execution = action_norm.contains("patch")
        || action_norm.contains("implement")
        || action_norm.contains("verify")
        || action_norm.contains("binding")
        || action_norm.contains("resume workpoint");
    if action_requires_execution
        && notes_only_markers.iter().any(|marker| latest_norm.contains(marker))
        && !implementation_markers.iter().any(|marker| latest_norm.contains(marker))
    {
        classes.push("notes_only_drift".to_string());
        reasons.push("latest action appears notes-only while Workpoint requires implementation or verification".to_string());
    }

    let mut active_objects = record.active_object_refs.clone();
    active_objects.extend(request_objects.iter().cloned());
    if let Some(target) = record.action_intent.as_ref().and_then(|intent| intent.target_ref.clone()) {
        active_objects.push(target);
    }
    active_objects.sort();
    active_objects.dedup();
    if !active_objects.is_empty()
        && !active_objects.iter().any(|object| latest_mentions_object(latest_action, object))
    {
        classes.push("wrong_object_drift".to_string());
        reasons.push("latest action does not mention any active target object or action target".to_string());
    }

    let mut boundaries: Vec<String> = request_boundaries.to_vec();
    if let Some(next) = &record.next_slice {
        boundaries.extend(
            next.lines()
                .filter_map(|line| line.split_once("DO_NOT_DRIFT:").map(|(_, rhs)| rhs.trim().to_string())),
        );
    }
    for boundary in boundaries.iter().filter(|boundary| !boundary.trim().is_empty()) {
        if latest_mentions_object(latest_action, boundary) || latest_norm.contains(&normalize_for_match(boundary)) {
            classes.push("do_not_drift_boundary".to_string());
            reasons.push(format!("latest action touches prohibited boundary: {boundary}"));
            break;
        }
    }

    if !action_norm.is_empty() && !latest_norm.contains(&action_norm) {
        let action_terms: Vec<_> = action_norm.split_whitespace().filter(|term| term.len() >= 4).collect();
        if !action_terms.is_empty() && !action_terms.iter().any(|term| latest_norm.contains(term)) {
            classes.push("action_intent_ignored".to_string());
            reasons.push(format!("latest action does not align with expected action {action}"));
        }
    }

    let drift_detected = !classes.is_empty();
    DriftDecision {
        drift_detected,
        severity: if classes.iter().any(|class| class == "do_not_drift_boundary" || class == "wrong_object_drift") {
            WorkpointDriftSeverity::High
        } else if drift_detected {
            WorkpointDriftSeverity::Medium
        } else {
            WorkpointDriftSeverity::Info
        },
        reason: if reasons.is_empty() { "latest action aligns with active Workpoint".to_string() } else { reasons.join("; ") },
        recovery_hint: if drift_detected { "call /v1/workpoint/resume and continue the packet next_slice before adjacent work".to_string() } else { "continue current action".to_string() },
        drift_classes: classes,
    }
}

fn active_workpoint<'a>(state: &'a focusa_core::types::FocusaState) -> Option<&'a WorkpointRecord> {
    state
        .workpoint
        .active_workpoint_id
        .and_then(|id| state.workpoint.records.iter().find(|record| record.workpoint_id == id))
}

fn parse_checkpoint_reason(reason: Option<&str>) -> Result<WorkpointCheckpointReason, (StatusCode, Json<Value>)> {
    let Some(reason) = reason.map(str::trim).filter(|reason| !reason.is_empty()) else {
        return Ok(WorkpointCheckpointReason::Manual);
    };
    match reason {
        "session-start" | "session_start" => Ok(WorkpointCheckpointReason::SessionStart),
        "session-resume" | "session_resume" => Ok(WorkpointCheckpointReason::SessionResume),
        "before-compact" | "before_compact" => Ok(WorkpointCheckpointReason::BeforeCompact),
        "after-compact" | "after_compact" => Ok(WorkpointCheckpointReason::AfterCompact),
        "context-overflow" | "context_overflow" => Ok(WorkpointCheckpointReason::ContextOverflow),
        "model-switch" | "model_switch" => Ok(WorkpointCheckpointReason::ModelSwitch),
        "fork" => Ok(WorkpointCheckpointReason::Fork),
        "operator-checkpoint" | "operator_checkpoint" => Ok(WorkpointCheckpointReason::OperatorCheckpoint),
        "manual" => Ok(WorkpointCheckpointReason::Manual),
        "unknown" => Ok(WorkpointCheckpointReason::Unknown),
        other => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "checkpoint_reason",
                "rejected_value": other,
                "allowed_values": [
                    "manual",
                    "operator_checkpoint",
                    "session_start",
                    "session_resume",
                    "before_compact",
                    "after_compact",
                    "context_overflow",
                    "model_switch",
                    "fork",
                    "unknown"
                ],
                "retry_posture": "do_not_retry_unchanged",
                "next_step_hint": "choose a supported checkpoint_reason or omit it to use manual"
            })),
        )),
    }
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
        "idempotency_key": record.idempotency_key,
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

async fn wait_for_active_workpoint(state: &Arc<AppState>, workpoint_id: Uuid) -> Option<WorkpointRecord> {
    for _ in 0..240 {
        {
            let focusa = state.focusa.read().await;
            if focusa.workpoint.active_workpoint_id == Some(workpoint_id) {
                if let Some(record) = focusa.workpoint.records.iter().find(|record| record.workpoint_id == workpoint_id) {
                    return Some(record.clone());
                }
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    None
}

async fn wait_for_workpoint_evidence(state: &Arc<AppState>, workpoint_id: Uuid, evidence_ref: Option<&str>, target_ref: &str, result: &str) -> Option<WorkpointRecord> {
    for _ in 0..240 {
        {
            let focusa = state.focusa.read().await;
            if let Some(record) = focusa.workpoint.records.iter().find(|record| record.workpoint_id == workpoint_id) {
                let linked = record.verification_records.iter().any(|verification| {
                    verification.target_ref == target_ref
                        && verification.result == result
                        && verification.evidence_ref.as_deref() == evidence_ref
                });
                if linked {
                    return Some(record.clone());
                }
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    None
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

    if let Some(key) = req.idempotency_key.as_ref().filter(|key| !key.trim().is_empty()) {
        if let Some(existing) = WORKPOINT_IDEMPOTENCY_CACHE.lock().ok().and_then(|cache| cache.get(key).cloned()) {
            return Ok(Json(json!({
                "status": "completed",
                "workpoint_id": existing.workpoint_id,
                "canonical": existing.canonical,
                "idempotent_replay": true,
                "workpoint": workpoint_packet(&existing),
                "warnings": [],
                "next_step_hint": "idempotency key already accepted; call /v1/workpoint/resume to render the packet"
            })));
        }
        let focusa = state.focusa.read().await;
        if let Some(existing) = focusa
            .workpoint
            .records
            .iter()
            .find(|record| record.idempotency_key.as_deref() == Some(key.as_str()))
        {
            if let Ok(mut cache) = WORKPOINT_IDEMPOTENCY_CACHE.lock() {
                cache.insert(key.clone(), existing.clone());
            }
            return Ok(Json(json!({
                "status": "completed",
                "workpoint_id": existing.workpoint_id,
                "canonical": existing.canonical,
                "idempotent_replay": true,
                "workpoint": workpoint_packet(existing),
                "warnings": [],
                "next_step_hint": "idempotency key already recorded; call /v1/workpoint/resume to render the packet"
            })));
        }
    }

    let workpoint_id = req.workpoint_id.unwrap_or_else(Uuid::now_v7);
    let promote = req.promote.unwrap_or(true);
    let idempotency_key = req.idempotency_key.clone();
    let record = WorkpointRecord {
        workpoint_id,
        work_item_id: req.work_item_id,
        session_id: req.session_id,
        frame_id: req.frame_id,
        status: WorkpointStatus::Proposed,
        checkpoint_reason: parse_checkpoint_reason(req.checkpoint_reason.as_deref())?,
        confidence: req.confidence.unwrap_or(WorkpointConfidence::High),
        canonical: req.canonical.unwrap_or(true),
        mission: req.mission,
        active_object_refs: req.active_object_refs.unwrap_or_default(),
        action_intent: req.action_intent,
        verification_records: req.verification_records.unwrap_or_default(),
        next_slice: req.next_slice,
        source_turn_id: req.source_turn_id,
        idempotency_key: req.idempotency_key,
        ..WorkpointRecord::default()
    };
    let canonical = record.canonical;

    dispatch_event(
        &state,
        FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
    )
    .await?;
    let mut promoted_record = None;
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
        promoted_record = wait_for_active_workpoint(&state, workpoint_id).await;
        if promoted_record.is_none() {
            return Err((
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "pending",
                    "workpoint_id": workpoint_id,
                    "canonical": canonical,
                    "idempotent_replay": false,
                    "warnings": ["checkpoint accepted but active Workpoint promotion has not materialized yet"],
                    "next_step_hint": "retry /v1/workpoint/current before relying on this Workpoint"
                })),
            ));
        }
    }
    if let (Some(key), Some(record)) = (idempotency_key.as_ref().filter(|key| !key.trim().is_empty()), promoted_record.as_ref()) {
        if let Ok(mut cache) = WORKPOINT_IDEMPOTENCY_CACHE.lock() {
            cache.insert(key.clone(), record.clone());
        }
    }

    Ok(Json(json!({
        "status": if promote && canonical { "accepted" } else { "partial" },
        "workpoint_id": workpoint_id,
        "canonical": canonical,
        "idempotent_replay": false,
        "workpoint": promoted_record.as_ref().map(workpoint_packet),
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

async fn resolve_active_object(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActiveObjectResolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;
    let record = active_workpoint(&focusa);
    let mut refs: Vec<String> = record
        .map(|record| record.active_object_refs.clone())
        .unwrap_or_default();
    if let Some(work_item_id) = record.and_then(|record| record.work_item_id.clone()) {
        refs.push(work_item_id);
    }
    if let Some(target_ref) = record
        .and_then(|record| record.action_intent.as_ref())
        .and_then(|intent| intent.target_ref.clone())
    {
        refs.push(target_ref);
    }
    if let Some(hint) = req.hint.filter(|hint| !hint.trim().is_empty()) {
        refs.push(hint);
    }
    refs.sort();
    refs.dedup();
    Ok(Json(json!({
        "status": "completed",
        "canonical": record.is_some(),
        "workpoint_id": record.map(|record| record.workpoint_id),
        "refs": refs,
        "verified": false,
        "next_step_hint": "treat refs as candidates unless verified by a canonical object read"
    })))
}

async fn link_evidence(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointEvidenceLinkRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    if req.target_ref.trim().is_empty() || req.result.trim().is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
            "status": "validation_rejected",
            "canonical": false,
            "field": "target_ref/result",
            "retry_posture": "do_not_retry_unchanged",
            "next_step_hint": "provide target_ref and result before linking Workpoint evidence"
        }))));
    }
    let focusa = state.focusa.read().await;
    let record = if let Some(workpoint_id) = req.workpoint_id {
        focusa.workpoint.records.iter().find(|record| record.workpoint_id == workpoint_id).cloned()
    } else {
        active_workpoint(&focusa).cloned()
    };
    let Some(record) = record else {
        return Err((StatusCode::NOT_FOUND, Json(json!({
            "status": "blocked",
            "canonical": false,
            "error": "no active Workpoint to link evidence",
            "next_step_hint": "create or resume a canonical Workpoint before linking evidence"
        }))));
    };
    drop(focusa);
    let workpoint_id = record.workpoint_id;
    let verification = WorkpointVerificationRecord {
        target_ref: req.target_ref,
        result: req.result,
        evidence_ref: req.evidence_ref,
        verified_at: None,
    };
    dispatch_event(&state, FocusaEvent::WorkpointEvidenceLinked {
        workpoint_id,
        verification: verification.clone(),
    }).await?;
    let linked_record = wait_for_workpoint_evidence(
        &state,
        workpoint_id,
        verification.evidence_ref.as_deref(),
        &verification.target_ref,
        &verification.result,
    ).await;
    if linked_record.is_none() {
        return Err((
            StatusCode::ACCEPTED,
            Json(json!({
                "status": "pending",
                "canonical": true,
                "workpoint_id": workpoint_id,
                "verification": verification,
                "warnings": ["evidence link accepted but is not visible in Workpoint state yet"],
                "next_step_hint": "retry /v1/workpoint/resume before relying on this evidence link"
            })),
        ));
    }
    Ok(Json(json!({
        "status": "accepted",
        "canonical": true,
        "workpoint_id": workpoint_id,
        "verification": verification,
        "workpoint": linked_record.as_ref().map(workpoint_packet),
        "next_step_hint": "call /v1/workpoint/resume to see linked evidence in the packet"
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
    if req.emit.unwrap_or(false) && !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
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

    let expected = req.expected_action_type.clone().or_else(|| {
        record
            .action_intent
            .as_ref()
            .map(|intent| intent.action_type.clone())
    });
    let latest = req.latest_action.clone().unwrap_or_default();
    let request_objects = req.active_object_refs.clone().unwrap_or_default();
    let request_boundaries = req.do_not_drift.clone().unwrap_or_default();
    let decision = classify_drift(
        record,
        &latest,
        expected.as_deref(),
        &request_objects,
        &request_boundaries,
    );
    let workpoint_id = record.workpoint_id;
    let canonical = record.canonical;
    drop(focusa);

    if req.emit.unwrap_or(false) && decision.drift_detected {
        dispatch_event(
            &state,
            FocusaEvent::WorkpointDriftDetected {
                workpoint_id: Some(workpoint_id),
                severity: decision.severity,
                reason: decision.reason.clone(),
                recovery_hint: Some(decision.recovery_hint.clone()),
            },
        )
        .await?;
    }

    Ok(Json(json!({
        "status": if decision.drift_detected { "drift_detected" } else { "no_drift" },
        "workpoint_id": workpoint_id,
        "canonical": canonical,
        "drift_detected": decision.drift_detected,
        "drift_classes": decision.drift_classes,
        "severity": decision.severity,
        "reason": decision.reason,
        "recovery_hint": decision.recovery_hint,
        "expected_action_type": expected,
        "warnings": [],
        "next_step_hint": if decision.drift_detected { "call /v1/workpoint/resume and realign before continuing" } else { "continue current action" }
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/workpoint/checkpoint", post(checkpoint))
        .route("/v1/workpoint/current", get(current))
        .route("/v1/workpoint/resume", post(resume))
        .route("/v1/workpoint/active-object/resolve", post(resolve_active_object))
        .route("/v1/workpoint/evidence/link", post(link_evidence))
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
            idempotency_key: Some("idem-1".to_string()),
            ..WorkpointRecord::default()
        };
        let packet = workpoint_packet(&record);
        assert_eq!(packet.get("canonical").and_then(Value::as_bool), Some(true));
        assert_eq!(
            packet.get("next_slice").and_then(Value::as_str),
            Some("Resume from packet")
        );
        assert_eq!(packet.get("idempotency_key").and_then(Value::as_str), Some("idem-1"));
    }

    #[test]
    fn drift_classifier_flags_notes_only_wrong_object_and_boundaries() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            canonical: true,
            active_object_refs: vec!["Component:homepage.audio_widget".to_string()],
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "patch_component_binding".to_string(),
                target_ref: Some("Component:homepage.audio_widget".to_string()),
                verification_hooks: vec!["verify UI play state".to_string()],
                status: Some("ready".to_string()),
            }),
            next_slice: Some("Patch the widget binding\nDO_NOT_DRIFT: notes-only/generic validation".to_string()),
            ..WorkpointRecord::default()
        };
        let decision = classify_drift(
            &record,
            "Updated notes and generic validation summary for unrelated backend endpoint",
            None,
            &[],
            &[],
        );
        assert!(decision.drift_detected);
        assert!(decision.drift_classes.contains(&"notes_only_drift".to_string()));
        assert!(decision.drift_classes.contains(&"wrong_object_drift".to_string()));
    }

    #[test]
    fn drift_classifier_accepts_matching_target_and_action_term() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            canonical: true,
            active_object_refs: vec!["Component:homepage.audio_widget".to_string()],
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "patch_component_binding".to_string(),
                target_ref: Some("Component:homepage.audio_widget".to_string()),
                verification_hooks: vec![],
                status: Some("ready".to_string()),
            }),
            ..WorkpointRecord::default()
        };
        let decision = classify_drift(
            &record,
            "Patch homepage audio widget component binding and verify play pause state",
            None,
            &[],
            &[],
        );
        assert!(!decision.drift_detected, "{}", decision.reason);
    }

    #[test]
    fn checkpoint_reason_accepts_operator_checkpoint_and_rejects_unknown_field_value() {
        assert_eq!(
            parse_checkpoint_reason(Some("operator_checkpoint")).unwrap(),
            WorkpointCheckpointReason::OperatorCheckpoint
        );
        let err = parse_checkpoint_reason(Some("not_a_reason")).unwrap_err();
        assert_eq!(err.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.1.0.get("status").and_then(Value::as_str), Some("validation_rejected"));
        assert_eq!(err.1.0.get("field").and_then(Value::as_str), Some("checkpoint_reason"));
    }

    #[test]
    fn drift_classifier_does_not_match_boundary_tokens_inside_compound_words() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            canonical: true,
            active_object_refs: vec!["FocusaToolSuite".to_string()],
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "stress_verify".to_string(),
                target_ref: Some("FocusaToolSuite".to_string()),
                verification_hooks: vec!["api".to_string(), "cli".to_string(), "pi".to_string()],
                status: Some("ready".to_string()),
            }),
            next_slice: Some("Complete stress suite\nDO_NOT_DRIFT: Do not demote existing tools.".to_string()),
            ..WorkpointRecord::default()
        };
        let decision = classify_drift(
            &record,
            "stress verify FocusaToolSuite api cli pi",
            Some("stress_verify"),
            &[],
            &[],
        );
        assert!(!decision.drift_detected, "{}", decision.reason);
    }
}

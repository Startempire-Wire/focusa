//! Focus stack routes.
//!
//! GET  /v1/focus/stack   — read current stack
//! POST /v1/focus/push    — push a new frame
//! POST /v1/focus/pop     — pop (complete) active frame
//! POST /v1/focus/set-active — switch active frame
//! POST /v1/focus/update  — update focus state delta (ASCC)
//! GET  /v1/focusa/enabled — get focusa toggle state (Pi-session-local)
//! PATCH /v1/focusa/enabled — set focusa toggle state

use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, patch, post},
};
use chrono::Utc;
use focusa_core::reducer;
use focusa_core::scope_safety::classify_project_root_option;
use focusa_core::types::{
    CompletionReason, EventLogEntry, FocusStackState, FocusStateDelta, FocusaEvent, FocusaState,
    FrameRecord, FrameStatus, SessionState, SessionStatus, SignalOrigin, WorkpointStatus,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

type FocusResult = Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>;
type FocusUnitResult = Result<(), (StatusCode, Json<serde_json::Value>)>;

fn focus_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let error = error.into();
    let why = why.into();
    let retry_safe = !matches!(failure_class, "validation_rejected" | "not_found");
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    let reflex_suggestions = crate::routes::reflex::reflex_suggestions_for_failure(failure_class);
    (
        http_status,
        Json(json!({
            "status": "blocked", "canonical": false, "degraded": true,
            "error": error, "failure_class": failure_class, "why": why,
            "recovery_hint": recovery_hint, "misuse_hint": misuse_hint,
            "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"],
            "reflex_suggestions": reflex_suggestions,
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class}, "recovery_hint": recovery_hint, "misuse_hint": misuse_hint, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_tool_doctor", "focusa_workpoint_resume"], "reflex_suggestions": reflex_suggestions, "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn focus_reducer_failed(error: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    focus_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("focus reducer rejected event: {error}"),
        "reducer_rejected",
        "Focus reducer rejected the requested focus mutation.",
        "Refresh scoped frame/session state, verify project_root/continuity_id, and retry after correcting the request.",
        "Likely stale frame/session identity, invalid focus transition, or unsafe project context.",
    )
}

fn focus_persistence_failed(
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    focus_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("focus persistence failed: {error}"),
        "persistence_unavailable",
        "Focus event could not be persisted after reducer emission.",
        "Check SQLite/daemon health, verify project database writability, then retry after recovery.",
        "Likely database lock, wrong project database, or daemon shutdown during focus mutation.",
    )
}

fn focus_active_frame_missing() -> (StatusCode, Json<serde_json::Value>) {
    focus_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        "active focus frame missing",
        "frame_unavailable",
        "Focus pop could not find an active frame after validation.",
        "Refresh /v1/focus/frame/current or checkpoint a fresh Workpoint before retrying pop.",
        "Likely stale active frame state or concurrent focus-stack mutation.",
    )
}

fn focus_toggle_io_failed(
    operation: &str,
    path: &std::path::Path,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    focus_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "focus toggle {operation} failed for {}: {error}",
            path.display()
        ),
        "persistence_unavailable",
        format!("Focusa toggle file {operation} could not complete."),
        "Check config.data_dir path, filesystem permissions, and disk health before retrying.",
        "Likely unwritable Pi config directory, missing parent permissions, or filesystem pressure.",
    )
}

#[derive(Debug, Clone, Deserialize, Default)]
struct FocusStackQuery {
    limit: Option<usize>,
    cursor: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ScopedFrameQuery {
    frame_id: Option<Uuid>,
    session_key: Option<String>,
    continuity_id: Option<String>,
    project_root: Option<String>,
}

fn normalize_project_root_authority(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unsafe_project_root_reason(value: Option<&str>) -> Option<&'static str> {
    classify_project_root_option(value).reason()
}

fn clean_scope_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn frame_matches_project_root(frame: &FrameRecord, project_root: Option<&str>) -> bool {
    let Some(expected) = project_root
        .map(normalize_project_root_authority)
        .filter(|value| !value.is_empty())
    else {
        return unsafe_project_root_reason(frame.project_root.as_deref()).is_none();
    };
    unsafe_project_root_reason(Some(&expected)).is_none()
        && frame
            .project_root
            .as_deref()
            .map(normalize_project_root_authority)
            .as_deref()
            == Some(expected.as_str())
}

fn unsafe_project_root_response(reason: &'static str, value: Option<&str>) -> serde_json::Value {
    json!({
        "status": "rejected_unsafe_project_root",
        "canonical": false,
        "failure_class": "scope_mismatch",
        "unsafe_reason": reason,
        "project_root": value,
        "retry_posture": "do_not_retry_unchanged",
        "safe_recovery": "use an exact project folder/root before creating or reading project-bound Focus frames",
    })
}

fn exact_request_scope_matches(
    scope: &ScopeContext,
    project_root: &str,
    continuity_id: &str,
) -> Result<(), Value> {
    let Some(request_root) = clean_scope_value(scope.project_root.as_deref()) else {
        return Err(json!({
            "status": "rejected_missing_scope",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "missing": "x-scope-project-root",
        }));
    };
    let Some(request_continuity) = clean_scope_value(scope.continuity_id.as_deref()) else {
        return Err(json!({
            "status": "rejected_missing_scope",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "missing": "x-scope-continuity-id",
        }));
    };
    if unsafe_project_root_reason(Some(request_root.as_str())).is_some()
        || normalize_project_root_authority(&request_root)
            != normalize_project_root_authority(project_root)
        || request_continuity != continuity_id.trim()
    {
        return Err(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "requested_project_root": request_root,
            "requested_continuity_id": request_continuity,
        }));
    }
    Ok(())
}

fn frame_matches_exact_request_scope(scope: &ScopeContext, frame: &FrameRecord) -> bool {
    let Some(project_root) = frame.project_root.as_deref() else {
        return false;
    };
    let Some(continuity_id) = frame.continuity_id.as_deref() else {
        return false;
    };
    exact_request_scope_matches(scope, project_root, continuity_id).is_ok()
}

fn beads_issue_exists(project_root: &str, beads_issue_id: &str) -> bool {
    let issue_id = beads_issue_id.trim();
    if issue_id.is_empty() || unsafe_project_root_reason(Some(project_root)).is_some() {
        return false;
    }
    let issue_paths = [
        Path::new(project_root).join(".beads/issues.jsonl"),
        Path::new(project_root).join(".git/beads-worktrees/beads-sync/.beads/issues.jsonl"),
    ];
    issue_paths.iter().any(|path| {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return false;
        };
        contents.lines().any(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|value| {
                    value
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(str::to_string)
                })
                .as_deref()
                == Some(issue_id)
        })
    })
}

fn focus_frame_legacy_migration_warnings(frame: &FrameRecord) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if frame
        .project_root
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
        || frame
            .continuity_id
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        warnings.push("focus_state_legacy_scope_inferred");
    }
    if frame.beads_issue_id.trim().is_empty()
        || frame.beads_issue_id.starts_with("synthetic")
        || frame.beads_issue_id.starts_with("pi-turn-")
    {
        warnings.push("focus_frame_missing_beads_source");
    }
    warnings
}

fn focus_frame_authority_posture(frame: &FrameRecord) -> serde_json::Value {
    let warnings = focus_frame_legacy_migration_warnings(frame);
    json!({
        "migration_class": "old_focus_state_records_and_focus_stack_frames",
        "read_behavior": if warnings.is_empty() { "reducer_backed_frame_scope" } else { "readable_history_noncanonical_when source Beads proof missing" },
        "authority_status": if warnings.is_empty() { "canonical_when_reducer_backed_frame_scope_matches" } else { "canonical_false_for_synthetic_or_missing_beads" },
        "migration_warnings": warnings,
        "scope": {
            "project_root": frame.project_root,
            "continuity_id": frame.continuity_id,
            "frame_id": frame.id,
            "scope_status": if warnings.is_empty() { "verified" } else { "partial" },
            "scope_source": if warnings.is_empty() { "frame_record" } else { "legacy_focus_frame" },
        },
        "promotion_path": ["focusa_project_verify", "focusa_workpoint_checkpoint", "focusa_current_focus"],
    })
}

fn focus_state_workpoint_bridge(
    state: &FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    focus_state_status: &'static str,
) -> Option<Value> {
    let project_root = clean_scope_value(project_root)?;
    if unsafe_project_root_reason(Some(project_root.as_str())).is_some() {
        return None;
    }
    let continuity_id = clean_scope_value(continuity_id)?;
    let record = state.workpoint.records.iter().rev().find(|record| {
        record.status == WorkpointStatus::Active
            && record.canonical
            && unsafe_project_root_reason(record.project_root.as_deref()).is_none()
            && record.project_root.as_deref().map(str::trim) == Some(project_root.as_str())
            && record.continuity_id.as_deref().map(str::trim) == Some(continuity_id.as_str())
    })?;
    Some(json!({
        "focus_state_status": focus_state_status,
        "workpoint_status": "canonical",
        "workpoint_id": record.workpoint_id,
        "authority_for_next_action": "workpoint",
        "resolution": "use canonical Workpoint for immediate next action; create or select a project-bound Focus frame before durable Focus State writes",
        "next_repair_tool": "focusa_workpoint_checkpoint",
        "supporting_context": "same project_root+continuity_id canonical Workpoint exists while Focus State frame is unavailable",
    }))
}

fn attach_focus_state_workpoint_bridge(response: &mut Value, bridge: Option<Value>) {
    let Some(bridge) = bridge else {
        return;
    };
    if let Some(obj) = response.as_object_mut() {
        obj.insert("focus_state_workpoint_bridge".to_string(), bridge.clone());
        obj.insert("workpoint_status".to_string(), json!("canonical"));
        obj.insert(
            "workpoint_id".to_string(),
            bridge.get("workpoint_id").cloned().unwrap_or(Value::Null),
        );
        obj.insert(
            "focus_state_status".to_string(),
            bridge
                .get("focus_state_status")
                .cloned()
                .unwrap_or(Value::Null),
        );
        obj.insert("authority_for_next_action".to_string(), json!("workpoint"));
        obj.insert(
            "next_tools".to_string(),
            json!([
                "focusa_workpoint_resume",
                "focusa_workpoint_checkpoint",
                "focusa_tool_doctor"
            ]),
        );
    }
}

fn resolve_scoped_frame<'a>(
    stack: &'a FocusStackState,
    frame_id: Option<Uuid>,
    session_key: Option<&str>,
    continuity_id: Option<&str>,
    project_root: Option<&str>,
) -> Option<(&'a FrameRecord, &'static str)> {
    if let Some(frame) = frame_id.and_then(|id| stack.frames.iter().find(|f| f.id == id)) {
        return frame_matches_project_root(frame, project_root).then_some((frame, "frame_id"));
    }

    let continuity = continuity_id.map(str::trim).unwrap_or_default();
    if !continuity.is_empty()
        && let Some(frame) = stack.frames.iter().rev().find(|frame| {
            frame.status == FrameStatus::Active
                && frame_matches_project_root(frame, project_root)
                && (frame.continuity_id.as_deref().map(str::trim) == Some(continuity)
                    || frame.tags.iter().any(|tag| {
                        tag == continuity || tag == &format!("continuity_id:{continuity}")
                    }))
        })
    {
        return Some((frame, "continuity_id"));
    }

    let key = session_key.map(str::trim).unwrap_or_default();
    if key.is_empty() {
        return None;
    }

    stack.frames.iter().rev().find_map(|frame| {
        (frame.status == FrameStatus::Active
            && frame_matches_project_root(frame, project_root)
            && frame.tags.iter().any(|tag| tag == key))
        .then_some((frame, "session_key"))
    })
}

/// Rebuild root, active, and path metadata after exact-scope frame filtering.
fn rebuild_scoped_stack_metadata(stack: &mut FocusStackState) {
    stack.active_id = stack
        .frames
        .iter()
        .rev()
        .find(|frame| frame.status == FrameStatus::Active)
        .map(|frame| frame.id);

    let mut path = Vec::new();
    let mut current = stack.active_id;
    while let Some(id) = current {
        if path.contains(&id) || path.len() >= stack.frames.len() {
            break;
        }
        let Some(frame) = stack.frames.iter().find(|frame| frame.id == id) else {
            break;
        };
        path.push(id);
        current = frame
            .parent_id
            .filter(|parent_id| stack.frames.iter().any(|frame| frame.id == *parent_id));
    }
    path.reverse();

    stack.root_id = path.first().copied().or_else(|| {
        stack
            .frames
            .iter()
            .find(|frame| {
                frame
                    .parent_id
                    .is_none_or(|parent_id| !stack.frames.iter().any(|item| item.id == parent_id))
            })
            .map(|frame| frame.id)
    });
    stack.stack_path_cache = path;
}

/// FS-01: scoped focus stack read.
async fn get_stack(
    scope: ScopeContext,
    Query(query): Query<FocusStackQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
    let mut scoped_stack = focusa.focus_stack.clone();
    scoped_stack
        .frames
        .retain(|frame| frame_matches_exact_request_scope(&scope, frame));
    rebuild_scoped_stack_metadata(&mut scoped_stack);
    let total = scoped_stack.frames.len();
    let default_limit = 25usize;
    let hard_limit = 200usize;
    let limit = query.limit.unwrap_or(default_limit).clamp(1, hard_limit);
    let cursor = query.cursor.unwrap_or(0).min(total);
    let frames_window = scoped_stack
        .frames
        .iter()
        .skip(cursor)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let next_cursor =
        (cursor + frames_window.len() < total).then(|| (cursor + frames_window.len()).to_string());
    Json(json!({
        "stack": &scoped_stack,
        "active_frame_id": scoped_stack.active_id,
        "frames_window": frames_window,
        "frames_authority_posture": scoped_stack.frames.iter().skip(cursor).take(limit).map(focus_frame_authority_posture).collect::<Vec<_>>(),
        "legacy_migration_policy": {
            "old_focus_state_records": "readable_with_safe_default_profile_and_frame_scope_when_available",
            "old_focus_stack_frames": "readable_history_noncanonical_when source Beads proof missing",
            "migration_warnings": ["focus_state_legacy_scope_inferred", "focus_frame_missing_beads_source"]
        },
        "traversal_metadata": {
            "surface": "focus_stack",
            "selector": "window",
            "total": total,
            "returned": frames_window.len(),
            "limit": limit,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "summary_only": true,
            "cold_full_payload_opt_in": false,
        },
        "rehydrate_refs": [
            {"route":"/v1/focus/frame/current", "reason":"active project-bound frame"},
            {"tool":"focusa_traverse", "surface":"focus_stack", "selector":"window"}
        ]
    }))
}

/// FS-01: scoped focus frame read.
async fn get_scoped_frame(
    scope: ScopeContext,
    Query(query): Query<ScopedFrameQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if query.frame_id.is_none()
        && query.session_key.is_none()
        && query.continuity_id.is_none()
        && query.project_root.is_none()
    {
        return Json(json!({
            "status": "rejected",
            "canonical": false,
            "failure_class": "scope_required",
            "reason": "frame_current_requires_project_scope",
            "retry_posture": "do_not_retry_unchanged",
            "safe_recovery": "call /v1/focus/frame/current with project_root+continuity_id or explicit frame_id",
        }));
    }

    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
    let resolved = resolve_scoped_frame(
        &focusa.focus_stack,
        query.frame_id,
        query.session_key.as_deref(),
        query.continuity_id.as_deref(),
        query.project_root.as_deref(),
    );
    if let Some((frame, _)) = resolved
        && !frame_matches_exact_request_scope(&scope, frame)
    {
        return Json(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "frame": null,
        }));
    }
    Json(json!({
        "frame": resolved.map(|(frame, _)| frame),
        "matched_by": resolved.map(|(_, matched_by)| matched_by).unwrap_or("none"),
        "authority_posture": resolved.map(|(frame, _)| focus_frame_authority_posture(frame)),
        "migration_warnings": resolved.map(|(frame, _)| focus_frame_legacy_migration_warnings(frame)).unwrap_or_else(|| vec!["focus_state_legacy_scope_inferred"]),
        "requested_project_root": query.project_root,
    }))
}

fn ensure_active_session(session: Option<&SessionState>) -> Result<(), serde_json::Value> {
    match session {
        Some(session) if session.status == SessionStatus::Active => Ok(()),
        Some(session) => Err(json!({
            "status": "rejected",
            "reason": "session_inactive",
            "session_status": session.status,
        })),
        None => Err(json!({
            "status": "rejected",
            "reason": "no_active_session",
        })),
    }
}

fn validate_can_pop(stack: &FocusStackState) -> Result<(), serde_json::Value> {
    let active_id = match stack.active_id {
        Some(id) => id,
        None => return Err(json!({"status": "no_active_frame"})),
    };

    let active = stack
        .frames
        .iter()
        .find(|f| f.id == active_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "active_frame_missing"}))?;

    let parent_id = active
        .parent_id
        .ok_or_else(|| json!({"status": "rejected", "reason": "cannot_complete_root_frame"}))?;

    let parent = stack
        .frames
        .iter()
        .find(|f| f.id == parent_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "parent_frame_missing"}))?;

    if parent.status != FrameStatus::Paused {
        return Err(json!({
            "status": "rejected",
            "reason": "parent_not_paused",
            "parent_status": parent.status,
        }));
    }

    Ok(())
}

fn validate_set_active(stack: &FocusStackState, frame_id: Uuid) -> Result<bool, serde_json::Value> {
    let active_id = match stack.active_id {
        Some(id) => id,
        None => return Err(json!({"status": "no_active_frame"})),
    };

    if active_id == frame_id {
        return Ok(false);
    }

    if !stack.stack_path_cache.contains(&frame_id) {
        return Err(json!({
            "status": "rejected",
            "reason": "target_not_in_current_stack_path",
        }));
    }

    let target = stack
        .frames
        .iter()
        .find(|f| f.id == frame_id)
        .ok_or_else(|| json!({"status": "rejected", "reason": "frame_not_found"}))?;

    if target.status != FrameStatus::Paused {
        return Err(json!({
            "status": "rejected",
            "reason": "target_not_paused",
            "frame_status": target.status,
        }));
    }

    Ok(true)
}

async fn materialize_focus_event(
    state: &AppState,
    event: FocusaEvent,
    correlation_id: &'static str,
) -> FocusUnitResult {
    let _guard = state.write_serial_lock.lock().await;
    let event_scope = focusa_core::scoped_state::workstream_scope_of_event(&event);
    let current = match &event_scope {
        Some((root, continuity)) => state
            .workstream_states
            .get_or_create(root, continuity)
            .await
            .read()
            .await
            .clone(),
        None => { state.focusa.read().await.clone() }
    };
    let result = reducer::reduce_with_meta(current, event, None, None, false).map_err(|error| {
        tracing::warn!(error = %error, correlation_id, "focus event rejected by reducer");
        focus_reducer_failed(error)
    })?;

    let new_state = result.new_state;
    for emitted in result.emitted_events {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event: emitted,
            correlation_id: Some(correlation_id.to_string()),
            origin: SignalOrigin::Adapter,
            machine_id: None,
            instance_id: None,
            session_id: new_state.session.as_ref().map(|session| session.session_id),
            thread_id: None,
            is_observation: false,
        };
        if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
            tracing::error!(error = %error, correlation_id, "failed to persist focus event");
            return Err(focus_persistence_failed(error));
        } else if let Ok(serialized) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(serialized);
        }
    }

    *state.focusa.write().await = new_state;
    state.mark_external_mutation();
    Ok(())
}

#[derive(Deserialize)]
struct PushFrameBody {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    beads_issue_id: Option<String>,
    #[serde(default)]
    constraints: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
}

/// FS-01: scoped focus stack push.
async fn push_frame(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PushFrameBody>,
) -> FocusResult {
    {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        if let Err(resp) = ensure_active_session(focusa.session.as_ref()) {
            return Ok(Json(resp));
        }
    }

    let Some(project_root) = clean_scope_value(body.project_root.as_deref()) else {
        return Ok(Json(unsafe_project_root_response(
            "missing_project_root",
            body.project_root.as_deref(),
        )));
    };
    if let Some(reason) = unsafe_project_root_reason(Some(project_root.as_str())) {
        return Ok(Json(unsafe_project_root_response(
            reason,
            Some(project_root.as_str()),
        )));
    }
    let Some(continuity_id) = clean_scope_value(body.continuity_id.as_deref()) else {
        return Ok(Json(json!({
            "status": "rejected_missing_scope",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "missing": "continuity_id",
            "retry_posture": "do_not_retry_unchanged",
            "safe_recovery": "pass continuity_id so the Focus frame is bound to a project workstream",
            "human_message": "Missing continuity_id. Run: bd ready → select a bead → pass its continuity_id",
            "next_tools": ["focusa_project_identity", "focusa_workpoint_resume"],
        })));
    };
    if let Err(response) = exact_request_scope_matches(&scope, &project_root, &continuity_id) {
        return Ok(Json(response));
    }

    let beads_issue_id = body.beads_issue_id.unwrap_or_default();
    let beads_issue_id = beads_issue_id.trim().to_string();
    if beads_issue_id.is_empty() {
        return Ok(Json(json!({
            "status": "rejected",
            "canonical": false,
            "failure_class": "validation_rejected",
            "reason": "missing_beads_issue_id",
            "safe_recovery": "pass a real Beads issue id from the project .beads workspace before creating a canonical Focus frame",
            "human_message": "Missing beads_issue_id. Run: bd ready → copy the bead ID → pass it as beads_issue_id",
            "next_tools": ["bd"],
        })));
    }
    if !beads_issue_exists(&project_root, &beads_issue_id) {
        return Ok(Json(json!({
            "status": "rejected",
            "canonical": false,
            "failure_class": "validation_rejected",
            "reason": "beads_issue_not_found",
            "beads_issue_id": beads_issue_id,
            "project_root": project_root,
            "safe_recovery": "create or select a real Beads issue in this project, or keep proposal/demo frames noncanonical outside FocusFramePushed",
            "retry_posture": "do_not_retry_unchanged",
            "human_message": format!("Bead '{}' not found in {}. Run: bd show {} → verify it exists, or bd create to create one", beads_issue_id, project_root, beads_issue_id),
            "next_tools": ["bd"],
        })));
    }

    let frame_id = Uuid::now_v7();
    materialize_focus_event(
        &state,
        FocusaEvent::FocusFramePushed {
            frame_id,
            title: body.title.unwrap_or_default(),
            goal: body.goal.unwrap_or_default(),
            beads_issue_id,
            project_root: Some(project_root),
            continuity_id: Some(continuity_id),
            constraints: body.constraints,
            tags: body.tags,
        },
        "api:focus:push",
    )
    .await?;

    Ok(Json(
        json!({"status": "accepted", "frame_id": frame_id, "materialized_by": "api_reducer_sync"}),
    ))
}

#[derive(Deserialize)]
struct PopFrameBody {
    #[serde(default = "default_completion_reason")]
    completion_reason: CompletionReason,
}

fn default_completion_reason() -> CompletionReason {
    CompletionReason::GoalAchieved
}

/// FS-01: scoped focus stack pop.
async fn pop_frame(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PopFrameBody>,
) -> FocusResult {
    {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        if let Err(resp) = ensure_active_session(focusa.session.as_ref()) {
            return Ok(Json(resp));
        }
        let Some(active_frame) = focusa.focus_stack.active_id.and_then(|id| {
            focusa
                .focus_stack
                .frames
                .iter()
                .find(|frame| frame.id == id)
        }) else {
            return Ok(Json(json!({"status": "no_active_frame"})));
        };
        if !frame_matches_exact_request_scope(&scope, active_frame) {
            return Ok(Json(json!({
                "status": "scope_mismatch",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "reason": "request_scope_does_not_match_active_frame",
            })));
        }
        if let Err(resp) = validate_can_pop(&focusa.focus_stack) {
            return Ok(Json(resp));
        }
    }

    let frame_id = state
        .focusa
        .read()
        .await
        .focus_stack
        .active_id
        .ok_or_else(focus_active_frame_missing)?;
    materialize_focus_event(
        &state,
        FocusaEvent::FocusFrameCompleted {
            frame_id,
            completion_reason: body.completion_reason,
        },
        "api:focus:pop",
    )
    .await?;

    Ok(Json(
        json!({"status": "accepted", "frame_id": frame_id, "materialized_by": "api_reducer_sync"}),
    ))
}

#[derive(Deserialize)]
struct SetActiveBody {
    frame_id: uuid::Uuid,
}

/// FS-01: scoped focus stack set-active.
async fn set_active(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetActiveBody>,
) -> FocusResult {
    {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        if let Err(resp) = ensure_active_session(focusa.session.as_ref()) {
            return Ok(Json(resp));
        }
        let Some(target_frame) = focusa
            .focus_stack
            .frames
            .iter()
            .find(|frame| frame.id == body.frame_id)
        else {
            return Ok(Json(
                json!({"status": "rejected", "reason": "frame_not_found"}),
            ));
        };
        if !frame_matches_exact_request_scope(&scope, target_frame) {
            return Ok(Json(json!({
                "status": "scope_mismatch",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "reason": "request_scope_does_not_match_target_frame",
            })));
        }
        match validate_set_active(&focusa.focus_stack, body.frame_id) {
            Ok(true) => {}
            Ok(false) => return Ok(Json(json!({"status": "accepted", "noop": true}))),
            Err(resp) => return Ok(Json(resp)),
        }
    }

    materialize_focus_event(
        &state,
        FocusaEvent::FocusFrameResumed {
            frame_id: body.frame_id,
        },
        "api:focus:set-active",
    )
    .await?;

    Ok(Json(
        json!({"status": "accepted", "frame_id": body.frame_id, "materialized_by": "api_reducer_sync"}),
    ))
}

/// POST /v1/focus/update — update focus state delta (ASCC).
///
/// Per spec: adapters provide transcript summaries to ASCC.
/// §AsccSections §Validation: validates ALL slots at API boundary before any write.
#[derive(Deserialize)]
struct UpdateDeltaBody {
    #[serde(default)]
    frame_id: Option<Uuid>,
    #[serde(default)]
    turn_id: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    delta: FocusStateDelta,
}

/// Validate a single slot value. Rejects verbose output, task patterns,
/// self-reference, markdown noise — same rules as tools.ts validateSlot.
/// Slot-specific stricter rules for result-adjacent slots.
fn validate_slot(value: &str, max_chars: usize, slot_kind: &str) -> bool {
    if value.is_empty() || value.len() > max_chars {
        return false;
    }
    let lower = value.to_lowercase();

    // Slot-specific: reject verbose process narration in result/question slots
    if matches!(slot_kind, "recent_results" | "notes" | "open_questions") && value.len() > 180 {
        return false; // verbose entries don't belong in result/question slots
    }

    // Verbose process narration patterns — NEVER valid in any slot
    if lower.contains("root cause")
        || lower.contains("bypass")
        || lower.contains("pollut")
        || lower.contains("investigation")
        || lower.contains("pattern ") && lower.contains("match")
        || lower.contains("verbose") && lower.len() < 100
        || lower.contains("i was able to")
        || lower.contains("it appears that")
        || lower.contains("this confirms")
        || lower.contains("as suspected")
        || lower.contains("confirmed in the running system")
        || lower.contains("still the old version")
        || lower.contains("three bugs")
        || lower.contains("daemon restarted")
        || lower.contains("binary confirmed")
    {
        return false;
    }

    // Table/structured markup — investigation noise, not results
    if value.contains("| ") && value.contains(" | ") && value.contains(" : ") {
        return false;
    }

    // Task patterns
    if lower.contains("implement ")
        || lower.contains(" add ")
        || lower.contains("create ")
        || lower.contains("update ")
        || lower.contains("remove ")
        || lower.contains("fix all")
        || lower.contains("next:")
        || lower.contains("signal:")
    {
        return false;
    }
    // Self-reference
    if lower.contains("i think")
        || lower.contains("i tried")
        || lower.contains("i'm working")
        || lower.contains("i was")
        || lower.contains("in this session")
        || lower.contains("while i was")
        || lower.contains("my fs.")
        || lower.contains("my fix")
        || lower.contains("let me")
        || lower.contains("i need to")
        || lower.contains("i will")
        || lower.contains("i'll need")
    {
        return false;
    }
    // Markdown / noise patterns
    if value.contains("**")
        || value.contains("\u{2705}")
        || value.contains("\u{274C}")
        || value.contains("- [ ]")
        || value.contains("---")
        || value.contains("```")
        || value.contains("|")
        || value.starts_with("2.")
        || value.starts_with("3.")
        || value.starts_with("- ")
        || lower.contains("spec-compliant")
        || lower.contains("matches")
        || lower.contains("exactly")
        || lower.contains("fixme")
        || value.starts_with("Modified:")
        || value.starts_with("Added:")
        || value.starts_with("Deleted:")
    {
        return false;
    }
    // Verbose continuation
    if lower.contains("now") && lower.contains("need to") {
        return false;
    }
    if lower.contains("continue") && value.len() > 80 {
        return false;
    }
    true
}

/// Slot capacity caps per §AsccSections.
fn slot_cap(slot_kind: &str) -> usize {
    match slot_kind {
        "decisions" | "next_steps" | "recent_results" => 10,
        "open_questions" | "notes" => 20,
        "constraints" => 15,
        "failures" => 10,
        _ => 50,
    }
}

/// FS-01: scoped focus state mutation.
async fn update_delta(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateDeltaBody>,
) -> FocusResult {
    // §AsccSections: validate ALL slots at API boundary before any write.
    let delta = &body.delta;
    let frame = {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        if let Some(target_frame_id) = body.frame_id {
            focusa
                .focus_stack
                .frames
                .iter()
                .find(|f| f.id == target_frame_id)
                .cloned()
        } else {
            focusa
                .focus_stack
                .frames
                .iter()
                .find(|f| Some(f.id) == focusa.focus_stack.active_id)
                .cloned()
        }
    };

    if let Some(target_frame_id) = body.frame_id {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        let active_frame_id = focusa.focus_stack.active_id;
        let target = focusa
            .focus_stack
            .frames
            .iter()
            .find(|frame| frame.id == target_frame_id);
        if let Some(frame) = target
            && let Some(reason) = unsafe_project_root_reason(frame.project_root.as_deref())
        {
            return Ok(Json(unsafe_project_root_response(
                reason,
                frame.project_root.as_deref(),
            )));
        }
        if target.is_none() {
            let bridge = focus_state_workpoint_bridge(
                &focusa,
                body.project_root.as_deref(),
                body.continuity_id.as_deref(),
                "target_frame_unavailable",
            );
            let mut response = json!({
                "status": "frame_unavailable",
                "failure_class": "frame_unavailable",
                "reason": "target_frame_id_not_found",
                "target_frame_id": target_frame_id,
                "active_frame_id": active_frame_id,
                "retry_posture": "safe_retry",
                "retry": {"safe": true, "posture": "safe_retry", "reason": "frame_unavailable"},
                "safe_recovery": "call /v1/focus/frame/current with continuity_id or checkpoint a fresh Workpoint/Focus frame",
                "next_tools": ["focusa_workpoint_resume", "focusa_workpoint_checkpoint", "focusa_tool_doctor"],
                "details": {"tool_result_v1": {"ok": false, "status": "blocked", "failure_class": "frame_unavailable", "canonical": false, "degraded": true, "retry": {"safe": true, "posture": "safe_retry", "reason": "frame_unavailable"}, "next_tools": ["focusa_workpoint_resume", "focusa_workpoint_checkpoint", "focusa_tool_doctor"]}}
            });
            attach_focus_state_workpoint_bridge(&mut response, bridge);
            return Ok(Json(response));
        }
    }

    // Per-slot validation with kind + capacity cap check
    if let Some(ref intent) = delta.intent
        && !validate_slot(intent, 500, "intent")
    {
        return Ok(Json(
            json!({"status": "rejected", "reason": "intent: validation failed"}),
        ));
    }
    if let Some(ref cs) = delta.current_state
        && !validate_slot(cs, 300, "current_state")
    {
        return Ok(Json(
            json!({"status": "rejected", "reason": "current_state: validation failed"}),
        ));
    }
    for (kind, values, max_chars) in [
        ("decisions", &delta.decisions, 160),
        ("constraints", &delta.constraints, 200),
        ("failures", &delta.failures, 300),
        ("next_steps", &delta.next_steps, 160),
        ("recent_results", &delta.recent_results, 300),
        ("notes", &delta.notes, 200),
        ("open_questions", &delta.open_questions, 200),
    ] {
        if let Some(vals) = values {
            if let Some(ref f) = frame {
                let current_len = match kind {
                    "decisions" => f.focus_state.decisions.len(),
                    "constraints" => f.focus_state.constraints.len(),
                    "failures" => f.focus_state.failures.len(),
                    "next_steps" => f.focus_state.next_steps.len(),
                    "recent_results" => f.focus_state.recent_results.len(),
                    "notes" => f.focus_state.notes.len(),
                    "open_questions" => f.focus_state.open_questions.len(),
                    _ => 0,
                };
                if current_len >= slot_cap(kind) {
                    return Ok(Json(
                        json!({"status": "rejected", "reason": format!("{}: at capacity ({})", kind, current_len)}),
                    ));
                }
            }
            if vals.iter().any(|s| !validate_slot(s, max_chars, kind)) {
                return Ok(Json(
                    json!({"status": "rejected", "reason": format!("{}: validation failed", kind)}),
                ));
            }
        }
    }
    // Validate artifacts
    if let Some(ref artifacts) = delta.artifacts {
        for a in artifacts {
            if a.label.is_empty() || a.label.len() > 100 {
                return Ok(Json(
                    json!({"status": "rejected", "reason": "artifacts: label validation failed"}),
                ));
            }
        }
    }

    // Prefer explicit frame_id; otherwise resolve by ProjectRootKey + WorkstreamKey.
    // Never adopt the daemon-global active frame as canonical Focus State write authority.
    let (fid, auto_started_session) = {
        let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &scope).await;
        let session_active = focusa
            .session
            .as_ref()
            .map(|session| session.status == SessionStatus::Active)
            .unwrap_or(false);
        if let Some(frame_id) = body.frame_id {
            let Some(frame) = focusa
                .focus_stack
                .frames
                .iter()
                .find(|frame| frame.id == frame_id)
            else {
                let bridge = focus_state_workpoint_bridge(
                    &focusa,
                    body.project_root.as_deref(),
                    body.continuity_id.as_deref(),
                    "target_frame_unavailable",
                );
                let mut response = json!({
                    "status": "no_active_frame",
                    "canonical": false,
                    "failure_class": "frame_unavailable",
                    "reason": "target_frame_id_not_found",
                    "safe_recovery": "call /v1/focus/frame/current with project_root+continuity_id or pass explicit frame_id"
                });
                attach_focus_state_workpoint_bridge(&mut response, bridge);
                return Ok(Json(response));
            };
            if let Some(reason) = unsafe_project_root_reason(frame.project_root.as_deref()) {
                return Ok(Json(unsafe_project_root_response(
                    reason,
                    frame.project_root.as_deref(),
                )));
            }
            if clean_scope_value(frame.continuity_id.as_deref()).is_none() {
                return Ok(Json(json!({
                    "status": "rejected_missing_scope",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "missing": "frame.continuity_id",
                    "retry_posture": "do_not_retry_unchanged",
                    "safe_recovery": "checkpoint or create a project-workstream scoped Focus frame before writing Focus State"
                })));
            }
            if !frame_matches_exact_request_scope(&scope, frame) {
                return Ok(Json(json!({
                    "status": "scope_mismatch",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "reason": "request_scope_does_not_match_frame",
                })));
            }
            if let Some(expected_project_root) = clean_scope_value(body.project_root.as_deref())
                && frame
                    .project_root
                    .as_deref()
                    .map(normalize_project_root_authority)
                    .as_deref()
                    != Some(expected_project_root.as_str())
            {
                return Ok(Json(
                    json!({"status":"scope_mismatch", "canonical": false, "failure_class":"scope_mismatch", "field":"project_root"}),
                ));
            }
            if let Some(expected_continuity_id) = clean_scope_value(body.continuity_id.as_deref())
                && frame.continuity_id.as_deref().map(str::trim)
                    != Some(expected_continuity_id.as_str())
            {
                return Ok(Json(
                    json!({"status":"scope_mismatch", "canonical": false, "failure_class":"scope_mismatch", "field":"continuity_id"}),
                ));
            }
            (frame_id, !session_active)
        } else {
            let resolved = resolve_scoped_frame(
                &focusa.focus_stack,
                None,
                None,
                body.continuity_id.as_deref(),
                body.project_root.as_deref(),
            );
            let Some((frame, _)) = resolved else {
                let bridge = focus_state_workpoint_bridge(
                    &focusa,
                    body.project_root.as_deref(),
                    body.continuity_id.as_deref(),
                    "missing_project_bound_frame",
                );
                let mut response = json!({
                    "status": "no_active_frame",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "reason": "focus_update_requires_frame_id_or_project_root_plus_continuity_id",
                    "retry_posture": "do_not_retry_unchanged",
                    "safe_recovery": "call /v1/focus/frame/current with project_root+continuity_id or pass explicit frame_id"
                });
                attach_focus_state_workpoint_bridge(&mut response, bridge);
                return Ok(Json(response));
            };
            if !frame_matches_exact_request_scope(&scope, frame) {
                return Ok(Json(json!({
                    "status": "scope_mismatch",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "reason": "request_scope_does_not_match_frame",
                })));
            }
            (frame.id, !session_active)
        }
    };

    if auto_started_session {
        materialize_focus_event(
            &state,
            FocusaEvent::SessionStarted {
                session_id: Uuid::now_v7(),
                adapter_id: Some("focus-update".to_string()),
                workspace_id: Some("auto-recovered-focus-write".to_string()),
                project_root: None,
                continuity_id: None,
            },
            "api:focus:update:auto-session",
        )
        .await?;
    }

    let turn_id = body.turn_id.unwrap_or_else(|| Uuid::now_v7().to_string());
    materialize_focus_event(
        &state,
        FocusaEvent::FocusStateUpdated {
            frame_id: fid,
            delta: body.delta,
        },
        "api:focus:update",
    )
    .await?;

    Ok(Json(json!({
        "status": "accepted",
        "frame_id": fid,
        "turn_id": turn_id,
        "auto_started_session": auto_started_session,
        "materialized_by": "api_reducer_sync"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════════
// PI-TOGGLE ENDPOINT — SPEC-33.5 disk persistence
// ═══════════════════════════════════════════════════════════════════════════════

/// Path to the Pi toggle state file.
fn pi_enabled_path(config: &focusa_core::types::FocusaConfig) -> std::path::PathBuf {
    let expanded = if config.data_dir.starts_with("~") {
        std::env::var("HOME").unwrap_or_else(|_| "~".to_string()) + &config.data_dir[1..]
    } else {
        config.data_dir.clone()
    };
    std::path::PathBuf::from(expanded).join("pi-enabled")
}

/// GET /v1/focusa/enabled — read current toggle state.
async fn get_enabled(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let path = pi_enabled_path(&state.config);
    let enabled = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().strip_prefix("enabled=").map(|v| v == "1"))
            .unwrap_or(true) // default: enabled
    } else {
        true
    };
    Json(json!({"enabled": enabled}))
}

#[derive(Deserialize)]
struct SetEnabledBody {
    enabled: bool,
}

/// PATCH /v1/focusa/enabled — set toggle state.
async fn set_enabled(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetEnabledBody>,
) -> FocusResult {
    let path = pi_enabled_path(&state.config);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| focus_toggle_io_failed("create parent directory", parent, error))?;
    }
    let content = format!("enabled={}", if body.enabled { "1" } else { "0" });
    std::fs::write(&path, content)
        .map_err(|error| focus_toggle_io_failed("write", &path, error))?;
    tracing::info!(
        path = path.display().to_string(),
        enabled = body.enabled,
        "Pi focusa toggle updated"
    );
    Ok(Json(json!({"status": "updated", "enabled": body.enabled})))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/focus/stack", get(get_stack))
        .route("/v1/focus/frame/current", get(get_scoped_frame))
        .route("/v1/focus/push", post(push_frame))
        .route("/v1/focus/pop", post(pop_frame))
        .route("/v1/focus/set-active", post(set_active))
        .route("/v1/focus/update", post(update_delta))
        .route("/v1/focusa/enabled", get(get_enabled))
        .route("/v1/focusa/enabled", patch(set_enabled))
}

#[cfg(test)]
mod tests {
    use super::{exact_request_scope_matches, rebuild_scoped_stack_metadata, resolve_scoped_frame};
    use crate::scope::ScopeContext;
    use chrono::Utc;
    use focusa_core::types::{
        CompletionReason, FocusStackState, FocusState, FrameRecord, FrameStats, FrameStatus,
    };
    use uuid::Uuid;

    #[test]
    fn exact_request_scope_rejects_host_and_cross_workstream_mutation() {
        let exact = ScopeContext {
            project_root: Some("/home/wirebot/focusa".to_string()),
            continuity_id: Some("cont-focusa".to_string()),
            ..Default::default()
        };
        let host = ScopeContext {
            project_root: Some("/root".to_string()),
            continuity_id: Some("cont-focusa".to_string()),
            ..Default::default()
        };
        let other = ScopeContext {
            project_root: Some("/home/wirebot/focusa".to_string()),
            continuity_id: Some("cont-other".to_string()),
            ..Default::default()
        };

        assert!(exact_request_scope_matches(&exact, "/home/wirebot/focusa", "cont-focusa").is_ok());
        assert!(exact_request_scope_matches(&host, "/home/wirebot/focusa", "cont-focusa").is_err());
        assert!(
            exact_request_scope_matches(&other, "/home/wirebot/focusa", "cont-focusa").is_err()
        );
    }

    fn frame(id: Uuid, status: FrameStatus, title: &str, tags: &[&str]) -> FrameRecord {
        FrameRecord {
            id,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status,
            title: title.to_string(),
            goal: title.to_string(),
            beads_issue_id: format!("issue-{title}"),
            project_root: Some("/repo/test".to_string()),
            continuity_id: tags
                .iter()
                .find(|tag| tag.starts_with("cont-"))
                .map(|tag| tag.to_string()),
            tags: tags.iter().map(|x| x.to_string()).collect(),
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: FocusState::default(),
            completed_at: None,
            completion_reason: None::<CompletionReason>,
        }
    }

    #[test]
    fn scoped_stack_metadata_is_derived_from_filtered_frames() {
        let external_parent = Uuid::now_v7();
        let root_id = Uuid::now_v7();
        let active_id = Uuid::now_v7();
        let mut root = frame(root_id, FrameStatus::Paused, "root", &["cont-focusa"]);
        root.parent_id = Some(external_parent);
        let mut active = frame(active_id, FrameStatus::Active, "active", &["cont-focusa"]);
        active.parent_id = Some(root_id);
        let mut stack = FocusStackState {
            root_id: Some(external_parent),
            active_id: Some(external_parent),
            frames: vec![root, active],
            stack_path_cache: vec![external_parent],
            ..FocusStackState::default()
        };

        rebuild_scoped_stack_metadata(&mut stack);

        assert_eq!(stack.root_id, Some(root_id));
        assert_eq!(stack.active_id, Some(active_id));
        assert_eq!(stack.stack_path_cache, vec![root_id, active_id]);
    }

    #[test]
    fn resolve_scoped_frame_requires_scope() {
        let active_id = Uuid::now_v7();
        let mut unsafe_frame = frame(active_id, FrameStatus::Active, "unsafe", &["cont-root"]);
        unsafe_frame.project_root = Some("/root".to_string());
        let stack = FocusStackState {
            root_id: Some(active_id),
            active_id: Some(active_id),
            frames: vec![unsafe_frame],
            stack_path_cache: vec![active_id],
            version: 1,
        };
        assert!(
            resolve_scoped_frame(&stack, None, None, None, Some("/workspace/focusa")).is_none()
        );
        assert!(resolve_scoped_frame(&stack, None, None, None, None).is_none());
    }

    #[test]
    fn resolve_scoped_frame_matches_safe_project_and_continuity() {
        let safe_id = Uuid::now_v7();
        let mut safe_frame = frame(safe_id, FrameStatus::Active, "safe", &["cont-focusa"]);
        safe_frame.project_root = Some("/workspace/focusa".to_string());
        let stack = FocusStackState {
            root_id: Some(safe_id),
            active_id: Some(safe_id),
            frames: vec![safe_frame],
            stack_path_cache: vec![safe_id],
            version: 1,
        };
        let resolved = resolve_scoped_frame(
            &stack,
            None,
            None,
            Some("cont-focusa"),
            Some("/workspace/focusa"),
        );
        assert_eq!(
            resolved.map(|(frame, by)| (frame.id, by)),
            Some((safe_id, "continuity_id"))
        );
    }

    #[test]
    fn resolve_scoped_frame_prefers_explicit_frame_id() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let stack = FocusStackState {
            root_id: Some(a),
            active_id: Some(b),
            frames: vec![
                frame(a, FrameStatus::Paused, "paused", &["pi-1"]),
                frame(b, FrameStatus::Active, "active", &["pi-1"]),
            ],
            stack_path_cache: vec![a, b],
            version: 2,
        };

        let (resolved, matched_by) =
            resolve_scoped_frame(&stack, Some(a), Some("pi-1"), None, None).expect("frame");
        assert_eq!(resolved.id, a);
        assert_eq!(matched_by, "frame_id");
    }

    #[test]
    fn resolve_scoped_frame_without_scope_is_none() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let stack = FocusStackState {
            root_id: Some(a),
            active_id: Some(b),
            frames: vec![
                frame(a, FrameStatus::Paused, "paused", &["pi-1"]),
                frame(b, FrameStatus::Active, "active", &["pi-2"]),
            ],
            stack_path_cache: vec![a, b],
            version: 2,
        };

        assert!(resolve_scoped_frame(&stack, None, None, None, None).is_none());
    }

    #[test]
    fn resolve_scoped_frame_falls_back_to_latest_active_session_key() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        let c = Uuid::now_v7();
        let stack = FocusStackState {
            root_id: Some(a),
            active_id: Some(c),
            frames: vec![
                frame(a, FrameStatus::Paused, "old", &["pi-1"]),
                frame(b, FrameStatus::Active, "other", &["pi-2"]),
                frame(c, FrameStatus::Active, "current", &["pi-1"]),
            ],
            stack_path_cache: vec![a, c],
            version: 3,
        };

        let (resolved, matched_by) =
            resolve_scoped_frame(&stack, None, Some("pi-1"), None, None).expect("frame");
        assert_eq!(resolved.id, c);
        assert_eq!(matched_by, "session_key");
    }
}

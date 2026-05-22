//! Spec96 per-project Trajectory Intelligence API.
//!
//! Trajectory is a bounded, read-only projection over existing Focusa
//! primitives. It orients agents per project; it does not select work, mutate
//! Focus State, switch frames, or execute actions.

use crate::routes::project::project_identity_payload_for_scope;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::reducer;
use focusa_core::types::{
    EventLogEntry, FocusState, FocusaEvent, FocusaSessionIdentity, FocusaState, FrameRecord,
    SignalOrigin, TrajectoryConfidence, TrajectoryDefinitionOfDoneRecord,
    TrajectoryDefinitionStatus, TrajectoryGoalProvenanceRecord, TrajectoryMilestoneRecord,
    TrajectoryMilestoneStatus, TrajectoryProjectionRecord, WorkpointRecord, WorkpointStatus,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryViewQuery {
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryDefineGoalRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub long_term_goal: String,
    pub desired_end_state: String,
    pub short_term_goal: Option<String>,
    pub current_state: Option<String>,
    pub goal_source: Option<String>,
    pub supersedes_trajectory_id: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub operator_confirmed: Option<bool>,
    pub supersession_evidence_refs: Option<Vec<String>>,
    pub required_evidence_refs: Option<Vec<String>>,
    pub required_checks: Option<Vec<String>>,
    pub acceptance_risks: Option<Vec<String>>,
    pub not_done_if: Option<Vec<String>>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryAssessRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub observed_state: Option<String>,
    pub evidence_refs: Option<Vec<String>>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryProposeWorkpointRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub trajectory_id: Option<String>,
    pub target_ref: Option<String>,
    pub action_type: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryCheckpointRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub summary: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryResumeRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub mode: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn session_identity_project_root(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| {
        clean(Some(identity.project_root.as_str())).or_else(|| {
            identity
                .project_identity
                .as_ref()
                .and_then(|project| clean(Some(project.project_root.as_str())))
        })
    })
}

fn session_identity_session_id(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| {
        clean(identity.pi_session_id.as_deref())
            .or_else(|| clean(Some(identity.session_frame_key.as_str())))
    })
}

fn session_identity_continuity_id(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| clean(identity.continuity_id.as_deref()))
}

fn scoped_query_from_identity(
    project_root: Option<&str>,
    session_id: Option<&str>,
    continuity_id: Option<&str>,
    mode: Option<&str>,
    session_identity: Option<&FocusaSessionIdentity>,
) -> TrajectoryViewQuery {
    let identity_project_root = session_identity_project_root(session_identity);
    let identity_session_id = session_identity_session_id(session_identity);
    let identity_continuity_id = session_identity_continuity_id(session_identity);
    query_from_scope(
        identity_project_root.as_deref().or(project_root),
        identity_session_id.as_deref().or(session_id),
        identity_continuity_id.as_deref().or(continuity_id),
        mode,
    )
}

fn bounded(value: &str, max: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let mut out = trimmed
        .chars()
        .take(max.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

fn active_frame(state: &FocusaState) -> Option<&FrameRecord> {
    state
        .focus_stack
        .active_id
        .and_then(|id| state.focus_stack.frames.iter().find(|frame| frame.id == id))
}

fn scoped_active_frame<'a>(
    state: &'a FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a FrameRecord> {
    let frame = active_frame(state)?;
    if project_root.is_some() || continuity_id.is_some() {
        let project_matches = project_root
            .map(|expected| clean(frame.project_root.as_deref()).as_deref() == Some(expected))
            .unwrap_or(true);
        let continuity_matches = continuity_id
            .map(|expected| clean(frame.continuity_id.as_deref()).as_deref() == Some(expected))
            .unwrap_or(true);
        return (project_matches && continuity_matches).then_some(frame);
    }
    Some(frame)
}

fn active_workpoint(state: &FocusaState) -> Option<&WorkpointRecord> {
    state.workpoint.active_workpoint_id.and_then(|id| {
        state
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    })
}

fn scoped_active_workpoint<'a>(
    state: &'a FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a WorkpointRecord> {
    let scope_requested = project_root.is_some() || continuity_id.is_some();
    if scope_requested {
        return state.workpoint.records.iter().rev().find(|record| {
            record.status == WorkpointStatus::Active
                && record.canonical
                && project_root
                    .map(|expected| {
                        clean(record.project_root.as_deref()).as_deref() == Some(expected)
                    })
                    .unwrap_or(true)
                && continuity_id
                    .map(|expected| {
                        clean(record.continuity_id.as_deref()).as_deref() == Some(expected)
                    })
                    .unwrap_or(true)
        });
    }
    active_workpoint(state)
}

fn first_nonempty(candidates: &[Option<&str>]) -> Option<String> {
    candidates.iter().find_map(|candidate| clean(*candidate))
}

fn top_strings(values: &[String], limit: usize, max_chars: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| bounded(value, max_chars))
        .filter(|value| !value.is_empty())
        .take(limit)
        .collect()
}

fn stable_project_fingerprint(project_root: &str, session_id: Option<&str>) -> String {
    // FNV-1a 64-bit: deterministic, dependency-free, good enough for a
    // projection fingerprint. This is not a security boundary.
    let mut hash: u64 = 0xcbf29ce484222325;
    let input = format!(
        "{}|{}",
        project_root.trim(),
        session_id.unwrap_or_default().trim()
    );
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("project-fnv1a64:{hash:016x}")
}

fn focus_state_for(frame: Option<&FrameRecord>) -> Option<&FocusState> {
    frame.map(|frame| &frame.focus_state)
}

fn query_from_scope(
    project_root: Option<&str>,
    session_id: Option<&str>,
    continuity_id: Option<&str>,
    mode: Option<&str>,
) -> TrajectoryViewQuery {
    TrajectoryViewQuery {
        project_root: clean(project_root),
        session_id: clean(session_id),
        continuity_id: clean(continuity_id),
        mode: clean(mode),
    }
}

fn trajectory_id_for(project_root: &str, session_id: Option<&str>, suffix: &str) -> String {
    format!(
        "trajectory:{}:{}",
        stable_project_fingerprint(project_root, session_id),
        suffix
    )
}

fn view_project_root(view: &Value) -> String {
    view.pointer("/project_identity/project_root")
        .and_then(Value::as_str)
        .unwrap_or("unbound")
        .to_string()
}

fn view_session_id(view: &Value) -> Option<String> {
    view.pointer("/project_identity/session_id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn view_continuity_id(view: &Value) -> Option<String> {
    view.pointer("/project_identity/continuity_id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn view_str<'a>(view: &'a Value, pointer: &str) -> Option<&'a str> {
    view.pointer(pointer)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn trajectory_definition_status(value: &str) -> TrajectoryDefinitionStatus {
    match value {
        "clear" => TrajectoryDefinitionStatus::Clear,
        "provisional" => TrajectoryDefinitionStatus::Provisional,
        "conflicted" => TrajectoryDefinitionStatus::Conflicted,
        _ => TrajectoryDefinitionStatus::Unclear,
    }
}

fn trajectory_confidence(value: &str) -> TrajectoryConfidence {
    match value {
        "high" | "very_high" => TrajectoryConfidence::High,
        "low" => TrajectoryConfidence::Low,
        _ => TrajectoryConfidence::Medium,
    }
}

fn active_persisted_trajectory<'a>(
    state: &'a FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a TrajectoryProjectionRecord> {
    state
        .trajectory
        .active_trajectory_id
        .as_ref()
        .and_then(|id| {
            state
                .trajectory
                .records
                .iter()
                .find(|record| &record.trajectory_id == id)
        })
        .filter(|record| {
            project_root
                .map(|root| record.project_root.as_deref() == Some(root))
                .unwrap_or(true)
                && continuity_id
                    .map(|id| record.continuity_id.as_deref() == Some(id))
                    .unwrap_or(true)
        })
        .or_else(|| {
            state.trajectory.records.iter().rev().find(|record| {
                record.canonical
                    && project_root
                        .map(|root| record.project_root.as_deref() == Some(root))
                        .unwrap_or(true)
                    && continuity_id
                        .map(|id| record.continuity_id.as_deref() == Some(id))
                        .unwrap_or(true)
            })
        })
}

fn trajectory_record_from_define_payload(
    payload: &Value,
    body: &TrajectoryDefineGoalRequest,
) -> Option<TrajectoryProjectionRecord> {
    if payload.get("status").and_then(Value::as_str) != Some("completed") {
        return None;
    }
    let trajectory_id = payload.get("trajectory_id")?.as_str()?.to_string();
    let candidate = payload.get("trajectory_candidate")?;
    let long_term_goal = candidate.get("long_term_goal")?.as_str()?.to_string();
    let desired_end_state = candidate.get("desired_end_state")?.as_str()?.to_string();
    let project_root = view_str(payload, "/project_identity/project_root")
        .map(str::to_string)
        .or_else(|| body.project_root.clone())
        .or_else(|| session_identity_project_root(body.session_identity.as_ref()));
    let continuity_id = body
        .continuity_id
        .clone()
        .or_else(|| session_identity_continuity_id(body.session_identity.as_ref()));
    let definition_status = trajectory_definition_status(
        candidate
            .get("definition_status")
            .and_then(Value::as_str)
            .unwrap_or("clear"),
    );
    let confidence = trajectory_confidence(
        payload
            .pointer("/project_identity/confidence")
            .and_then(Value::as_str)
            .unwrap_or("medium"),
    );
    let source = candidate
        .get("goal_source")
        .and_then(Value::as_str)
        .unwrap_or("operator")
        .to_string();
    let inferred = source != "operator";
    let mut goal_provenance = vec![
        TrajectoryGoalProvenanceRecord {
            field: "long_term_goal".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        },
        TrajectoryGoalProvenanceRecord {
            field: "desired_end_state".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        },
    ];
    if body.short_term_goal.is_some() {
        goal_provenance.push(TrajectoryGoalProvenanceRecord {
            field: "short_term_goal".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        });
    }
    if body.current_state.is_some() {
        goal_provenance.push(TrajectoryGoalProvenanceRecord {
            field: "current_state".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        });
    }
    let milestone_id = format!("{trajectory_id}:milestone:active");
    let verified_evidence_refs = body.supersession_evidence_refs.clone().unwrap_or_default();
    let required_evidence_refs = {
        let refs = top_strings(
            body.required_evidence_refs.as_deref().unwrap_or(&[]),
            8,
            180,
        );
        if refs.is_empty() {
            if verified_evidence_refs.is_empty() {
                vec!["evidence proving desired end state".to_string()]
            } else {
                verified_evidence_refs.clone()
            }
        } else {
            refs
        }
    };
    let required_checks = {
        let checks = top_strings(body.required_checks.as_deref().unwrap_or(&[]), 8, 180);
        if checks.is_empty() {
            vec!["verify desired end state with linked evidence".to_string()]
        } else {
            checks
        }
    };
    let acceptance_risks = {
        let risks = top_strings(body.acceptance_risks.as_deref().unwrap_or(&[]), 8, 180);
        if risks.is_empty() {
            vec![
                "current state is stale or unverified".to_string(),
                "scope mismatch hides incomplete work".to_string(),
            ]
        } else {
            risks
        }
    };
    let not_done_if = {
        let traps = top_strings(body.not_done_if.as_deref().unwrap_or(&[]), 8, 180);
        if traps.is_empty() {
            vec![
                "desired end state lacks linked evidence".to_string(),
                "required checks have not run".to_string(),
                "project_root or continuity_id mismatch remains unresolved".to_string(),
            ]
        } else {
            traps
        }
    };
    Some(TrajectoryProjectionRecord {
        trajectory_id: trajectory_id.clone(),
        session_identity: body.session_identity.clone(),
        project_root,
        continuity_id,
        root_long_term_goal: long_term_goal.clone(),
        long_term_goal,
        desired_end_state: desired_end_state.clone(),
        short_term_goal: body
            .short_term_goal
            .as_deref()
            .map(|value| bounded(value, 240)),
        current_state: body
            .current_state
            .as_deref()
            .map(|value| bounded(value, 240)),
        session_clarity_status: definition_status,
        definition_status,
        confidence,
        goal_provenance,
        milestones: vec![TrajectoryMilestoneRecord {
            milestone_id: milestone_id.clone(),
            title: body
                .short_term_goal
                .as_deref()
                .map(|value| bounded(value, 160))
                .unwrap_or_else(|| "Active trajectory milestone".to_string()),
            desired_state_delta: desired_end_state.clone(),
            status: TrajectoryMilestoneStatus::Active,
            ..TrajectoryMilestoneRecord::default()
        }],
        active_milestone_id: Some(milestone_id),
        source_refs: json!({
            "project_identity": payload.get("project_identity").cloned().unwrap_or(Value::Null),
            "goal_source": source,
            "supersession_evidence_refs": body.supersession_evidence_refs.clone().unwrap_or_default(),
        }),
        definition_of_done: Some(TrajectoryDefinitionOfDoneRecord {
            criteria: vec![desired_end_state.clone()],
            evidence_required: vec!["evidence proving desired end state".to_string()],
            verified_evidence_refs,
            status: "defined".to_string(),
            desired_end_state: Some(desired_end_state),
            required_evidence_refs,
            required_checks,
            acceptance_risks,
            not_done_if,
        }),
        supersedes_trajectory_id: body.supersedes_trajectory_id.clone(),
        canonical: true,
        ..TrajectoryProjectionRecord::default()
    })
}

async fn dispatch_event(
    state: &Arc<AppState>,
    event: FocusaEvent,
) -> Result<(), (StatusCode, Json<Value>)> {
    let _guard = state.write_serial_lock.lock().await;
    let current = { state.focusa.read().await.clone() };
    let result = reducer::reduce_with_meta(current, event, None, None, false).map_err(|error| {
        (
            StatusCode::OK,
            Json(json!({
                "status": "rejected",
                "canonical": false,
                "degraded": true,
                "failure_class": "validation_rejected",
                "error": error.to_string(),
                "next_step_hint": "correct the trajectory payload and retry"
            })),
        )
    })?;

    let new_state = result.new_state;
    for emitted in result.emitted_events {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event: emitted,
            correlation_id: Some("api:trajectory".to_string()),
            origin: SignalOrigin::Adapter,
            machine_id: None,
            instance_id: None,
            session_id: new_state.session.as_ref().map(|session| session.session_id),
            thread_id: None,
            is_observation: false,
        };
        if let Err(error) = state.persistence.append_event(&entry) {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "rejected",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": "persistence_failed",
                    "error": error.to_string(),
                    "next_step_hint": "retry after trajectory persistence recovers"
                })),
            ));
        } else if let Ok(serialized) = serde_json::to_string(&entry) {
            let _ = state.events_tx.send(serialized);
        }
    }

    *state.focusa.write().await = new_state;
    state.mark_external_mutation();
    Ok(())
}

fn source_precedence() -> Vec<&'static str> {
    vec![
        "operator_confirmed",
        "durable_supersession_evidence",
        "workpoint_checkpoint",
        "active_focus_frame",
        "focus_state_projection",
        "inferred_context",
    ]
}

fn lifecycle_refresh_triggers() -> Vec<&'static str> {
    vec![
        "operator_goal_changed",
        "trajectory_superseded",
        "workpoint_completed",
        "evidence_captured",
        "project_identity_mismatch",
        "continuity_id_changed",
        "post_compaction_resume",
    ]
}

fn trajectory_clarity_gate_payload(
    definition_status: &str,
    project_identity_status: &str,
    missing_facts: &[&str],
    mismatch_count: usize,
    evidence_count: usize,
) -> Value {
    let mut blocking_reasons = Vec::new();
    if mismatch_count > 0 || project_identity_status == "mismatch" {
        blocking_reasons.push("conflicting_project_or_continuity_scope");
    }
    for fact in missing_facts {
        blocking_reasons.push(*fact);
    }
    if evidence_count == 0 && definition_status != "unclear" {
        blocking_reasons.push("stale_or_missing_evidence_refs");
    }
    let status = if mismatch_count > 0 || definition_status == "conflicted" {
        "conflicted"
    } else if missing_facts
        .iter()
        .any(|fact| *fact == "long_term_goal" || *fact == "desired_end_state")
        || definition_status == "unclear"
    {
        "unclear"
    } else if !missing_facts.is_empty() || evidence_count == 0 || definition_status == "provisional"
    {
        "provisional"
    } else {
        "clear"
    };
    let recommended_action = match status {
        "clear" => "proceed",
        "unclear" => "operator_input",
        "conflicted" => "verify_first",
        _ => "verify_first",
    };
    json!({
        "status": status,
        "recommended_action": recommended_action,
        "blocking_reasons": blocking_reasons,
        "source_precedence": source_precedence(),
        "root_goal_change_policy": "operator_confirmed_or_durable_supersession_evidence_only",
        "refresh_triggers": lifecycle_refresh_triggers(),
        "operator_confirm_path": "ask for missing long_term_goal/desired_end_state or explicit supersession confirmation only",
    })
}

fn define_goal_lifecycle_status(
    body: &TrajectoryDefineGoalRequest,
    basic_valid: bool,
) -> (&'static str, bool, Vec<String>) {
    let source = body
        .goal_source
        .as_deref()
        .unwrap_or("operator")
        .trim()
        .to_ascii_lowercase();
    let operator_confirmed = body.operator_confirmed.unwrap_or(source == "operator");
    let durable_supersession = body
        .supersession_evidence_refs
        .as_ref()
        .map(|refs| refs.iter().any(|r| !r.trim().is_empty()))
        .unwrap_or(false)
        || source == "durable_supersession";
    let supersession_requested = body
        .supersedes_trajectory_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let root_goal_change_allowed =
        !supersession_requested || operator_confirmed || durable_supersession;
    let mut errors = Vec::new();
    if !basic_valid {
        errors.push("long_term_goal and desired_end_state are required".to_string());
    }
    if !root_goal_change_allowed {
        errors.push(
            "root goal supersession requires operator confirmation or durable supersession evidence"
                .to_string(),
        );
    }
    let status = if !basic_valid {
        "unclear"
    } else if !root_goal_change_allowed {
        "conflicted"
    } else if operator_confirmed || durable_supersession {
        "clear"
    } else {
        "provisional"
    };
    (status, root_goal_change_allowed, errors)
}

fn status_from_validation(valid: bool) -> &'static str {
    if valid {
        "completed"
    } else {
        "validation_rejected"
    }
}

fn trajectory_group_key(value: Option<&str>) -> Option<String> {
    let text = value?.trim().to_ascii_lowercase();
    if text.is_empty() {
        return None;
    }
    let key = text
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .take(10)
        .collect::<Vec<_>>()
        .join("-");
    (!key.is_empty()).then_some(key)
}

fn trajectory_similarity_group_payload(
    project_root: &str,
    long_term_goal: Option<&str>,
    mid_level_goal: Option<&str>,
    low_level_goal: Option<&str>,
    continuity_id: Option<&str>,
) -> Value {
    json!({
        "advisory_only": true,
        "authority_boundary": "project_root_plus_continuity_id",
        "must_not_merge_sessions": true,
        "project_root": project_root,
        "continuity_id": continuity_id,
        "high_level_group_key": trajectory_group_key(long_term_goal),
        "mid_level_group_key": trajectory_group_key(mid_level_goal),
        "low_level_group_key": trajectory_group_key(low_level_goal),
        "grouping_policy": "high-level similarity may orient or cluster sessions; mid/low-level differences and Workpoint identity prevent authority merging",
    })
}

fn trajectory_view_payload(state: &FocusaState, query: &TrajectoryViewQuery) -> Value {
    let query_project = clean(query.project_root.as_deref());
    let query_session = clean(query.session_id.as_deref());
    let query_continuity = clean(query.continuity_id.as_deref());
    let frame = scoped_active_frame(state, query_project.as_deref(), query_continuity.as_deref());
    let focus_state = focus_state_for(frame);
    let workpoint =
        scoped_active_workpoint(state, query_project.as_deref(), query_continuity.as_deref());
    let workpoint_project = workpoint.and_then(|record| clean(record.project_root.as_deref()));
    let workpoint_session = workpoint.and_then(|record| clean(record.session_id.as_deref()));
    let workpoint_continuity = workpoint.and_then(|record| clean(record.continuity_id.as_deref()));
    let persisted_candidate = active_persisted_trajectory(
        state,
        query_project.as_deref().or(workpoint_project.as_deref()),
        query_continuity
            .as_deref()
            .or(workpoint_continuity.as_deref()),
    );
    let persisted_project =
        persisted_candidate.and_then(|record| clean(record.project_root.as_deref()));
    let persisted_continuity =
        persisted_candidate.and_then(|record| clean(record.continuity_id.as_deref()));
    let persisted_session = persisted_candidate
        .and_then(|record| record.session_identity.as_ref())
        .and_then(|identity| {
            clean(identity.pi_session_id.as_deref())
                .or_else(|| clean(Some(identity.session_frame_key.as_str())))
        });
    let project_root = query_project
        .clone()
        .or(workpoint_project.clone())
        .or(persisted_project.clone())
        .unwrap_or_else(|| "unbound".to_string());
    let session_id = query_session
        .clone()
        .or(workpoint_session.clone())
        .or(persisted_session.clone());
    let continuity_id = query_continuity
        .clone()
        .or(workpoint_continuity.clone())
        .or(persisted_continuity.clone());
    let persisted_trajectory = active_persisted_trajectory(
        state,
        Some(project_root.as_str()).filter(|root| *root != "unbound"),
        continuity_id.as_deref(),
    );
    let project_identity_api = if project_root != "unbound" {
        project_identity_payload_for_scope(Some(project_root.as_str()), Some(project_root.as_str()))
    } else {
        project_identity_payload_for_scope(None, None)
    };
    let project_identity_record = project_identity_api
        .get("project_identity")
        .cloned()
        .unwrap_or(Value::Null);
    let project_identity_quorum_status = project_identity_record
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let project_identity_quorum_confidence = project_identity_record
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or("low")
        .to_string();
    let mut identity_warnings = Vec::new();

    let mut mismatches = Vec::new();
    if let (Some(expected), Some(actual)) = (&query_project, &workpoint_project)
        && expected != actual
    {
        mismatches.push(json!({
            "field": "project_root",
            "expected": expected,
            "actual": actual,
            "source": "workpoint",
        }));
    }
    if let (Some(expected), Some(actual)) = (&query_continuity, &workpoint_continuity)
        && expected != actual
    {
        mismatches.push(json!({
            "field": "continuity_id",
            "expected": expected,
            "actual": actual,
            "source": "workpoint",
        }));
    }
    if let (Some(expected), Some(actual)) = (&query_session, &workpoint_session)
        && expected != actual
    {
        identity_warnings.push(json!({
            "field": "session_id",
            "expected": expected,
            "actual": actual,
            "source": "workpoint",
            "policy": "session_id_is_temporal_metadata"
        }));
    }

    let project_bound = project_root != "unbound";
    let scope_match = mismatches.is_empty();
    let project_identity_status = if project_bound && scope_match {
        "verified"
    } else if project_bound {
        "mismatch"
    } else {
        "unbound"
    };
    let project_confidence = if project_identity_quorum_confidence == "high"
        && project_identity_status == "verified"
        && query_project.is_some()
    {
        "high"
    } else if project_identity_status == "verified" && project_identity_quorum_confidence != "low" {
        "medium"
    } else {
        "low"
    };

    let fs_intent = focus_state.map(|fs| fs.intent.as_str());
    let fs_current = focus_state.map(|fs| fs.current_state.as_str());
    let frame_goal = frame.map(|frame| frame.goal.as_str());
    let frame_title = frame.map(|frame| frame.title.as_str());
    let persisted_long_term_goal =
        persisted_trajectory.map(|record| record.long_term_goal.as_str());
    let persisted_desired_end_state =
        persisted_trajectory.map(|record| record.desired_end_state.as_str());
    let persisted_current_state =
        persisted_trajectory.and_then(|record| record.current_state.as_deref());
    let persisted_short_term_goal =
        persisted_trajectory.and_then(|record| record.short_term_goal.as_deref());
    let workpoint_next = workpoint.and_then(|record| record.next_slice.as_deref());
    let workpoint_action = workpoint
        .and_then(|record| record.action_intent.as_ref())
        .map(|intent| intent.action_type.as_str());

    // Spec96: Workpoint/frame text may shape short-term goals and candidates,
    // but must not silently become the project long-term goal or desired end
    // state. Those require persisted Trajectory state or Focus State intent.
    let long_term_goal = first_nonempty(&[persisted_long_term_goal, fs_intent]);
    let desired_end_state = first_nonempty(&[persisted_desired_end_state, fs_intent]);
    let current_state = first_nonempty(&[persisted_current_state, fs_current]);
    let short_term_goal = first_nonempty(&[
        persisted_short_term_goal,
        workpoint_next,
        workpoint_action,
        frame_goal,
        frame_title,
        fs_current,
    ]);
    let active_gap = match (desired_end_state.as_deref(), current_state.as_deref()) {
        (Some(desired), Some(current)) if desired == current => None,
        (Some(_), Some(_)) => first_nonempty(&[workpoint_next, workpoint_action])
            .map(|gap| bounded(&gap, 240))
            .or_else(|| Some("Current verified state differs from desired end state".to_string())),
        _ => Some("Trajectory gap unclear until desired end state and current verified state are both present".to_string()),
    };
    let mid_level_goal = first_nonempty(&[
        short_term_goal.as_deref(),
        workpoint_action,
        frame_goal,
        frame_title,
        fs_current,
    ]);
    let low_level_goal = first_nonempty(&[
        workpoint_next,
        workpoint_action,
        active_gap.as_deref(),
        frame_title,
    ]);
    let similarity_group = trajectory_similarity_group_payload(
        &project_root,
        long_term_goal.as_deref(),
        mid_level_goal.as_deref(),
        low_level_goal.as_deref(),
        continuity_id.as_deref(),
    );

    let missing_facts = [
        ("project_identity", project_bound && scope_match),
        ("long_term_goal", long_term_goal.is_some()),
        ("desired_end_state", desired_end_state.is_some()),
        ("current_verified_state", current_state.is_some()),
        ("next_workpoint", workpoint.is_some()),
    ]
    .into_iter()
    .filter_map(|(name, present)| (!present).then_some(name))
    .collect::<Vec<_>>();

    let context_score =
        (100_i64 - (missing_facts.len() as i64 * 18) - (mismatches.len() as i64 * 25))
            .clamp(0, 100);
    let recommended_action = if !scope_match {
        "verify_first"
    } else if missing_facts
        .iter()
        .any(|fact| *fact == "long_term_goal" || *fact == "desired_end_state")
    {
        "operator_input"
    } else if missing_facts.is_empty() {
        "proceed"
    } else {
        "verify_first"
    };
    let definition_status = if !scope_match {
        "conflicted"
    } else if missing_facts.is_empty() {
        "clear"
    } else if long_term_goal.is_some() || desired_end_state.is_some() || current_state.is_some() {
        "provisional"
    } else {
        "unclear"
    };

    let do_not_use = mismatches
        .iter()
        .filter_map(|mismatch| mismatch.get("field").and_then(Value::as_str))
        .map(|field| format!("mismatched_{field}_context"))
        .chain((!scope_match).then_some("cross_scope_workpoint_resume".to_string()))
        .collect::<Vec<_>>();

    let evidence_refs = workpoint
        .map(|record| {
            record
                .verification_records
                .iter()
                .take(8)
                .map(|verification| {
                    json!({
                        "target_ref": verification.target_ref,
                        "result": bounded(&verification.result, 180),
                        "evidence_ref": verification.evidence_ref,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let blockers = workpoint
        .map(|record| {
            record
                .blockers
                .iter()
                .take(8)
                .map(|blocker| {
                    json!({
                        "reason": bounded(&blocker.reason, 180),
                        "severity": blocker.severity,
                        "target_ref": blocker.target_ref,
                        "status": blocker.status,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let clarity_gate = trajectory_clarity_gate_payload(
        definition_status,
        project_identity_status,
        &missing_facts,
        mismatches.len(),
        evidence_refs.len(),
    );
    let clarity_recommended_action = clarity_gate
        .get("recommended_action")
        .and_then(Value::as_str)
        .unwrap_or(recommended_action);
    let proceed_posture = match clarity_recommended_action {
        "proceed" => "proceed",
        "verify_first" => "verify_first",
        "operator_input" | "operator_required" => "operator_required",
        _ => "verify_first",
    };
    let stale_refs = Vec::<String>::new();
    let conflicting_signals = mismatches
        .iter()
        .map(|mismatch| {
            let field = mismatch
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("unknown_field");
            let source = mismatch
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("unknown_source");
            let expected = mismatch
                .get("expected")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let actual = mismatch
                .get("actual")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            format!("{field} mismatch from {source}: expected {expected}, actual {actual}")
        })
        .collect::<Vec<_>>();
    let ask_operator_if = missing_facts
        .iter()
        .filter_map(|fact| match *fact {
            "long_term_goal" => Some("confirm the project long-term goal".to_string()),
            "desired_end_state" => Some("confirm the desired end state".to_string()),
            _ => None,
        })
        .chain((!scope_match).then_some("confirm project_root and continuity_id scope".to_string()))
        .collect::<Vec<_>>();
    let mut relevance_rationale = vec![json!({
        "ref": "project_identity",
        "why_included": "bounds Trajectory to project_root plus continuity_id",
        "confidence": project_confidence,
    })];
    if let Some(record) = persisted_trajectory {
        relevance_rationale.push(json!({
            "ref": format!("trajectory:{}", record.trajectory_id),
            "why_included": "provides persisted goal-state binding",
            "confidence": if record.canonical { "high" } else { "medium" },
        }));
    }
    if focus_state.is_some() {
        relevance_rationale.push(json!({
            "ref": "focus_state:active",
            "why_included": "provides compact intent, current state, decisions, and constraints",
            "confidence": "medium",
        }));
    }
    if let Some(record) = workpoint {
        relevance_rationale.push(json!({
            "ref": format!("workpoint:{}", record.workpoint_id),
            "why_included": "provides active short-term execution point and next candidate",
            "confidence": if record.canonical { "high" } else { "medium" },
        }));
    }
    if let Some(record) = frame {
        relevance_rationale.push(json!({
            "ref": format!("frame:{}", record.id),
            "why_included": "provides current Focus Stack alignment evidence",
            "confidence": "medium",
        }));
    }
    if !evidence_refs.is_empty() {
        relevance_rationale.push(json!({
            "ref": "workpoint:evidence_refs",
            "why_included": "supports verified current-state and completion claims",
            "confidence": "high",
        }));
    }

    let next_workpoint_candidate = workpoint.map(|record| {
        json!({
            "workpoint_id": record.workpoint_id,
            "work_item_id": record.work_item_id,
            "status": record.status,
            "canonical": record.canonical,
            "mission": record.mission.as_deref().map(|value| bounded(value, 240)),
            "action_intent": record.action_intent,
            "active_object_refs": record.active_object_refs.iter().take(8).cloned().collect::<Vec<_>>(),
            "next_slice": record.next_slice.as_deref().map(|value| bounded(value, 240)),
            "advisory_only": true,
        })
    });

    let status = if !scope_match {
        "degraded"
    } else if definition_status == "unclear" {
        "not_found"
    } else {
        "completed"
    };
    let canonical = status == "completed" && project_identity_status == "verified";
    let active_trajectory_id = persisted_trajectory
        .map(|record| record.trajectory_id.clone())
        .unwrap_or_else(|| {
            trajectory_id_for(
                &project_root,
                continuity_id.as_deref().or(session_id.as_deref()),
                "active",
            )
        });
    let lifecycle_checkpoints = state
        .trajectory
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.trajectory_id == active_trajectory_id)
        .rev()
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let lifecycle_state_deltas = state
        .trajectory
        .state_deltas
        .iter()
        .filter(|delta| delta.trajectory_id == active_trajectory_id)
        .rev()
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
    let since_checkpoint = lifecycle_checkpoints
        .first()
        .and_then(|checkpoint| checkpoint.persisted_at.as_ref())
        .map(|value| value.to_rfc3339());
    let changed_refs = lifecycle_state_deltas
        .iter()
        .map(|delta| {
            delta
                .current_state
                .as_deref()
                .map(|value| format!("current_state:{}", bounded(value, 120)))
                .unwrap_or_else(|| format!("reason:{}", bounded(&delta.reason, 120)))
        })
        .collect::<Vec<_>>();
    let delta_evidence_refs = lifecycle_state_deltas
        .iter()
        .flat_map(|delta| delta.evidence_refs.iter().take(8).cloned())
        .take(8)
        .collect::<Vec<_>>();
    let current_state_delta = json!({
        "since_checkpoint": since_checkpoint,
        "changed_refs": changed_refs,
        "evidence_refs": delta_evidence_refs,
    });

    json!({
        "status": status,
        "canonical": canonical,
        "degraded": status == "degraded",
        "source": "per_project_trajectory_projection_v1",
        "mode": query.mode.as_deref().unwrap_or("summary"),
        "project_identity": {
            "status": project_identity_status,
            "project_root": project_root,
            "session_id": session_id,
            "continuity_id": continuity_id,
            "authority_boundary": "project_root_plus_continuity_id",
            "session_id_policy": "temporal_metadata_only",
            "fingerprint": stable_project_fingerprint(&project_root, continuity_id.as_deref().or(session_id.as_deref())),
            "confidence": project_confidence,
            "quorum_status": project_identity_quorum_status,
            "quorum_confidence": project_identity_quorum_confidence,
            "project_identity_api": project_identity_record.clone(),
            "signals": [
                {"source": "query", "project_root": query_project, "session_id": query_session, "continuity_id": query_continuity},
                {"source": "workpoint", "project_root": workpoint_project, "session_id": workpoint_session, "continuity_id": workpoint_continuity},
                {"source": "project_identity_api", "project_identity": project_identity_record.clone()}
            ],
            "mismatches": mismatches,
            "warnings": identity_warnings,
        },
        "trajectory": {
            "trajectory_id": active_trajectory_id,
            "definition_status": definition_status,
            "long_term_goal": long_term_goal.as_deref().map(|value| bounded(value, 240)),
            "desired_end_state": desired_end_state.as_deref().map(|value| bounded(value, 240)),
            "current_state": current_state.as_deref().map(|value| bounded(value, 240)),
            "short_term_goal": short_term_goal.as_deref().map(|value| bounded(value, 240)),
            "mid_level_goal": mid_level_goal.as_deref().map(|value| bounded(value, 240)),
            "low_level_goal": low_level_goal.as_deref().map(|value| bounded(value, 240)),
            "active_gap": active_gap,
            "similarity_group": similarity_group,
            "durable_lifecycle": {
                "persisted": persisted_trajectory.is_some(),
                "active_trajectory_id": state.trajectory.active_trajectory_id.clone(),
                "canonical": persisted_trajectory.map(|record| record.canonical).unwrap_or(false),
                "root_goal_stability": persisted_trajectory.map(|record| record.root_goal_stability),
                "supersedes_trajectory_id": persisted_trajectory.and_then(|record| record.supersedes_trajectory_id.clone()),
                "created_at": persisted_trajectory.and_then(|record| record.created_at.as_ref().map(|value| value.to_rfc3339())),
                "updated_at": persisted_trajectory.and_then(|record| record.updated_at.as_ref().map(|value| value.to_rfc3339())),
                "checkpoint_count": lifecycle_checkpoints.len(),
                "state_delta_count": lifecycle_state_deltas.len(),
                "checkpoints": lifecycle_checkpoints,
                "state_deltas": lifecycle_state_deltas,
                "definition_of_done": persisted_trajectory.and_then(|record| record.definition_of_done.clone()),
                "goal_provenance": persisted_trajectory.map(|record| record.goal_provenance.clone()).unwrap_or_default(),
                "milestones": persisted_trajectory.map(|record| record.milestones.clone()).unwrap_or_default(),
            },
            "lifecycle": {
                "clarity_gate": clarity_gate,
                "source_precedence": source_precedence(),
                "refresh_triggers": lifecycle_refresh_triggers(),
            },
            "active_workpoint_id": workpoint.map(|record| record.workpoint_id),
            "frame_id": frame.map(|frame| frame.id),
            "beads_issue_id": frame.map(|frame| frame.beads_issue_id.clone()),
            "evidence_refs": evidence_refs,
            "blockers": blockers,
        },
        "intelligence_view": {
            "context_sufficiency": {
                "score": context_score,
                "status": definition_status,
                "proceed_posture": proceed_posture,
                "missing_facts": missing_facts,
                "stale_refs": stale_refs,
                "conflicting_signals": conflicting_signals,
                "recommended_action": clarity_recommended_action,
            },
            "similarity_group": similarity_group,
            "clarity_gate": clarity_gate,
            "relevance_rationale": relevance_rationale,
            "current_state_delta": current_state_delta,
            "learning_refs": Vec::<String>::new(),
            "prediction_refs": Vec::<String>::new(),
            "ask_operator_if": ask_operator_if,
            "do_not_use": do_not_use,
            "next_workpoint_candidate": next_workpoint_candidate,
            "tool_affordances": [
                "focusa_trajectory_view",
                "focusa_workpoint_resume",
                "focusa_active_object_resolve",
                "focusa_evidence_capture"
            ],
            "recent_results": focus_state.map(|fs| top_strings(&fs.recent_results, 4, 180)).unwrap_or_default(),
            "decisions": focus_state.map(|fs| top_strings(&fs.decisions, 4, 160)).unwrap_or_default(),
            "constraints": focus_state.map(|fs| top_strings(&fs.constraints, 4, 180)).unwrap_or_default(),
        },
        "next_tools": if status == "not_found" { json!(["focusa_trajectory_define_goal", "focusa_project_identity"]) } else { json!(["focusa_trajectory_view", "focusa_workpoint_resume", "focusa_active_object_resolve"]) },
        "warnings": if canonical { Vec::<String>::new() } else if status == "not_found" { vec!["trajectory is not set for this project scope; define or confirm the goal".to_string()] } else { vec!["trajectory projection is degraded or provisional; verify before treating as canonical".to_string()] },
    })
}

fn define_goal_payload(state: &FocusaState, body: &TrajectoryDefineGoalRequest) -> Value {
    let query = scoped_query_from_identity(
        body.project_root.as_deref(),
        body.session_id.as_deref(),
        body.continuity_id.as_deref(),
        Some("summary"),
        body.session_identity.as_ref(),
    );
    let view = trajectory_view_payload(state, &query);
    let project_root = view_project_root(&view);
    let session_id = view_continuity_id(&view).or_else(|| view_session_id(&view));
    let basic_valid =
        !body.long_term_goal.trim().is_empty() && !body.desired_end_state.trim().is_empty();
    let (lifecycle_status, root_goal_change_allowed, validation_errors) =
        define_goal_lifecycle_status(body, basic_valid);
    let valid = basic_valid && root_goal_change_allowed;
    let trajectory_id = trajectory_id_for(
        &project_root,
        session_id.as_deref(),
        body.idempotency_key.as_deref().unwrap_or("defined-goal"),
    );
    json!({
        "status": status_from_validation(valid),
        "canonical": valid,
        "degraded": !valid,
        "advisory_only": true,
        "mutates_canonical_state": valid,
        "persisted": valid,
        "trajectory_id": trajectory_id,
        "project_identity": view.get("project_identity").cloned().unwrap_or(Value::Null),
        "trajectory_candidate": {
            "definition_status": lifecycle_status,
            "long_term_goal": bounded(&body.long_term_goal, 240),
            "desired_end_state": bounded(&body.desired_end_state, 240),
            "short_term_goal": body.short_term_goal.as_deref().map(|value| bounded(value, 240)),
            "current_state": body.current_state.as_deref().map(|value| bounded(value, 240)),
            "goal_source": body.goal_source.as_deref().unwrap_or("operator"),
            "operator_confirmed": body.operator_confirmed.unwrap_or_else(|| body.goal_source.as_deref().unwrap_or("operator") == "operator"),
            "supersedes_trajectory_id": body.supersedes_trajectory_id,
            "supersession_evidence_refs": body.supersession_evidence_refs.clone().unwrap_or_default().into_iter().take(8).collect::<Vec<_>>(),
            "root_goal_change_allowed": root_goal_change_allowed,
            "provenance": "operator_or_tool_supplied_projection_candidate",
            "lifecycle": {
                "source_precedence": source_precedence(),
                "refresh_triggers": lifecycle_refresh_triggers(),
                "root_goal_change_policy": "operator_confirmed_or_durable_supersession_evidence_only",
            },
        },
        "validation_errors": validation_errors,
        "next_step_hint": if valid { "use focusa_trajectory_assess, then propose a Workpoint candidate if the gap is actionable" } else { "provide missing goals or operator confirmation/durable supersession evidence" },
        "next_tools": ["focusa_trajectory_assess", "focusa_trajectory_propose_workpoint"],
    })
}

fn assess_payload(state: &FocusaState, body: &TrajectoryAssessRequest) -> Value {
    let query = scoped_query_from_identity(
        body.project_root.as_deref(),
        body.session_id.as_deref(),
        body.continuity_id.as_deref(),
        Some("summary"),
        body.session_identity.as_ref(),
    );
    let view = trajectory_view_payload(state, &query);
    let trajectory = view.get("trajectory").cloned().unwrap_or(Value::Null);
    let current_state = body
        .observed_state
        .as_deref()
        .and_then(|value| clean(Some(value)))
        .or_else(|| view_str(&view, "/trajectory/current_state").map(str::to_string));
    let desired_end_state = view_str(&view, "/trajectory/desired_end_state").map(str::to_string);
    let mut gaps = Vec::new();
    if desired_end_state.is_none() {
        gaps.push(json!({"gap_ref":"missing_desired_end_state", "severity":"high", "recommended_action":"define_goal"}));
    }
    if current_state.is_none() {
        gaps.push(json!({"gap_ref":"missing_current_state", "severity":"high", "recommended_action":"verify_current_state"}));
    }
    if let (Some(current), Some(desired)) = (&current_state, &desired_end_state)
        && current != desired
    {
        gaps.push(json!({
            "gap_ref": "current_to_desired_delta",
            "severity": "medium",
            "current_state": bounded(current, 180),
            "desired_end_state": bounded(desired, 180),
            "recommended_action": "propose_workpoint",
        }));
    }
    let clarity_gate = view
        .pointer("/intelligence_view/clarity_gate")
        .cloned()
        .unwrap_or(Value::Null);
    let clarity_action = clarity_gate
        .get("recommended_action")
        .and_then(Value::as_str);
    let recommended_action = if matches!(clarity_action, Some("operator_input")) {
        "operator_input"
    } else if matches!(clarity_action, Some("verify_first")) && gaps.is_empty() {
        "verify_first"
    } else if gaps
        .iter()
        .any(|gap| gap.get("recommended_action").and_then(Value::as_str) == Some("define_goal"))
    {
        "define_goal"
    } else if gaps.iter().any(|gap| {
        gap.get("recommended_action").and_then(Value::as_str) == Some("verify_current_state")
    }) {
        "verify_first"
    } else if gaps.is_empty() {
        "proceed"
    } else {
        "propose_workpoint"
    };
    json!({
        "status": "completed",
        "canonical": view.get("canonical").and_then(Value::as_bool).unwrap_or(false),
        "degraded": view.get("degraded").and_then(Value::as_bool).unwrap_or(false),
        "project_identity": view.get("project_identity").cloned().unwrap_or(Value::Null),
        "trajectory": trajectory,
        "observed_state": current_state.as_deref().map(|value| bounded(value, 240)),
        "desired_end_state": desired_end_state.as_deref().map(|value| bounded(value, 240)),
        "gaps": gaps,
        "evidence_refs": body.evidence_refs.clone().unwrap_or_default().into_iter().take(8).collect::<Vec<_>>(),
        "context_sufficiency": view.pointer("/intelligence_view/context_sufficiency").cloned().unwrap_or(Value::Null),
        "clarity_gate": clarity_gate,
        "recommended_action": recommended_action,
        "next_tools": if recommended_action == "propose_workpoint" { vec!["focusa_trajectory_propose_workpoint"] } else { vec!["focusa_trajectory_view"] },
    })
}

fn trajectory_candidate_blockers(view: &Value) -> Vec<Value> {
    let mut blockers = view
        .pointer("/trajectory/blockers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let clarity = view
        .pointer("/intelligence_view/clarity_gate/status")
        .and_then(Value::as_str)
        .unwrap_or("unclear");
    if clarity != "clear" && clarity != "provisional" {
        blockers.push(json!({
            "reason": format!("trajectory_clarity_gate_{clarity}"),
            "severity": "high",
            "status": "open",
            "target_ref": "trajectory_clarity_gate",
        }));
    }
    if view
        .get("degraded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        blockers.push(json!({
            "reason": "trajectory_projection_degraded",
            "severity": "high",
            "status": "open",
            "target_ref": "project_identity",
        }));
    }
    blockers.into_iter().take(8).collect()
}

fn trajectory_candidate_do_not_drift(view: &Value) -> Vec<Value> {
    let mut items = view
        .pointer("/intelligence_view/do_not_use")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for guard in [
        "Trajectory proposal is advisory; call focusa_workpoint_checkpoint before acting",
        "Do not call work-loop selection or execution from trajectory proposal",
        "Do not merge same-high-level sessions without project_root+continuity_id match",
    ] {
        items.push(Value::String(guard.to_string()));
    }
    items.into_iter().take(12).collect()
}

fn propose_workpoint_payload(
    state: &FocusaState,
    body: &TrajectoryProposeWorkpointRequest,
) -> Value {
    let query = scoped_query_from_identity(
        body.project_root.as_deref(),
        body.session_id.as_deref(),
        body.continuity_id.as_deref(),
        Some("summary"),
        body.session_identity.as_ref(),
    );
    let view = trajectory_view_payload(state, &query);
    let project_root = view_project_root(&view);
    let session_id = view_continuity_id(&view).or_else(|| view_session_id(&view));
    let trajectory_id = body
        .trajectory_id
        .clone()
        .unwrap_or_else(|| trajectory_id_for(&project_root, session_id.as_deref(), "active"));
    let active_gap = view_str(&view, "/trajectory/active_gap")
        .unwrap_or("Continue from per-project trajectory gap");
    let target_ref = body.target_ref.clone().or_else(|| {
        view.pointer("/intelligence_view/next_workpoint_candidate/action_intent/target_ref")
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    let action_type = body
        .action_type
        .clone()
        .or_else(|| {
            view.pointer("/intelligence_view/next_workpoint_candidate/action_intent/action_type")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "trajectory_gap_followup".to_string());
    let blockers = trajectory_candidate_blockers(&view);
    let do_not_drift = trajectory_candidate_do_not_drift(&view);
    let checkpoint_ready = blockers.is_empty();
    json!({
        "status": "completed",
        "canonical": false,
        "degraded": view.get("degraded").and_then(Value::as_bool).unwrap_or(false),
        "advisory_only": true,
        "mutates_canonical_state": false,
        "no_execution_side_effects": true,
        "trajectory_id": trajectory_id,
        "project_identity": view.get("project_identity").cloned().unwrap_or(Value::Null),
        "workpoint_candidate": {
            "candidate_type": "advisory_workpoint_candidate_v1",
            "mission": bounded(active_gap, 240),
            "action_intent": {
                "action_type": action_type,
                "target_ref": target_ref,
                "verification_hooks": [
                    "verify trajectory gap evidence before completion",
                    "confirm project_root+continuity_id before checkpoint",
                    "link proof via focusa_workpoint_link_evidence after execution"
                ],
                "status": if checkpoint_ready { "ready" } else { "blocked" }
            },
            "active_object_refs": target_ref.iter().cloned().collect::<Vec<_>>(),
            "target_refs": target_ref.iter().cloned().collect::<Vec<_>>(),
            "next_slice": bounded(active_gap, 240),
            "blockers": blockers,
            "do_not_drift": do_not_drift,
            "checkpoint_required": true,
            "handoff_policy": {
                "advisory_only": true,
                "canonicalization_tool": "focusa_workpoint_checkpoint",
                "authority_path": "operator_accepts_candidate_then_workpoint_checkpoint",
                "forbidden_side_effects": ["work_loop_select_next", "execute_action", "mutate_focus_state"],
            }
        },
        "next_step_hint": if checkpoint_ready { "If accepted, pass this candidate to focusa_workpoint_checkpoint; Trajectory does not auto-promote Workpoints." } else { "Resolve blockers with focusa_trajectory_assess or operator confirmation before checkpointing this candidate." },
        "next_tools": if checkpoint_ready { vec!["focusa_workpoint_checkpoint"] } else { vec!["focusa_trajectory_assess", "focusa_active_object_resolve"] },
    })
}

fn checkpoint_payload(state: &FocusaState, body: &TrajectoryCheckpointRequest) -> Value {
    let query = scoped_query_from_identity(
        body.project_root.as_deref(),
        body.session_id.as_deref(),
        body.continuity_id.as_deref(),
        Some("summary"),
        body.session_identity.as_ref(),
    );
    let view = trajectory_view_payload(state, &query);
    let project_root = view_project_root(&view);
    let session_id = view_continuity_id(&view).or_else(|| view_session_id(&view));
    let trajectory_id = trajectory_id_for(
        &project_root,
        session_id.as_deref(),
        body.idempotency_key.as_deref().unwrap_or("checkpoint"),
    );
    json!({
        "status": "completed",
        "canonical": true,
        "degraded": false,
        "persisted": true,
        "advisory_only": true,
        "trajectory_checkpoint": {
            "trajectory_id": trajectory_id,
            "summary": body.summary.as_deref().map(|value| bounded(value, 240)),
            "project_identity": view.get("project_identity").cloned().unwrap_or(Value::Null),
            "trajectory": view.get("trajectory").cloned().unwrap_or(Value::Null),
            "intelligence_view": view.get("intelligence_view").cloned().unwrap_or(Value::Null),
        },
        "next_step_hint": "Trajectory checkpoint is advisory until reducer-backed trajectory metadata exists; pair with Workpoint checkpoint for canonical continuation.",
        "next_tools": ["focusa_workpoint_checkpoint", "focusa_trajectory_resume"],
    })
}

fn resume_payload(state: &FocusaState, body: &TrajectoryResumeRequest) -> Value {
    let query = scoped_query_from_identity(
        body.project_root.as_deref(),
        body.session_id.as_deref(),
        body.continuity_id.as_deref(),
        body.mode.as_deref(),
        body.session_identity.as_ref(),
    );
    let view = trajectory_view_payload(state, &query);
    json!({
        "status": view.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "canonical": view.get("canonical").cloned().unwrap_or(Value::Bool(false)),
        "degraded": view.get("degraded").cloned().unwrap_or(Value::Bool(true)),
        "resume_packet": view,
        "next_step_hint": "Inject trajectory resume packet plus Workpoint resume before the next agent turn.",
        "next_tools": ["focusa_workpoint_resume", "focusa_active_object_resolve"],
    })
}

fn attach_trajectory_tool_result(
    mut payload: Value,
    side_effects: Vec<&str>,
    evidence_refs: Vec<String>,
) -> Value {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();
    let canonical = payload
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let degraded = payload
        .get("degraded")
        .and_then(Value::as_bool)
        .unwrap_or(!canonical);
    let failure_class = if status == "validation_rejected" {
        json!("validation_rejected")
    } else if status == "degraded" || degraded {
        json!("scope_mismatch")
    } else {
        Value::Null
    };
    let ok = failure_class.is_null();
    let next_tools = payload
        .get("next_tools")
        .cloned()
        .unwrap_or_else(|| json!([]));
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "details".to_string(),
            json!({
                "tool_result_v1": {
                    "ok": ok,
                    "status": status,
                    "canonical": canonical,
                    "degraded": degraded,
                    "failure_class": failure_class,
                    "retry": {"safe": ok, "posture": if ok { "safe_retry" } else { "do_not_retry_unchanged" }},
                    "side_effects": side_effects,
                    "evidence_refs": evidence_refs,
                    "next_tools": next_tools,
                }
            }),
        );
    }
    payload
}

async fn view(
    Query(query): Query<TrajectoryViewQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(attach_trajectory_tool_result(
        trajectory_view_payload(&focusa, &query),
        vec![],
        vec![],
    ))
}

async fn define_goal(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryDefineGoalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;
    let mut payload = define_goal_payload(&focusa, &body);
    let trajectory_record = trajectory_record_from_define_payload(&payload, &body);
    drop(focusa);
    let mut side_effects = Vec::new();
    if let Some(trajectory) = trajectory_record {
        dispatch_event(&state, FocusaEvent::TrajectoryGoalDefined { trajectory }).await?;
        side_effects.push("trajectory_goal_defined");
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("persisted".to_string(), Value::Bool(true));
            obj.insert("mutates_canonical_state".to_string(), Value::Bool(true));
            obj.insert("canonical".to_string(), Value::Bool(true));
            obj.insert(
                "persistence_event".to_string(),
                json!("trajectory_goal_defined"),
            );
        }
    }
    let evidence_refs = body.supersession_evidence_refs.clone().unwrap_or_default();
    Ok(Json(attach_trajectory_tool_result(
        payload,
        side_effects,
        evidence_refs,
    )))
}

async fn assess(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryAssessRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;
    let payload = assess_payload(&focusa, &body);
    let trajectory_id = payload
        .pointer("/trajectory/trajectory_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    drop(focusa);
    let mut side_effects = Vec::new();
    if let Some(trajectory_id) = trajectory_id
        && (body.observed_state.is_some()
            || body
                .evidence_refs
                .as_ref()
                .is_some_and(|refs| !refs.is_empty()))
    {
        dispatch_event(
            &state,
            FocusaEvent::TrajectoryStateDeltaRecorded {
                trajectory_id,
                current_state: body.observed_state.clone(),
                evidence_refs: body.evidence_refs.clone().unwrap_or_default(),
                reason: "trajectory_assess".to_string(),
            },
        )
        .await?;
        side_effects.push("trajectory_state_delta_recorded");
    }
    Ok(Json(attach_trajectory_tool_result(
        payload,
        side_effects,
        body.evidence_refs.clone().unwrap_or_default(),
    )))
}

async fn propose_workpoint(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryProposeWorkpointRequest>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(attach_trajectory_tool_result(
        propose_workpoint_payload(&focusa, &body),
        vec![],
        vec![],
    ))
}

async fn checkpoint(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryCheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = state.focusa.read().await;
    let mut payload = checkpoint_payload(&focusa, &body);
    let trajectory_id = payload
        .pointer("/trajectory_checkpoint/trajectory_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| "trajectory:unknown".to_string());
    drop(focusa);
    dispatch_event(
        &state,
        FocusaEvent::TrajectoryCheckpointPersisted {
            trajectory_id,
            checkpoint: payload
                .get("trajectory_checkpoint")
                .cloned()
                .unwrap_or(Value::Null),
            summary: body.summary.clone(),
        },
    )
    .await?;
    let mut side_effects = Vec::new();
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("persisted".to_string(), Value::Bool(true));
        obj.insert("canonical".to_string(), Value::Bool(true));
        obj.insert(
            "persistence_event".to_string(),
            json!("trajectory_checkpoint_persisted"),
        );
    }
    side_effects.push("trajectory_checkpoint_persisted");
    Ok(Json(attach_trajectory_tool_result(
        payload,
        side_effects,
        vec![],
    )))
}

async fn resume(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryResumeRequest>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(attach_trajectory_tool_result(
        resume_payload(&focusa, &body),
        vec![],
        vec![],
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/trajectory/view", get(view))
        .route("/v1/trajectory/define-goal", post(define_goal))
        .route("/v1/trajectory/assess", post(assess))
        .route("/v1/trajectory/propose-workpoint", post(propose_workpoint))
        .route("/v1/trajectory/checkpoint", post(checkpoint))
        .route("/v1/trajectory/resume", post(resume))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use focusa_core::types::{
        CompletionReason, FocusState, FrameStats, FrameStatus, WorkpointActionIntentRecord,
        WorkpointCheckpointReason, WorkpointConfidence, WorkpointRecord, WorkpointStatus,
    };
    use uuid::Uuid;

    fn state_with_workpoint(project_root: &str) -> FocusaState {
        let workpoint_id = Uuid::now_v7();
        let mut state = FocusaState::default();
        state.workpoint.active_workpoint_id = Some(workpoint_id);
        state.workpoint.records.push(WorkpointRecord {
            workpoint_id,
            work_item_id: Some("focusa-test".to_string()),
            session_id: Some("session-a".to_string()),
            continuity_id: Some("cont-a".to_string()),
            project_root: Some(project_root.to_string()),
            status: WorkpointStatus::Active,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::Verified,
            canonical: true,
            mission: Some("Make trajectory project scoped".to_string()),
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "patch_trajectory_view".to_string(),
                target_ref: Some("crates/focusa-api/src/routes/trajectory.rs".to_string()),
                verification_hooks: vec!["cargo test".to_string()],
                status: Some("ready".to_string()),
            }),
            next_slice: Some("Implement hot-path trajectory view".to_string()),
            ..WorkpointRecord::default()
        });
        state
    }

    fn add_active_frame(
        state: &mut FocusaState,
        project_root: &str,
        continuity_id: &str,
        title: &str,
    ) {
        let frame_id = Uuid::now_v7();
        state.focus_stack.active_id = Some(frame_id);
        state.focus_stack.root_id = Some(frame_id);
        state.focus_stack.stack_path_cache = vec![frame_id];
        state.focus_stack.frames.push(FrameRecord {
            id: frame_id,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FrameStatus::Active,
            title: title.to_string(),
            goal: title.to_string(),
            beads_issue_id: format!("issue-{title}"),
            project_root: Some(project_root.to_string()),
            continuity_id: Some(continuity_id.to_string()),
            tags: vec![],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: FocusState::default(),
            completed_at: None,
            completion_reason: None::<CompletionReason>,
        });
    }

    fn add_defined_trajectory(state: &mut FocusaState, project_root: &str, continuity_id: &str) {
        let body = TrajectoryDefineGoalRequest {
            long_term_goal: "Ship the project north star".to_string(),
            desired_end_state: "Project desired end state verified".to_string(),
            short_term_goal: Some("Use scoped short-term work".to_string()),
            current_state: Some("Project current state verified".to_string()),
            goal_source: Some("operator".to_string()),
            project_root: Some(project_root.to_string()),
            continuity_id: Some(continuity_id.to_string()),
            operator_confirmed: Some(true),
            ..TrajectoryDefineGoalRequest::default()
        };
        let payload = define_goal_payload(state, &body);
        let record =
            trajectory_record_from_define_payload(&payload, &body).expect("valid test trajectory");
        state.trajectory.active_trajectory_id = Some(record.trajectory_id.clone());
        state.trajectory.records.push(record);
    }

    #[test]
    fn trajectory_view_is_project_scoped_and_bounded() {
        let mut state = state_with_workpoint("/repo/focusa");
        add_defined_trajectory(&mut state, "/repo/focusa", "cont-a");
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: None,
                mode: None,
            },
        );
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["canonical"].as_bool(), Some(true));
        assert_eq!(
            payload["project_identity"]["status"].as_str(),
            Some("verified")
        );
        assert_eq!(
            payload["intelligence_view"]["next_workpoint_candidate"]["advisory_only"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn trajectory_view_does_not_promote_workpoint_to_long_term_goal() {
        let state = state_with_workpoint("/repo/focusa");
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                continuity_id: Some("cont-a".to_string()),
                session_id: None,
                mode: None,
            },
        );
        assert_eq!(payload["status"].as_str(), Some("not_found"));
        assert_eq!(payload["trajectory"]["long_term_goal"].as_str(), None);
        assert_eq!(payload["trajectory"]["desired_end_state"].as_str(), None);
        assert_eq!(
            payload["trajectory"]["short_term_goal"].as_str(),
            Some("Implement hot-path trajectory view")
        );
        assert_eq!(
            payload["intelligence_view"]["context_sufficiency"]["proceed_posture"].as_str(),
            Some("operator_required")
        );
        assert!(
            payload["intelligence_view"]["ask_operator_if"]
                .as_array()
                .unwrap()
                .contains(&json!("confirm the project long-term goal"))
        );
        assert!(
            payload["intelligence_view"]["relevance_rationale"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["ref"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("workpoint:"))
        );
    }

    #[test]
    fn trajectory_view_prefers_scoped_workpoint_over_stale_global_active() {
        let mut state = state_with_workpoint("/repo/focusa");
        let scoped_id = Uuid::now_v7();
        state.workpoint.records.push(WorkpointRecord {
            workpoint_id: scoped_id,
            work_item_id: Some("focusa-scoped".to_string()),
            session_id: Some("session-b".to_string()),
            continuity_id: Some("cont-b".to_string()),
            project_root: Some("/repo/focusa".to_string()),
            status: WorkpointStatus::Active,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::Verified,
            canonical: true,
            mission: Some("Use scoped trajectory workpoint".to_string()),
            next_slice: Some("Keep trajectory view canonical".to_string()),
            ..WorkpointRecord::default()
        });
        add_defined_trajectory(&mut state, "/repo/focusa", "cont-b");

        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-after-compact".to_string()),
                continuity_id: Some("cont-b".to_string()),
                mode: None,
            },
        );

        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["canonical"].as_bool(), Some(true));
        assert_eq!(
            payload["project_identity"]["mismatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        let scoped_id_text = scoped_id.to_string();
        assert_eq!(
            payload["intelligence_view"]["next_workpoint_candidate"]["workpoint_id"].as_str(),
            Some(scoped_id_text.as_str())
        );
    }

    #[test]
    fn trajectory_view_ignores_global_workpoint_and_frame_for_explicit_project_scope() {
        let mut state = state_with_workpoint("/repo/focusa");
        add_active_frame(
            &mut state,
            "/repo/focusa",
            "cont-a",
            "global focusa frame must not leak",
        );
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/other".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: None,
                mode: None,
            },
        );
        assert_eq!(payload["status"].as_str(), Some("not_found"));
        assert_eq!(payload["canonical"].as_bool(), Some(false));
        assert_eq!(payload["degraded"].as_bool(), Some(false));
        assert_eq!(
            payload["project_identity"]["status"].as_str(),
            Some("verified")
        );
        assert_eq!(
            payload["project_identity"]["mismatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert!(
            payload["intelligence_view"]["do_not_use"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_ne!(
            payload["trajectory"]["current_state"].as_str(),
            Some("global focusa frame must not leak")
        );
    }

    #[test]
    fn trajectory_define_goal_sets_goal_state_binding_visible_to_view() {
        let mut state = FocusaState::default();
        let body = TrajectoryDefineGoalRequest {
            long_term_goal: "Ship the Workbench product spine".to_string(),
            desired_end_state: "Workbench is usable end to end with verified digest output"
                .to_string(),
            short_term_goal: Some("Verify trajectory set/read contract".to_string()),
            current_state: Some("Trajectory set command received".to_string()),
            goal_source: Some("operator".to_string()),
            project_root: Some("/repo/workbench".to_string()),
            continuity_id: Some("cont-workbench".to_string()),
            operator_confirmed: Some(true),
            required_evidence_refs: Some(vec!["evidence:workbench-e2e".to_string()]),
            required_checks: Some(vec!["cargo test -p workbench".to_string()]),
            acceptance_risks: Some(vec!["stale digest output".to_string()]),
            not_done_if: Some(vec!["digest proof missing".to_string()]),
            ..TrajectoryDefineGoalRequest::default()
        };
        let payload = define_goal_payload(&state, &body);
        let record = trajectory_record_from_define_payload(&payload, &body)
            .expect("valid trajectory record");
        state.trajectory.active_trajectory_id = Some(record.trajectory_id.clone());
        state.trajectory.records.push(record);

        let view = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/workbench".to_string()),
                continuity_id: Some("cont-workbench".to_string()),
                mode: None,
                session_id: None,
            },
        );

        assert_eq!(view["status"].as_str(), Some("completed"));
        assert_eq!(view["canonical"].as_bool(), Some(true));
        assert_eq!(
            view["trajectory"]["long_term_goal"].as_str(),
            Some("Ship the Workbench product spine")
        );
        assert_eq!(
            view["trajectory"]["desired_end_state"].as_str(),
            Some("Workbench is usable end to end with verified digest output")
        );
        assert_eq!(
            view["trajectory"]["current_state"].as_str(),
            Some("Trajectory set command received")
        );
        let dod = &view["trajectory"]["durable_lifecycle"]["definition_of_done"];
        assert_eq!(
            dod["desired_end_state"].as_str(),
            Some("Workbench is usable end to end with verified digest output")
        );
        assert_eq!(
            dod["required_evidence_refs"].as_array().unwrap()[0].as_str(),
            Some("evidence:workbench-e2e")
        );
        assert_eq!(
            dod["required_checks"].as_array().unwrap()[0].as_str(),
            Some("cargo test -p workbench")
        );
        assert_eq!(
            dod["acceptance_risks"].as_array().unwrap()[0].as_str(),
            Some("stale digest output")
        );
        assert_eq!(
            dod["not_done_if"].as_array().unwrap()[0].as_str(),
            Some("digest proof missing")
        );
        assert_eq!(
            view["intelligence_view"]["context_sufficiency"]["proceed_posture"].as_str(),
            Some("verify_first")
        );
        assert!(
            view["intelligence_view"]["context_sufficiency"]["stale_refs"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            view["intelligence_view"]["context_sufficiency"]["conflicting_signals"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            view["intelligence_view"]["ask_operator_if"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(view["intelligence_view"]["current_state_delta"].is_object());
        assert!(
            view["intelligence_view"]["learning_refs"]
                .as_array()
                .is_some()
        );
        assert!(
            view["intelligence_view"]["prediction_refs"]
                .as_array()
                .is_some()
        );
        assert!(
            view["intelligence_view"]["relevance_rationale"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["ref"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("trajectory:"))
        );
    }

    #[test]
    fn trajectory_view_treats_session_id_as_metadata_and_missing_continuity_as_not_found() {
        let mut state = state_with_workpoint("/repo/focusa");
        add_defined_trajectory(&mut state, "/repo/focusa", "cont-a");
        let session_changed = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("pi-after-compact".to_string()),
                continuity_id: Some("cont-a".to_string()),
                mode: None,
            },
        );
        assert_eq!(session_changed["status"].as_str(), Some("completed"));
        assert_eq!(session_changed["canonical"].as_bool(), Some(true));
        assert_eq!(
            session_changed["project_identity"]["session_id_policy"].as_str(),
            Some("temporal_metadata_only")
        );

        let continuity_changed = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-b".to_string()),
                mode: None,
            },
        );
        assert_eq!(continuity_changed["status"].as_str(), Some("not_found"));
        assert_eq!(continuity_changed["canonical"].as_bool(), Some(false));
        assert_eq!(continuity_changed["degraded"].as_bool(), Some(false));
        assert_eq!(
            continuity_changed["project_identity"]["mismatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn trajectory_clarity_gate_guides_missing_and_conflicting_states() {
        let unclear = trajectory_clarity_gate_payload(
            "unclear",
            "verified",
            &["long_term_goal", "desired_end_state"],
            0,
            0,
        );
        assert_eq!(unclear["status"].as_str(), Some("unclear"));
        assert_eq!(
            unclear["recommended_action"].as_str(),
            Some("operator_input")
        );

        let conflicted = trajectory_clarity_gate_payload("clear", "mismatch", &[], 1, 2);
        assert_eq!(conflicted["status"].as_str(), Some("conflicted"));
        assert_eq!(
            conflicted["recommended_action"].as_str(),
            Some("verify_first")
        );
        assert!(
            conflicted["source_precedence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str() == Some("operator_confirmed"))
        );
    }

    #[test]
    fn define_goal_supersession_requires_operator_or_durable_evidence() {
        let state = state_with_workpoint("/repo/focusa");
        let rejected = define_goal_payload(
            &state,
            &TrajectoryDefineGoalRequest {
                long_term_goal: "Replace root goal".to_string(),
                desired_end_state: "New desired end state".to_string(),
                goal_source: Some("inferred_context".to_string()),
                supersedes_trajectory_id: Some("trajectory:old".to_string()),
                operator_confirmed: Some(false),
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-a".to_string()),
                ..TrajectoryDefineGoalRequest::default()
            },
        );
        assert_eq!(rejected["status"].as_str(), Some("validation_rejected"));
        assert_eq!(
            rejected["trajectory_candidate"]["definition_status"].as_str(),
            Some("conflicted")
        );
        assert_eq!(
            rejected["trajectory_candidate"]["root_goal_change_allowed"].as_bool(),
            Some(false)
        );

        let accepted = define_goal_payload(
            &state,
            &TrajectoryDefineGoalRequest {
                long_term_goal: "Replace root goal".to_string(),
                desired_end_state: "New desired end state".to_string(),
                goal_source: Some("durable_supersession".to_string()),
                supersedes_trajectory_id: Some("trajectory:old".to_string()),
                supersession_evidence_refs: Some(vec![
                    "evidence:operator-confirmed-doc".to_string(),
                ]),
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-a".to_string()),
                ..TrajectoryDefineGoalRequest::default()
            },
        );
        assert_eq!(accepted["status"].as_str(), Some("completed"));
        assert_eq!(
            accepted["trajectory_candidate"]["definition_status"].as_str(),
            Some("clear")
        );
        assert_eq!(
            accepted["trajectory_candidate"]["root_goal_change_allowed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn define_goal_returns_advisory_candidate_without_canonical_mutation() {
        let state = state_with_workpoint("/repo/focusa");
        let payload = define_goal_payload(
            &state,
            &TrajectoryDefineGoalRequest {
                long_term_goal: "Ship per-project trajectory".to_string(),
                desired_end_state: "All agents receive project trajectory".to_string(),
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                ..TrajectoryDefineGoalRequest::default()
            },
        );
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["canonical"].as_bool(), Some(true));
        assert_eq!(payload["mutates_canonical_state"].as_bool(), Some(true));
        assert_eq!(payload["persisted"].as_bool(), Some(true));
    }

    #[test]
    fn trajectory_view_exposes_durable_lifecycle_history() {
        let mut state = state_with_workpoint("/repo/focusa");
        let trajectory_id = "trajectory:focusa:cont-a:lifecycle".to_string();
        state.trajectory.active_trajectory_id = Some(trajectory_id.clone());
        state.trajectory.records.push(TrajectoryProjectionRecord {
            trajectory_id: trajectory_id.clone(),
            project_root: Some("/repo/focusa".to_string()),
            continuity_id: Some("cont-a".to_string()),
            root_long_term_goal: "Ship Focusa trajectory".to_string(),
            long_term_goal: "Ship Focusa trajectory".to_string(),
            desired_end_state: "Lifecycle queryable".to_string(),
            current_state: Some("Defined".to_string()),
            definition_status: TrajectoryDefinitionStatus::Clear,
            session_clarity_status: TrajectoryDefinitionStatus::Clear,
            confidence: TrajectoryConfidence::High,
            canonical: true,
            goal_provenance: vec![TrajectoryGoalProvenanceRecord {
                field: "long_term_goal".to_string(),
                source: "operator".to_string(),
                source_ref: Some("test".to_string()),
                inferred: false,
                confidence: TrajectoryConfidence::High,
            }],
            milestones: vec![TrajectoryMilestoneRecord {
                milestone_id: "m1".to_string(),
                title: "Expose lifecycle".to_string(),
                desired_state_delta: "history queryable".to_string(),
                current_state_evidence_refs: vec!["evidence:current".to_string()],
                completion_evidence_refs: vec!["evidence:done".to_string()],
                status: TrajectoryMilestoneStatus::Active,
                next_workpoint_candidate: Value::Null,
            }],
            definition_of_done: Some(TrajectoryDefinitionOfDoneRecord {
                criteria: vec!["DOD queryable".to_string()],
                evidence_required: vec!["evidence:done".to_string()],
                verified_evidence_refs: vec!["evidence:current".to_string()],
                status: "in_progress".to_string(),
                desired_end_state: Some("DOD queryable".to_string()),
                required_evidence_refs: vec!["evidence:done".to_string()],
                required_checks: vec!["cargo test".to_string()],
                acceptance_risks: vec!["stale daemon registry".to_string()],
                not_done_if: vec!["live proof is missing".to_string()],
            }),
            ..TrajectoryProjectionRecord::default()
        });
        state
            .trajectory
            .checkpoints
            .push(focusa_core::types::TrajectoryCheckpointRecord {
                trajectory_id: trajectory_id.clone(),
                summary: Some("checkpoint summary".to_string()),
                packet: json!({"ok": true}),
                persisted_at: None,
            });
        state
            .trajectory
            .state_deltas
            .push(focusa_core::types::TrajectoryStateDeltaRecord {
                trajectory_id: trajectory_id.clone(),
                current_state: Some("Defined".to_string()),
                evidence_refs: vec!["evidence:current".to_string()],
                reason: "test".to_string(),
                recorded_at: None,
            });

        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-a".to_string()),
                mode: None,
            },
        );
        let lifecycle = &payload["trajectory"]["durable_lifecycle"];
        assert_eq!(lifecycle["persisted"].as_bool(), Some(true));
        assert_eq!(lifecycle["checkpoint_count"].as_u64(), Some(1));
        assert_eq!(lifecycle["state_delta_count"].as_u64(), Some(1));
        assert_eq!(lifecycle["goal_provenance"].as_array().unwrap().len(), 1);
        assert_eq!(lifecycle["milestones"].as_array().unwrap().len(), 1);
        assert_eq!(lifecycle["checkpoints"].as_array().unwrap().len(), 1);
        assert_eq!(lifecycle["state_deltas"].as_array().unwrap().len(), 1);
        assert_eq!(
            lifecycle["definition_of_done"]["status"].as_str(),
            Some("in_progress")
        );
        assert_eq!(
            lifecycle["definition_of_done"]["required_checks"]
                .as_array()
                .unwrap()[0]
                .as_str(),
            Some("cargo test")
        );
    }

    #[test]
    fn trajectory_similarity_grouping_is_advisory_not_authority() {
        let payload_a = trajectory_similarity_group_payload(
            "/repo/a",
            Some("Build Focusa north star"),
            Some("Implement traversal adapters"),
            Some("Wire lineage path"),
            Some("cont-a"),
        );
        let payload_b = trajectory_similarity_group_payload(
            "/repo/a",
            Some("Build Focusa north star"),
            Some("Implement Workpoint v2"),
            Some("Render packet"),
            Some("cont-b"),
        );
        assert_eq!(
            payload_a.get("high_level_group_key"),
            payload_b.get("high_level_group_key")
        );
        assert_ne!(
            payload_a.get("mid_level_group_key"),
            payload_b.get("mid_level_group_key")
        );
        assert_ne!(
            payload_a.get("continuity_id"),
            payload_b.get("continuity_id")
        );
        assert_eq!(
            payload_a.get("advisory_only").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            payload_a
                .get("must_not_merge_sessions")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn propose_workpoint_candidate_carries_handoff_guards() {
        let state = state_with_workpoint("/repo/focusa");
        let payload = propose_workpoint_payload(
            &state,
            &TrajectoryProposeWorkpointRequest {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-a".to_string()),
                target_ref: Some("crates/focusa-api/src/routes/trajectory.rs".to_string()),
                action_type: Some("patch_trajectory_handoff".to_string()),
                ..TrajectoryProposeWorkpointRequest::default()
            },
        );
        let candidate = &payload["workpoint_candidate"];
        assert_eq!(payload["advisory_only"].as_bool(), Some(true));
        assert_eq!(payload["no_execution_side_effects"].as_bool(), Some(true));
        assert_eq!(
            candidate["candidate_type"].as_str(),
            Some("advisory_workpoint_candidate_v1")
        );
        assert_eq!(candidate["checkpoint_required"].as_bool(), Some(true));
        assert_eq!(
            candidate["handoff_policy"]["canonicalization_tool"].as_str(),
            Some("focusa_workpoint_checkpoint")
        );
        assert!(
            candidate["do_not_drift"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .unwrap_or_default()
                    .contains("Do not call work-loop selection"))
        );
        assert!(
            candidate["action_intent"]["verification_hooks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_str()
                    == Some("confirm project_root+continuity_id before checkpoint"))
        );
    }

    #[test]
    fn propose_workpoint_returns_checkpoint_required_candidate() {
        let state = state_with_workpoint("/repo/focusa");
        let payload = propose_workpoint_payload(
            &state,
            &TrajectoryProposeWorkpointRequest {
                project_root: Some("/repo/focusa".to_string()),
                session_id: Some("session-a".to_string()),
                ..TrajectoryProposeWorkpointRequest::default()
            },
        );
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["advisory_only"].as_bool(), Some(true));
        assert_eq!(
            payload["workpoint_candidate"]["checkpoint_required"].as_bool(),
            Some(true)
        );
    }
}

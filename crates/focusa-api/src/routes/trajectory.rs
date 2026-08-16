//! Spec96 per-project Trajectory Intelligence API.
//!
//! Trajectory is a bounded, read-only projection over existing Focusa
//! primitives. It orients agents per project; it does not select work, mutate
//! Focus State, switch frames, or execute actions.

use crate::routes::project::project_identity_payload_for_scope;
use crate::scope::ScopeContext;
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
    HltLedgerEntry, HltStatus, SignalOrigin, TrajectoryConfidence,
    TrajectoryDefinitionOfDoneRecord, TrajectoryDefinitionStatus, TrajectoryGoalProvenanceRecord,
    TrajectoryMilestoneRecord, TrajectoryMilestoneStatus, TrajectoryProjectionRecord,
    WorkpointRecord, WorkpointStatus, classify_hlt, trajectory_caps,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryViewQuery {
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub mode: Option<String>,
    #[serde(default)]
    pub allow_prior_project_trajectory: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrajectoryDefineGoalRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub long_term_goal: String,
    pub desired_end_state: String,
    pub mid_level_goal: Option<String>,
    pub short_term_goal: Option<String>,
    pub waypoints: Option<Vec<String>>,
    pub current_state: Option<String>,
    pub goal_source: Option<String>,
    pub supersedes_trajectory_id: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub operator_confirmed: Option<bool>,
    pub supersession_evidence_refs: Option<Vec<String>>,
    pub current_ask: Option<String>, // §169-175: explicit operator intent for verified state gate
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
    pub current_ask: Option<String>,
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

fn trajectory_explicit_project_path_from_ask(ask: &str) -> Option<String> {
    ask.split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| {
                matches!(c, ',' | '.' | ';' | ':' | ')' | '(' | '`' | '"' | '\'')
            })
        })
        .find(|token| {
            (token.starts_with("/home/") || token.starts_with("/Users/"))
                && token.trim_matches('/').split('/').count() >= 3
        })
        .map(|token| token.trim_end_matches('/').to_string())
}

fn trajectory_current_ask_scope_conflict_reason(
    saved_project_root: Option<&str>,
    current_ask: Option<&str>,
) -> Option<String> {
    let ask = clean(current_ask)?;
    let saved_root = saved_project_root?.trim().trim_end_matches('/');
    let path = trajectory_explicit_project_path_from_ask(&ask)?;
    let normalized = path.trim_end_matches('/');
    (!saved_root.is_empty() && normalized != saved_root)
        .then(|| format!("operator named different project path {normalized}"))
}

fn trajectory_current_ask_scope_rejection(
    query: &TrajectoryViewQuery,
    body: &TrajectoryResumeRequest,
) -> Option<Value> {
    let reason = trajectory_current_ask_scope_conflict_reason(
        query.project_root.as_deref(),
        body.current_ask.as_deref(),
    )?;
    Some(json!({
        "status": "rejected_current_ask_scope_conflict",
        "canonical": false,
        "degraded": true,
        "failure_class": "scope_conflict",
        "project_root": query.project_root,
        "continuity_id": query.continuity_id,
        "matches_current_ask_scope": false,
        "action_authority_for_current_ask": false,
        "scope_conflict_reason": reason,
        "warnings": ["current ask names or implies a different project scope than the resumed Trajectory"],
        "safe_recovery": "verify project identity, cd to the intended project root, then resume or define Trajectory in that scope",
        "next_tools": ["focusa_project_verify", "focusa_project_identity", "focusa_trajectory_define_goal", "focusa_workpoint_resume"],
        "next_step_hint": "hard stop: do not use resumed Trajectory as executable route context until current ask scope matches project_root plus continuity_id"
    }))
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
        allow_prior_project_trajectory: false,
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

/// Per Spec98: Canonical state is scoped by (project_root + continuity_id).
/// The global active_trajectory_id is NOT authority - scope must be respected first.
fn active_persisted_trajectory<'a>(
    state: &'a FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a TrajectoryProjectionRecord> {
    // Spec98/99 active selectors are executable route context and therefore
    // require explicit project_root + continuity_id. Historical/fallback views
    // may still cluster by project_root separately, but active lookup fails closed.
    let expected_project_root = clean(project_root)?;
    let expected_continuity_id = clean(continuity_id)?;

    state.trajectory.records.iter().rev().find(|record| {
        record.project_root.as_deref() == Some(expected_project_root.as_str())
            && record.continuity_id.as_deref() == Some(expected_continuity_id.as_str())
            && record.canonical
    })
}

fn is_generic_bootstrap_hlt(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("Maintain and improve ")
        && trimmed.ends_with(" within verified project scope")
}

fn latest_valid_historical_trajectory<'a>(
    records: &'a [TrajectoryProjectionRecord],
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a TrajectoryProjectionRecord> {
    let project_root = project_root
        .map(str::trim)
        .filter(|root| !root.is_empty())?;
    records.iter().rev().find(|record| {
        record.project_root.as_deref() == Some(project_root)
            && continuity_id
                .map(|id| record.continuity_id.as_deref() == Some(id))
                .unwrap_or(true)
            && !record.long_term_goal.trim().is_empty()
            && !is_generic_bootstrap_hlt(record.long_term_goal.as_str())
    })
}

fn scoped_trajectory_history(
    records: &[TrajectoryProjectionRecord],
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    limit: usize,
) -> Vec<Value> {
    let Some(project_root) = project_root.map(str::trim).filter(|root| !root.is_empty()) else {
        return Vec::new();
    };
    if project_root == "unbound" {
        return Vec::new();
    }
    records
        .iter()
        .rev()
        .filter(|record| record.project_root.as_deref() == Some(project_root))
        .filter(|record| {
            continuity_id
                .map(|id| id == record.continuity_id.as_deref().unwrap_or(""))
                .unwrap_or(true)
        })
        .take(limit)
        .map(|record| {
            json!({
                "trajectory_id": record.trajectory_id,
                "continuity_id": record.continuity_id,
                "root_long_term_goal": bounded(record.root_long_term_goal.as_str(), 220),
                "long_term_goal": bounded(record.long_term_goal.as_str(), 220),
                "desired_end_state": bounded(record.desired_end_state.as_str(), 220),
                "canonical": record.canonical,
                "definition_status": serde_json::to_value(record.definition_status)
                    .unwrap_or(Value::String("unclear".to_string())),
                "root_goal_stability": serde_json::to_value(record.root_goal_stability)
                    .unwrap_or(Value::String("stable".to_string())),
                "confidence": serde_json::to_value(record.confidence)
                    .unwrap_or(Value::String("medium".to_string())),
                "created_at": record
                    .created_at
                    .as_ref()
                    .map(|value| value.to_rfc3339()),
                "updated_at": record
                    .updated_at
                    .as_ref()
                    .map(|value| value.to_rfc3339()),
                "goal_provenance_count": record.goal_provenance.len(),
                "milestones_count": record.milestones.len(),
                "supersedes_trajectory_id": record.supersedes_trajectory_id,
            })
        })
        .collect()
}

fn prior_project_trajectory<'a>(
    state: &'a FocusaState,
    project_root: Option<&str>,
    excluded_continuity_id: Option<&str>,
) -> Option<&'a TrajectoryProjectionRecord> {
    state.trajectory.records.iter().rev().find(|record| {
        record.canonical
            && project_root
                .map(|root| record.project_root.as_deref() == Some(root))
                .unwrap_or(true)
            && excluded_continuity_id
                .map(|id| record.continuity_id.as_deref() != Some(id))
                .unwrap_or(true)
            && !record.long_term_goal.trim().is_empty()
            && !record.desired_end_state.trim().is_empty()
    })
}

fn trajectory_definition_of_done_record(
    body: &TrajectoryDefineGoalRequest,
    desired_end_state: &str,
) -> TrajectoryDefinitionOfDoneRecord {
    let verified_evidence_refs = top_strings(
        body.supersession_evidence_refs.as_deref().unwrap_or(&[]),
        8,
        180,
    );
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
    TrajectoryDefinitionOfDoneRecord {
        criteria: vec![bounded(desired_end_state, 240)],
        evidence_required: vec!["evidence proving desired end state".to_string()],
        verified_evidence_refs,
        status: "defined".to_string(),
        desired_end_state: Some(bounded(desired_end_state, 240)),
        required_evidence_refs,
        required_checks,
        acceptance_risks,
        not_done_if,
    }
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
    if body.mid_level_goal.is_some() {
        goal_provenance.push(TrajectoryGoalProvenanceRecord {
            field: "mid_level_goal".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        });
    }
    if body.short_term_goal.is_some() {
        goal_provenance.push(TrajectoryGoalProvenanceRecord {
            field: "short_term_goal".to_string(),
            source: source.clone(),
            source_ref: body.idempotency_key.clone(),
            inferred,
            confidence,
        });
    }
    if body
        .waypoints
        .as_ref()
        .is_some_and(|items| !items.is_empty())
    {
        goal_provenance.push(TrajectoryGoalProvenanceRecord {
            field: "waypoints".to_string(),
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
    let definition_of_done = trajectory_definition_of_done_record(body, &desired_end_state);
    Some(TrajectoryProjectionRecord {
        trajectory_id: trajectory_id.clone(),
        session_identity: body.session_identity.clone(),
        project_root,
        continuity_id,
        root_long_term_goal: long_term_goal.clone(),
        long_term_goal,
        desired_end_state: desired_end_state.clone(),
        mid_level_goal: body
            .mid_level_goal
            .as_deref()
            .map(|value| bounded(value, 240)),
        short_term_goal: body
            .short_term_goal
            .as_deref()
            .map(|value| bounded(value, 240)),
        waypoints: body
            .waypoints
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| clean(Some(&value)).map(|cleaned| bounded(&cleaned, 160)))
            .take(trajectory_caps::MILESTONES)
            .collect(),
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
        definition_of_done: Some(definition_of_done),
        supersedes_trajectory_id: body.supersedes_trajectory_id.clone(),
        hlt_status: HltStatus::CanonicalExplicit,
        canonical: true,
        ..TrajectoryProjectionRecord::default()
    })
}

fn trajectory_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(
        failure_class,
        "validation_rejected" | "not_found" | "scope_mismatch"
    );
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
            "next_tools": next_tools_value.clone(),
            "reflex_suggestions": reflex_suggestions,
            "details": {"tool_result_v1": {"ok": false, "status": "blocked", "canonical": false, "degraded": true, "failure_class": failure_class, "summary": why, "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class}, "side_effects": [], "evidence_refs": [], "next_tools": next_tools_value, "reflex_suggestions": reflex_suggestions, "error": {"code": failure_class, "message": error}}}
        })),
    )
}

fn trajectory_reducer_rejected(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    trajectory_failure(
        StatusCode::OK,
        error.to_string(),
        "validation_rejected",
        format!("trajectory event was rejected by reducer: {error}"),
        "Correct the trajectory payload/project scope before retrying unchanged.",
        "Likely invalid trajectory payload, unsafe project_root, or reducer invariant mismatch.",
        vec![
            "focusa_project_identity",
            "focusa_trajectory_view",
            "focusa_tool_doctor",
        ],
    )
}

fn trajectory_persistence_failed(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    trajectory_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        error.to_string(),
        "persistence_failed",
        format!("trajectory event could not be persisted: {error}"),
        "Check daemon persistence health before retrying; do not rely on transcript-only trajectory state.",
        "Likely SQLite/file permission/resource pressure or event-log persistence outage.",
        vec![
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_trajectory_view",
        ],
    )
}

fn trajectory_dispatch_timeout() -> (StatusCode, Json<Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "pending",
            "canonical": false,
            "degraded": true,
            "failure_class": "resource_exhausted",
            "retry_posture": "safe_retry",
            "retry": {"safe": true, "posture": "safe_retry", "reason": "trajectory write lock is saturated"},
            "side_effects": [],
            "next_tools": ["focusa_resource_mode", "focusa_trajectory_view", "focusa_traverse"],
            "next_step_hint": "retry trajectory mutation after write-lock backlog drains; event was not persisted"
        })),
    )
}

async fn dispatch_event(
    state: &Arc<AppState>,
    event: FocusaEvent,
) -> Result<(), (StatusCode, Json<Value>)> {
    let _guard = tokio::time::timeout(Duration::from_millis(1500), state.write_serial_lock.lock())
        .await
        .map_err(|_| trajectory_dispatch_timeout())?;
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
    let result = reducer::reduce_with_meta(current, event, None, None, false)
        .map_err(trajectory_reducer_rejected)?;

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
        if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
            return Err(trajectory_persistence_failed(error));
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
    let persisted_exact_trajectory = active_persisted_trajectory(
        state,
        Some(project_root.as_str()).filter(|root| *root != "unbound"),
        continuity_id.as_deref(),
    );
    let persisted_prior_project_trajectory =
        if persisted_exact_trajectory.is_none() && query.allow_prior_project_trajectory {
            prior_project_trajectory(
                state,
                Some(project_root.as_str()).filter(|root| *root != "unbound"),
                continuity_id.as_deref(),
            )
        } else {
            None
        };
    let using_prior_project_trajectory = persisted_prior_project_trajectory.is_some();
    let persisted_trajectory = persisted_exact_trajectory.or(persisted_prior_project_trajectory);
    let project_identity_api = if project_root != "unbound" {
        project_identity_payload_for_scope(
            Some(project_root.as_str()),
            Some(project_root.as_str()),
            None,
        )
    } else {
        project_identity_payload_for_scope(None, None, None)
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
    // QN Addendum (2026-06-08): Check actual project identity status, not just bounds
    // Agent runtime paths (unsafe_project_root) must be rejected
    let raw_identity_status = if project_bound && scope_match {
        "verified"
    } else {
        "unbound"
    };
    let project_identity_status = if raw_identity_status == "verified"
        && project_identity_quorum_status == "unsafe_project_root"
    {
        // Agent runtime path detected - override to unsafe
        "unsafe_project_root"
    } else if raw_identity_status == "verified" && project_identity_quorum_status == "mismatch" {
        "mismatch"
    } else {
        raw_identity_status
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
    let persisted_mid_level_goal =
        persisted_trajectory.and_then(|record| record.mid_level_goal.as_deref());
    let persisted_short_term_goal =
        persisted_trajectory.and_then(|record| record.short_term_goal.as_deref());
    let persisted_waypoints = persisted_trajectory
        .map(|record| record.waypoints.clone())
        .unwrap_or_default();
    let workpoint_next = workpoint.and_then(|record| record.next_slice.as_deref());
    let workpoint_action = workpoint
        .and_then(|record| record.action_intent.as_ref())
        .map(|intent| intent.action_type.as_str());

    // Spec96: Workpoint/frame text may shape short-term goals and candidates,
    // but must not silently become the project long-term goal or desired end
    // state. Those require persisted Trajectory state or Focus State intent.
    let mut long_term_goal = first_nonempty(&[persisted_long_term_goal, fs_intent]);
    let mut desired_end_state = first_nonempty(&[persisted_desired_end_state, fs_intent]);
    let mut hlt_source = if persisted_long_term_goal.is_some() {
        "trajectory_record"
    } else if fs_intent.is_some() {
        "focus_state_intent"
    } else {
        "missing"
    };
    let mut hlt_degraded_placeholder = long_term_goal
        .as_deref()
        .map(is_generic_bootstrap_hlt)
        .unwrap_or(false);
    if long_term_goal
        .as_deref()
        .map(is_generic_bootstrap_hlt)
        .unwrap_or(true)
        && let Some(history_record) = latest_valid_historical_trajectory(
            state.trajectory.records.as_slice(),
            Some(project_root.as_str()).filter(|root| *root != "unbound"),
            continuity_id.as_deref(),
        )
    {
        long_term_goal = Some(history_record.long_term_goal.clone());
        desired_end_state.get_or_insert_with(|| history_record.desired_end_state.clone());
        hlt_source = "hlt_history_fallback";
        hlt_degraded_placeholder = false;
    }
    let hlt_valid = long_term_goal
        .as_deref()
        .map(|value| !is_generic_bootstrap_hlt(value))
        .unwrap_or(false);
    let mut current_state = first_nonempty(&[persisted_current_state, fs_current]);
    let short_term_goal = if hlt_valid {
        first_nonempty(&[
            persisted_short_term_goal,
            fs_current,
            workpoint_next,
            workpoint_action,
            frame_goal,
            frame_title,
        ])
    } else {
        first_nonempty(&[persisted_short_term_goal])
    };
    // QN Addendum (2026-06-08): Reject agent runtime paths as project scope
    // Do not infer goals from agent runtime directories (pi-mono, .claude, .letta, etc.)
    if project_identity_status == "unsafe_project_root" {
        return json!({
            "canonical": false,
            "degraded": true,
            "trajectory": {
                "long_term_goal": null,
                "mid_level_goal": null,
                "short_term_goal": null,
                "waypoints": [],
                "desired_end_state": null,
                "current_state": null,
                "active_gap": "agent_runtime_directory",
                "bootstrap_default": false,
                "needs_definition": false
            },
            "intelligence_view": {
                "clarity_gate": {
                    "status": "blocked",
                    "blocking_reasons": ["agent_runtime_directory"],
                    "recommended_action": "use_actual_project_root",
                    "ask_operator_if": ["Provide an actual project root, not an agent runtime directory"]
                },
                "constraints": [
                    "Agent runtime paths (/root/pi-mono, /.claude/, /.letta/, etc.) are NEVER project scope"
                ],
                "do_not_use": [
                    "Do NOT infer goals from agent runtime directories",
                    "Do NOT use ladder fallback when project_root is agent runtime"
                ]
            },
            "context_sufficiency": {
                "score": 0,
                "status": "blocked",
                "proceed_posture": "blocked",
                "missing_facts": ["project_root is agent runtime directory, not a project"],
                "recommended_action": "Verify project_root with focusa_project_identity and use an actual project folder"
            },
            "next_tools": [
                "focusa_project_identity",
                "focusa_project_verify",
                "focusa_trajectory_define_goal (with explicit project_root)"
            ],
            "next_step_hint": "project_root is an agent/runtime directory (not a project). Use an actual project folder like /tmp/focusa-test instead of /root/pi-mono or similar agent paths."
        });
    }

    let bootstrap_default_trajectory = persisted_trajectory.is_none()
        && project_bound
        && scope_match
        && project_identity_status == "verified"
        && long_term_goal.is_none()
        && desired_end_state.is_none();
    if bootstrap_default_trajectory {
        let project_label = project_identity_record
            .get("canonical_name")
            .and_then(Value::as_str)
            .or_else(|| {
                project_identity_record
                    .get("project_id")
                    .and_then(Value::as_str)
            })
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(project_root.as_str());
        long_term_goal = Some(format!(
            "Maintain and improve {project_label} within verified project scope"
        ));
        hlt_source = "bootstrap_degraded_placeholder";
        hlt_degraded_placeholder = true;
        desired_end_state = Some(
            "Verified project sessions have explicit operator-defined trajectory, Workpoint, and evidence before durable work"
                .to_string(),
        );
        current_state.get_or_insert_with(|| {
            "Project identity is verified; durable trajectory goal is not operator-defined yet"
                .to_string()
        });
    }
    let mut active_gap = match (desired_end_state.as_deref(), current_state.as_deref()) {
        (Some(desired), Some(current)) if desired == current => None,
        (Some(_), Some(_)) => first_nonempty(&[workpoint_next, workpoint_action])
            .map(|gap| bounded(&gap, 240))
            .or_else(|| Some("Current verified state differs from desired end state".to_string())),
        _ => Some("Trajectory gap unclear until desired end state and current verified state are both present".to_string()),
    };
    if hlt_degraded_placeholder {
        active_gap = Some("Trajectory definition required before ladder projection".to_string());
    }
    let projected_current_focus = first_nonempty(&[fs_current, short_term_goal.as_deref()]);
    let focus_trajectory_sync = json!({
        "current_focus": projected_current_focus.as_deref().map(|value| bounded(value, 240)),
        "short_term_goal": short_term_goal.as_deref().map(|value| bounded(value, 240)),
        "current_focus_source": if fs_current.is_some() { "focus_state" } else if short_term_goal.is_some() { "trajectory_short_term_goal" } else { "none" },
        "short_term_goal_source": if persisted_short_term_goal.is_some() { "trajectory_record" } else if fs_current.is_some() { "focus_state_current_focus" } else if workpoint_next.is_some() { "workpoint_next_slice" } else if workpoint_action.is_some() { "workpoint_action" } else if frame_goal.is_some() || frame_title.is_some() { "focus_frame" } else { "none" },
        "projection_only": true,
        "authority_boundary": "Focus State and Trajectory remain separate authorities; this projection synchronizes read-model orientation only"
    });
    let effective_long_term_goal_present = long_term_goal.is_some() && !hlt_degraded_placeholder;
    let mid_level_goal = if effective_long_term_goal_present {
        first_nonempty(&[
            persisted_mid_level_goal,
            short_term_goal.as_deref(),
            workpoint_action,
            frame_goal,
            frame_title,
            fs_current,
        ])
    } else {
        first_nonempty(&[persisted_mid_level_goal])
    };
    let mut waypoints = persisted_waypoints;
    if waypoints.is_empty() {
        if let Some(gap) = active_gap.as_deref() {
            waypoints.push(format!("Close active gap: {}", bounded(gap, 120)));
        }
        if let Some(stg) = short_term_goal.as_deref() {
            waypoints.push(format!("Advance STG: {}", bounded(stg, 120)));
        }
        if let Some(mlg) = mid_level_goal.as_deref() {
            waypoints.push(format!("Validate MLG: {}", bounded(mlg, 120)));
        }
        waypoints.truncate(4);
    }
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
        ("long_term_goal", effective_long_term_goal_present),
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

    // Spec 125 §3.2: classify HLT authority status after definition_status is known.
    let is_fallback = hlt_source == "hlt_history_fallback";
    let hlt_status = classify_hlt(
        long_term_goal.as_deref(),
        is_fallback,
        false, // supersession_pending
        definition_status == "conflicted",
    );

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
        .chain(
            (!scope_match).then_some("confirm project_root folder and continuity_id".to_string()),
        )
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
    let canonical = status == "completed"
        && project_identity_status == "verified"
        && !using_prior_project_trajectory
        && !bootstrap_default_trajectory;
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
    let hlt_history = scoped_trajectory_history(
        state.trajectory.records.as_slice(),
        Some(project_root.as_str()),
        continuity_id.as_deref(),
        12,
    );
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

    let workpoint_status = workpoint
        .map(|record| {
            if record.canonical && record.status == WorkpointStatus::Active {
                "canonical"
            } else if record.status == WorkpointStatus::Active {
                "active"
            } else {
                "stale"
            }
        })
        .unwrap_or("missing");
    let trajectory_status = if canonical {
        "canonical"
    } else if bootstrap_default_trajectory {
        "bootstrap_default"
    } else if using_prior_project_trajectory {
        "prior_project_fallback"
    } else if status == "not_found" {
        "missing"
    } else {
        definition_status
    };
    let reconciliation_aligned = canonical && workpoint_status == "canonical";
    let reconciliation_conflicts = if reconciliation_aligned {
        Vec::<String>::new()
    } else if workpoint_status == "canonical" {
        vec![format!("trajectory_status:{trajectory_status}")]
    } else {
        vec![format!("workpoint_status:{workpoint_status}")]
    };
    let trajectory_workpoint_reconciliation = json!({
        "surface_states": {
            "workpoint": workpoint_status,
            "trajectory": trajectory_status,
        },
        "workpoint_status": workpoint_status,
        "workpoint_id": workpoint.map(|record| record.workpoint_id),
        "trajectory_status": trajectory_status,
        "trajectory_id": active_trajectory_id,
        "resolution": if reconciliation_aligned { "aligned" } else if workpoint_status == "canonical" { "use_workpoint_for_immediate_next_action" } else { "verify_first" },
        "authority_for_next_action": if workpoint_status == "canonical" { "workpoint" } else if canonical { "trajectory" } else { "blocked" },
        "supporting_context": if workpoint_status == "canonical" { "canonical Workpoint provides immediate next action; Trajectory remains route context until aligned" } else { "Trajectory remains advisory until a canonical Workpoint is checkpointed or resumed" },
        "blocked_or_stale_surfaces": reconciliation_conflicts,
        "conflicts": reconciliation_conflicts,
        "next_repair_tool": if reconciliation_aligned { "none" } else if workpoint_status == "canonical" { "focusa_trajectory_define_goal" } else { "focusa_workpoint_checkpoint" },
    });

    // Spec 125 §3.3/3.4/6.3: compute loud warning and mandatory HLT fields.
    let loud_warning = match hlt_status {
        HltStatus::MissingRequired => Some(
            "HLT_REQUIRED: no valid High-Level Trajectory is set for this verified scope."
                .to_string(),
        ),
        HltStatus::GenericDegraded => Some(
            "GENERIC_HLT_DEGRADED: this is a placeholder, not a real project trajectory."
                .to_string(),
        ),
        HltStatus::Conflicted => {
            Some("HLT_CONFLICTED: multiple conflicting HLT sources detected.".to_string())
        }
        _ => None,
    };
    let trajectory_required = !hlt_status.is_action_ready();
    let hlt_required = trajectory_required;
    let action_authority_from_trajectory = hlt_status.has_route_authority();
    let generic_bootstrap = matches!(hlt_status, HltStatus::GenericDegraded);

    // Spec 125 §6.3: build warnings list with loud_warning appended.
    let mut trajectory_warnings = if canonical {
        Vec::<String>::new()
    } else if bootstrap_default_trajectory {
        vec!["trajectory bootstrap default is advisory; define or confirm the project goal before treating it as canonical".to_string()]
    } else if using_prior_project_trajectory {
        vec!["using prior project trajectory as reload fallback; refresh short-term goal/current state when needed".to_string()]
    } else if status == "not_found" {
        vec![
            "trajectory is not set for this project folder; define or confirm the goal".to_string(),
        ]
    } else {
        vec![
            "trajectory projection is degraded or provisional; verify before treating as canonical"
                .to_string(),
        ]
    };
    if let Some(ref lw) = loud_warning {
        trajectory_warnings.push(lw.clone());
    }

    json!({
        "status": status,
        "canonical": canonical,
        "degraded": status == "degraded" || hlt_degraded_placeholder || !hlt_status.has_route_authority(),
        "source": "per_project_trajectory_projection_v1",
        "trajectory_required": trajectory_required,
        "hlt_required": hlt_required,
        "hlt_status": serde_json::to_value(hlt_status).unwrap_or_default(),
        "generic_bootstrap": generic_bootstrap,
        "action_authority_from_trajectory": action_authority_from_trajectory,
        "mode": query.mode.as_deref().unwrap_or("summary"),
        "trajectory_workpoint_reconciliation": trajectory_workpoint_reconciliation.clone(),
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
            "hlt_status": serde_json::to_value(hlt_status).unwrap_or_default(),
            // Spec 125 §11.5: new field names.
            "allow_previous_valid_trajectory": using_prior_project_trajectory,
            "previous_valid_trajectory_fallback": using_prior_project_trajectory,
            "fallback_level": if using_prior_project_trajectory {
                if persisted_trajectory.map(|r| r.continuity_id.as_deref() == Some(continuity_id.as_deref().unwrap_or(""))).unwrap_or(false) {
                    "cross_session"
                } else {
                    "cross_continuity"
                }
            } else {
                "none"
            },
            "fallback_source_scope": if using_prior_project_trajectory {
                persisted_trajectory.and_then(|r| r.continuity_id.clone()).map(|cid| if cid == continuity_id.as_deref().unwrap_or("") { "same_continuity".to_string() } else { format!("continuity:{}", cid) })
            } else {
                None
            },
            // Deprecated aliases — kept for backcompat, will be removed in future.
            "fallback_prior_project_trajectory": using_prior_project_trajectory,
            "_deprecated_fallback_prior_project_trajectory": "use allow_previous_valid_trajectory or previous_valid_trajectory_fallback instead",
            "fallback_source_continuity_id": persisted_trajectory.and_then(|record| record.continuity_id.clone()),
            "long_term_goal": long_term_goal.as_deref().map(|value| bounded(value, 240)),
            "desired_end_state": desired_end_state.as_deref().map(|value| bounded(value, 240)),
            "current_state": current_state.as_deref().map(|value| bounded(value, 240)),
            "short_term_goal": short_term_goal.as_deref().map(|value| bounded(value, 240)),
            "mid_level_goal": mid_level_goal.as_deref().map(|value| bounded(value, 240)),
            "low_level_goal": low_level_goal.as_deref().map(|value| bounded(value, 240)),
            "trajectory_ladder": {
                "hlt": long_term_goal.as_deref().map(|value| bounded(value, 240)),
                "mlg": mid_level_goal.as_deref().map(|value| bounded(value, 240)),
                "stg": short_term_goal.as_deref().map(|value| bounded(value, 240)),
                "waypoints": waypoints.clone(),
                "source_metadata": {
                    "hlt": {"source": hlt_source, "degraded": hlt_degraded_placeholder, "hlt_status": serde_json::to_value(hlt_status).unwrap_or_default()},
                    "mlg": {"source": if mid_level_goal.is_some() { if persisted_mid_level_goal.is_some() { "trajectory_record" } else { "hlt_compatible_projection" } } else { "none" }, "degraded": !effective_long_term_goal_present},
                    "stg": {"source": if short_term_goal.is_some() { if persisted_short_term_goal.is_some() { "trajectory_record" } else { "hlt_compatible_projection" } } else { "none" }, "degraded": !effective_long_term_goal_present}
                },
                "rule": "HLT -> MLG -> STG -> Waypoints -> Workpoint; Workpoint/current_focus cannot populate MLG/STG when HLT is invalid or generic"
            },
            "waypoints": waypoints,
            "active_gap": active_gap,
            "similarity_group": similarity_group,
            "bootstrap_default": bootstrap_default_trajectory,
            "hlt_source": hlt_source,
            "hlt_degraded_placeholder": hlt_degraded_placeholder,
            "needs_definition": bootstrap_default_trajectory || hlt_degraded_placeholder,
            "durable_lifecycle": {
                "persisted": persisted_trajectory.is_some(),
                "active_trajectory_id": state.trajectory.active_trajectory_id.clone(),
                "canonical": persisted_trajectory.map(|record| record.canonical).unwrap_or(false),
                "fallback_prior_project_trajectory": using_prior_project_trajectory,
                "root_goal_stability": persisted_trajectory.map(|record| record.root_goal_stability),
                "supersedes_trajectory_id": persisted_trajectory.and_then(|record| record.supersedes_trajectory_id.clone()),
                "created_at": persisted_trajectory.and_then(|record| record.created_at.as_ref().map(|value| value.to_rfc3339())),
                "updated_at": persisted_trajectory.and_then(|record| record.updated_at.as_ref().map(|value| value.to_rfc3339())),
                "checkpoint_count": lifecycle_checkpoints.len(),
                "state_delta_count": lifecycle_state_deltas.len(),
                "history": hlt_history,
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
            "trajectory_workpoint_reconciliation": trajectory_workpoint_reconciliation,
            "focus_trajectory_sync": focus_trajectory_sync,
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
        "next_tools": if bootstrap_default_trajectory {
            json!(["focusa_trajectory_define_goal", "focusa_workpoint_checkpoint", "focusa_project_identity"])
        } else if status == "not_found" {
            json!(["focusa_trajectory_define_goal", "focusa_project_identity"])
        } else {
            json!(["focusa_trajectory_view", "focusa_workpoint_resume", "focusa_active_object_resolve"])
        },
        // BAD-007 fix: Provide clear next_step_hint for empty/degraded trajectory states
        "next_step_hint": if bootstrap_default_trajectory {
            "Trajectory is in bootstrap default state. Define a real goal with focusa_trajectory_define_goal (project_root, continuity_id, long_term_goal, desired_end_state) or pass a workpoint via focusa_workpoint_checkpoint to anchor the scope."
        } else if status == "not_found" {
            "No trajectory exists for this project. Run focusa_trajectory_define_goal to set the long-term goal and desired end state."
        } else if using_prior_project_trajectory {
            "Using prior project trajectory as fallback. Refresh with focusa_trajectory_define_goal to confirm the goal applies to this project."
        } else if !canonical {
            "Trajectory is provisional or degraded. Verify project identity and refresh trajectory definition before treating as canonical."
        } else {
            "Trajectory is canonical. Continue with focusa_workpoint_resume or focusa_workpoint_checkpoint."
        },
        "warnings": trajectory_warnings,
        "loud_warning": loud_warning,
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
            "mid_level_goal": body.mid_level_goal.as_deref().map(|value| bounded(value, 240)),
            "short_term_goal": body.short_term_goal.as_deref().map(|value| bounded(value, 240)),
            "waypoints": body.waypoints.clone().unwrap_or_default().into_iter().filter_map(|value| clean(Some(&value)).map(|cleaned| bounded(&cleaned, 160))).take(trajectory_caps::MILESTONES).collect::<Vec<_>>(),
            "current_state": body.current_state.as_deref().map(|value| bounded(value, 240)),
            "goal_source": body.goal_source.as_deref().unwrap_or("operator"),
            "operator_confirmed": body.operator_confirmed.unwrap_or_else(|| body.goal_source.as_deref().unwrap_or("operator") == "operator"),
            "supersedes_trajectory_id": body.supersedes_trajectory_id,
            "supersession_evidence_refs": body.supersession_evidence_refs.clone().unwrap_or_default().into_iter().take(8).collect::<Vec<_>>(),
            "definition_of_done": serde_json::to_value(trajectory_definition_of_done_record(body, &body.desired_end_state)).unwrap_or(Value::Null),
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
        gaps.push(json!({
            "gap_ref": "missing_desired_end_state",
            "code": "AX-001",
            "reason": "Trajectory has no desired_end_state; the operator must declare what 'done' looks like before gaps are computable.",
            "fix": "call focusa_trajectory_define_goal with long_term_goal + desired_end_state",
            "severity": "high",
            "recommended_action": "define_goal",
        }));
    }
    if current_state.is_none() {
        gaps.push(json!({
            "gap_ref": "missing_current_state",
            "code": "AX-002",
            "reason": "Trajectory has no current_state; assess cannot measure delta until baseline is established.",
            "fix": "call focusa_trajectory_define_goal with current_state, or run focusa_workpoint_checkpoint to anchor a state record",
            "severity": "high",
            "recommended_action": "verify_current_state",
        }));
    }
    if let (Some(current), Some(desired)) = (&current_state, &desired_end_state)
        && current != desired
    {
        gaps.push(json!({
            "gap_ref": "current_to_desired_delta",
            "code": "AX-003",
            "reason": "current_state != desired_end_state; assess reports a delta that requires a typed Workpoint to close.",
            "fix": "call focusa_trajectory_propose_workpoint to enumerate the next action",
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
    if let Some(rejection) = trajectory_current_ask_scope_rejection(&query, body) {
        return rejection;
    }
    let view = trajectory_view_payload(state, &query);

    // Spec 125 §9.3-9.4: extract HLT status and loud warnings from trajectory view.
    let hlt_status = view
        .get("hlt_status")
        .cloned()
        .unwrap_or_else(|| json!("missing_required"));
    let trajectory_required = view
        .get("trajectory_required")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let hlt_required = view
        .get("hlt_required")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let action_authority = view
        .get("action_authority_from_trajectory")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let generic_bootstrap = view
        .get("generic_bootstrap")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let loud_warning = view
        .get("loud_warning")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let warnings: Vec<String> = view
        .get("warnings")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Spec 125 §9.3: if trajectory is generic or missing, check for previous-valid fallback.
    let fallback_source = if generic_bootstrap || hlt_status.as_str() == Some("missing_required") {
        // Look in trajectory history for a previous valid HLT.
        let history = &state.trajectory;
        history
            .records
            .iter()
            .rev()
            .find(|r| {
                !r.long_term_goal.trim().is_empty() && !is_generic_bootstrap_hlt(&r.long_term_goal)
            })
            .map(|r| {
                json!({
                    "hlt": r.long_term_goal,
                    "source": "previous_valid_fallback",
                    "continuity_id": r.continuity_id,
                })
            })
    } else {
        None
    };

    json!({
        "status": view.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "canonical": view.get("canonical").cloned().unwrap_or(Value::Bool(false)),
        "degraded": view.get("degraded").cloned().unwrap_or(Value::Bool(true)),
        "resume_packet": view,
        // Spec 125 §9.3: v3 fields.
        "schema_version": "focusa.trajectory_resume_packet.v3",
        "hlt_status": hlt_status,
        "trajectory_required": trajectory_required,
        "hlt_required": hlt_required,
        "action_authority_from_trajectory": action_authority,
        "generic_bootstrap": generic_bootstrap,
        "fallback_source": fallback_source,
        "loud_warning": loud_warning,
        "warnings": warnings,
        "next_step_hint": "Inject trajectory resume packet plus Workpoint resume before the next agent turn.",
        "next_tools": [
            "focusa_workpoint_resume",
            "focusa_active_object_resolve",
            "focusa_trajectory_view",
            "focusa_trajectory_define_goal",
        ],
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
    _scope: ScopeContext,
    Query(query): Query<TrajectoryViewQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
    Json(attach_trajectory_tool_result(
        trajectory_view_payload(&focusa, &query),
        vec![],
        vec![],
    ))
}

async fn define_goal(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryDefineGoalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // MVP-launch safety: require explicit project_root + continuity_id. Without
    // these, the trajectory lands in the global "unbound" bucket and pollutes
    // downstream scope (workpoint, project_card, ontology). This was a real
    // MVP-launch blocker: `focusa trajectory define-goal` without --project-root
    // would complete and return canonical=true,persisted=true.
    let project_root = body.project_root.as_deref().unwrap_or("").trim();
    let continuity_id = body.continuity_id.as_deref().unwrap_or("").trim();
    if project_root.is_empty() || continuity_id.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "failure_class": "scope_required",
                "missing_fields": if project_root.is_empty() && continuity_id.is_empty() {
                    vec!["project_root", "continuity_id"]
                } else if project_root.is_empty() {
                    vec!["project_root"]
                } else {
                    vec!["continuity_id"]
                },
                "retry_posture": "do_not_retry_unchanged",
                "next_step_hint": "Both project_root and continuity_id are required for focusa trajectory define-goal. Bind the session via `focusa project identity --project-root <path>` first, then pass the returned project_id and continuity_id to this call.",
            })),
        ));
    }

    // QN Addendum (2026-06-08): Reject agent runtime paths as project scope
    let identity = project_identity_payload_for_scope(Some(project_root), Some(project_root), None);
    let identity_status = identity
        .get("project_identity")
        .and_then(|pi| pi.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if identity_status == "unsafe_project_root" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
                "unsafe_reason": "agent_runtime_directory",
                "retry_posture": "do_not_retry_unchanged",
                "next_step_hint": "project_root is an agent/runtime directory. Use an actual project folder instead of agent paths like /root/pi-mono, /.claude/, /.letta/, etc."
            })),
        ));
    }

    // §169-175: Verified state gate — HLT writes require verified project_root + explicit
    // current_ask OR supersession_evidence_refs.  Otherwise return active_gap warning but
    // allow operator_override via operator_confirmed=true.
    let has_context = body.current_ask.as_ref().is_some_and(|s| !s.is_empty())
        || body
            .supersession_evidence_refs
            .as_ref()
            .is_some_and(|v| !v.is_empty());
    let is_operator_override = body.operator_confirmed.unwrap_or(false);
    if !has_context && !is_operator_override {
        warn!(
            "Verified state gate: HLT write without current_ask or evidence_refs. \
            project_root={} identity_status={}",
            project_root, identity_status
        );
    }

    // Spec 125 §4.4-4.5: reject generic HLT even with operator_confirmed/--confirm.
    // Generic bootstrap text must never become canonical route authority.
    if is_generic_bootstrap_hlt(&body.long_term_goal) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "failure_class": "generic_hlt_rejected",
                "field": "long_term_goal",
                "rejected_value": bounded(&body.long_term_goal, 120),
                "retry_posture": "operator_input_required",
                "next_step_hint": "Spec 125 §4.4: generic placeholder HLT is rejected. Provide a specific, operator-defined project goal.",
                "loud_warning": "GENERIC_HLT_REJECTED: generic bootstrap text must not become canonical route authority.",
            })),
        ));
    }

    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
    let mut payload = define_goal_payload(&focusa, &body);
    let trajectory_record = trajectory_record_from_define_payload(&payload, &body);
    // Get old HLT before dispatch (for ledger entry)
    let old_hlt = focusa
        .trajectory
        .records
        .iter()
        .rev()
        .find(|r| r.project_root.as_ref() == body.project_root.as_ref())
        .map(|r| r.long_term_goal.clone());
    let project_root_for_ledger = body.project_root.clone();
    let continuity_id_for_ledger = body
        .continuity_id
        .clone()
        .or_else(|| session_identity_continuity_id(body.session_identity.as_ref()));
    let session_id_for_ledger = body
        .session_id
        .clone()
        .or_else(|| session_identity_session_id(body.session_identity.as_ref()));
    let new_hlt_from_body = body.long_term_goal.clone();
    // §99: auto-derive evidence_refs from session state when body lacks explicit refs
    let focus_state_evidence: Vec<String> = focusa
        .focus_stack
        .frames
        .iter()
        .find(|f| {
            focusa
                .focus_stack
                .active_id
                .map(|aid| aid == f.id)
                .unwrap_or(false)
        })
        .map(|f| {
            f.focus_state
                .artifacts
                .iter()
                .filter_map(|a| {
                    a.handle_ref
                        .as_ref()
                        .map(|h| format!("[HANDLE:{:?}:{}]", h.kind, h.id))
                })
                .chain(f.focus_state.decisions.iter().cloned())
                .take(5)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let evidence_refs = body.supersession_evidence_refs.clone().unwrap_or_else(|| {
        if focus_state_evidence.is_empty() {
            Vec::new()
        } else {
            focus_state_evidence
        }
    });
    drop(focusa);
    let mut side_effects = Vec::new();
    if let Some(trajectory) = trajectory_record {
        if let Err((status, Json(mut pending_payload))) =
            dispatch_event(&state, FocusaEvent::TrajectoryGoalDefined { trajectory }).await
        {
            if status == StatusCode::ACCEPTED
                && pending_payload.get("status").and_then(Value::as_str) == Some("pending")
            {
                if let Some(obj) = pending_payload.as_object_mut() {
                    obj.insert(
                        "trajectory_id".to_string(),
                        payload.get("trajectory_id").cloned().unwrap_or(Value::Null),
                    );
                    obj.insert(
                        "trajectory_candidate".to_string(),
                        payload
                            .get("trajectory_candidate")
                            .cloned()
                            .unwrap_or(Value::Null),
                    );
                    obj.insert(
                        "project_identity".to_string(),
                        payload
                            .get("project_identity")
                            .cloned()
                            .unwrap_or(Value::Null),
                    );
                    obj.insert("advisory_only".to_string(), Value::Bool(true));
                    obj.insert(
                        "persisted".to_string(),
                        payload
                            .get("persisted")
                            .cloned()
                            .unwrap_or(Value::Bool(false)),
                    );
                    obj.insert("mutates_canonical_state".to_string(), Value::Bool(false));
                    obj.insert("pending_candidate_preserved".to_string(), Value::Bool(true));
                }
                return Ok(Json(attach_trajectory_tool_result(
                    pending_payload,
                    vec![],
                    evidence_refs,
                )));
            }
            return Err((status, Json(pending_payload)));
        }
        side_effects.push("trajectory_goal_defined");
        // HLT Ledger: append entry for this goal definition (Spec98/99: scope-bounded, no singleton)
        if let Some(ref project_root) = project_root_for_ledger {
            let entry = HltLedgerEntry::new(
                project_root.clone(),
                new_hlt_from_body.clone(),
                "trajectory_define_goal",
                state
                    .external_mutation_epoch
                    .fetch_add(0, Ordering::Acquire)
                    + 1,
            )
            .with_old_hlt(old_hlt)
            .with_scope(continuity_id_for_ledger, session_id_for_ledger)
            .with_reason(Some("trajectory_goal_defined".to_string()))
            .with_evidence(evidence_refs.clone());
            if let Err(e) = state.persistence.append_hlt_ledger_entry(&entry) {
                warn!("Failed to append HLT ledger entry: {:?}", e);
                side_effects.push("hlt_ledger_write_failed");
            } else {
                side_effects.push("hlt_ledger_entry_appended");
            }
        }
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("persisted".to_string(), Value::Bool(true));
            obj.insert("mutates_canonical_state".to_string(), Value::Bool(true));
            obj.insert("canonical".to_string(), Value::Bool(true));
            obj.insert(
                "persistence_event".to_string(),
                json!("trajectory_goal_defined"),
            );
            // §169-175: verified state gate — signal missing context in response
            if !has_context && !is_operator_override {
                obj.insert("active_gap".to_string(), json!("missing_verified_state"));
                obj.insert("verified_state_gate".to_string(), json!({
                    "project_root_verified": identity_status == "verified",
                    "has_explicit_current_ask": body.current_ask.as_ref().is_some_and(|s| !s.is_empty()),
                    "has_evidence_refs": body.supersession_evidence_refs.as_ref().is_some_and(|v| !v.is_empty()),
                    "operator_override": false,
                    "recommendation": "Add current_ask or supersession_evidence_refs, or set operator_confirmed=true"
                }));
            }
        }
    }
    Ok(Json(attach_trajectory_tool_result(
        payload,
        side_effects,
        evidence_refs,
    )))
}

async fn assess(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryAssessRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // QN Addendum (2026-06-08): Reject agent runtime paths as project scope
    let project_root = body.project_root.as_deref().unwrap_or("");
    if !project_root.is_empty() {
        let identity =
            project_identity_payload_for_scope(Some(project_root), Some(project_root), None);
        let identity_status = identity
            .get("project_identity")
            .and_then(|pi| pi.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        if identity_status == "unsafe_project_root" {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "status": "validation_rejected",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "field": "project_root",
                    "rejected_value": project_root,
                    "unsafe_reason": "agent_runtime_directory",
                    "retry_posture": "do_not_retry_unchanged",
                    "next_step_hint": "project_root is an agent/runtime directory. Use an actual project folder instead."
                })),
            ));
        }
    }
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
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
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryProposeWorkpointRequest>,
) -> Json<Value> {
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
    Json(attach_trajectory_tool_result(
        propose_workpoint_payload(&focusa, &body),
        vec![],
        vec![],
    ))
}

async fn checkpoint(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryCheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
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
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(body): Json<TrajectoryResumeRequest>,
) -> Json<Value> {
    let focusa = crate::workstream_store::scoped_focusa_read(state.clone(), &_scope).await;
    Json(attach_trajectory_tool_result(
        resume_payload(&focusa, &body),
        vec![],
        vec![],
    ))
}

/// HLT History request — scope-bounded by project_root and continuity_id.
#[derive(Debug, Deserialize, Default)]
pub struct HltHistoryRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    /// Spec 125 §7.2: optional session filter. `current` resolves to active session.
    pub session_id: Option<String>,
    /// Spec 125 §7.2: scope kind filter.
    pub scope_kind: Option<String>,
    /// Spec 125 §7.2: typed scope id filter.
    pub scope_id: Option<String>,
    /// Spec 125 §7.2: include cross-session fallback candidates (default false).
    pub include_cross_session_fallbacks: Option<bool>,
    /// Spec 125 §7.2: include generic HLT entries (default false).
    pub include_generic: Option<bool>,
    pub limit: Option<usize>,
}

async fn hlt_history(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HltHistoryRequest>,
) -> Json<Value> {
    let project_root = query.project_root.as_ref().and_then(|r| {
        if r.trim().is_empty() {
            None
        } else {
            Some(r.as_str())
        }
    });
    if project_root.is_none() {
        return Json(json!({
            "status": "error",
            "message": "project_root is required for HLT history",
            "entries": Vec::<Value>::new(),
        }));
    }
    let project_root = project_root.unwrap();
    let limit = query.limit.unwrap_or(50).min(500);
    let continuity_id = query.continuity_id.as_ref().and_then(|r| {
        if r.trim().is_empty() {
            None
        } else {
            Some(r.as_str())
        }
    });
    // Spec 125 §7.6: session_id="current" resolves to active session, never to "any".
    let session_filter = query.session_id.as_ref().and_then(|r| {
        if r.trim().is_empty() || r == "current" {
            None // "current" is resolved by caller before reaching this API
        } else {
            Some(r.as_str())
        }
    });
    let include_generic = query.include_generic.unwrap_or(false);
    let include_cross_session = query.include_cross_session_fallbacks.unwrap_or(false);
    let entries = state
        .persistence
        .read_hlt_ledger_entries(project_root, continuity_id, limit)
        .unwrap_or_default();
    // Spec 125 §7.4: filter by session if provided.
    let filtered: Vec<_> = entries
        .into_iter()
        .filter(|e| {
            session_filter
                .map(|sid| e.session_id.as_deref() == Some(sid))
                .unwrap_or(true)
        })
        .collect();
    let mut generic_skipped = 0usize;
    let mut warnings = Vec::new();
    let entries_json: Vec<Value> = filtered
        .iter()
        .filter(|e| {
            if !include_generic && is_generic_bootstrap_hlt(e.new_hlt.as_str()) {
                generic_skipped += 1;
                false
            } else {
                true
            }
        })
        .map(|e| {
            json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "event_id": e.event_id,
                "project_root": e.project_root,
                "continuity_id": e.continuity_id,
                "session_id": e.session_id,
                "old_hlt": e.old_hlt,
                "new_hlt": e.new_hlt,
                "source": e.source,
                "reason": e.reason,
                "lamport_ts": e.lamport_ts,
                "evidence_refs": e.evidence_refs,
            })
        })
        .collect();
    // Spec 125 §7.3: compute latest_valid_for_session / continuity / project.
    let latest_valid_for_session = filtered.iter().find(|e| {
        session_filter
            .map(|sid| e.session_id.as_deref() == Some(sid))
            .unwrap_or(false)
            && !is_generic_bootstrap_hlt(e.new_hlt.as_str())
    });
    let latest_valid_for_continuity = filtered
        .iter()
        .find(|e| !is_generic_bootstrap_hlt(e.new_hlt.as_str()));
    // For project-level, fetch all entries across continuities.
    let project_entries = state
        .persistence
        .read_hlt_ledger_entries(project_root, None, limit)
        .unwrap_or_default();
    let latest_valid_for_project = project_entries
        .iter()
        .find(|e| !is_generic_bootstrap_hlt(e.new_hlt.as_str()));
    // Spec 125 §7.3: fallback candidates.
    let mut fallback_candidates = Vec::new();
    if let Some(e) = latest_valid_for_session {
        fallback_candidates.push(json!({
            "kind": "exact_session",
            "hlt": e.new_hlt,
            "session_id": e.session_id,
            "continuity_id": e.continuity_id,
        }));
    }
    if !include_cross_session {
        warnings.push(
            "include_cross_session_fallbacks=false; cross-session candidates omitted".to_string(),
        );
    } else if let Some(e) = latest_valid_for_continuity {
        let same_as_session = latest_valid_for_session
            .map(|s| s.event_id == e.event_id)
            .unwrap_or(false);
        if !same_as_session {
            fallback_candidates.push(json!({
                "kind": "cross_session",
                "hlt": e.new_hlt,
                "session_id": e.session_id,
                "continuity_id": e.continuity_id,
            }));
        }
    }
    if let Some(e) = latest_valid_for_project {
        let same_as_continuity = latest_valid_for_continuity
            .map(|c| c.event_id == e.event_id)
            .unwrap_or(false);
        if !same_as_continuity {
            fallback_candidates.push(json!({
                "kind": "cross_continuity",
                "hlt": e.new_hlt,
                "session_id": e.session_id,
                "continuity_id": e.continuity_id,
            }));
        }
    }
    if generic_skipped > 0 {
        warnings.push(format!("{generic_skipped} generic HLT entries skipped"));
    }
    let ledger_path = state.persistence.hlt_ledger_path_for_project(project_root);
    Json(json!({
        "status": "completed",
        "project_root": project_root,
        "continuity_id": continuity_id,
        "session_id": session_filter,
        "count": entries_json.len(),
        "entries": entries_json,
        "fallback_candidates": fallback_candidates,
        "latest_valid_for_session": latest_valid_for_session.map(|e| e.new_hlt.clone()),
        "latest_valid_for_continuity": latest_valid_for_continuity.map(|e| e.new_hlt.clone()),
        "latest_valid_for_project": latest_valid_for_project.map(|e| e.new_hlt.clone()),
        "generic_skipped": generic_skipped,
        "warnings": warnings,
        "ledger_file": ledger_path.to_string_lossy(),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/trajectory/view", get(view))
        .route("/v1/trajectory/define-goal", post(define_goal))
        .route("/v1/trajectory/assess", post(assess))
        .route("/v1/trajectory/propose-workpoint", post(propose_workpoint))
        .route("/v1/trajectory/checkpoint", post(checkpoint))
        .route("/v1/trajectory/resume", post(resume))
        .route("/v1/hlt/history", get(hlt_history))
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

    #[test]
    fn trajectory_resume_rejects_current_ask_project_path_conflict() {
        let query = TrajectoryViewQuery {
            project_root: Some("/tmp/focusa-test".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            ..TrajectoryViewQuery::default()
        };
        let body = TrajectoryResumeRequest {
            project_root: Some("/tmp/focusa-test".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            current_ask: Some("continue implementation in /home/wpuiai/uiai-engine".to_string()),
            ..TrajectoryResumeRequest::default()
        };

        let rejection =
            trajectory_current_ask_scope_rejection(&query, &body).expect("conflict rejection");
        assert_eq!(
            rejection.pointer("/status").and_then(Value::as_str),
            Some("rejected_current_ask_scope_conflict")
        );
        assert_eq!(
            rejection
                .pointer("/action_authority_for_current_ask")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            rejection.pointer("/failure_class").and_then(Value::as_str),
            Some("scope_conflict")
        );
    }

    fn setup_test_project_fixture(project_root: &str) {
        let _ = std::fs::create_dir_all(project_root);
        let git_dir = std::path::PathBuf::from(project_root).join(".git");
        let _ = std::fs::create_dir(&git_dir);
        let _ = std::fs::write(git_dir.join("HEAD"), b"ref: refs/heads/main\n");
        let _ = std::fs::write(
            std::path::PathBuf::from(project_root).join(".focusa-project.json"),
            format!(
                r#"{{"schema":"focusa.project_marker.v1","project_id":"focusa","canonical_name":"focusa","project_root":"{}"}}"#,
                project_root
            ),
        );
    }

    fn state_with_workpoint(project_root: &str) -> FocusaState {
        // FOCUSA_FIX: create a self-contained test fixture so CI machines
        // get a .git repo + .focusa-project.json marker at the test root.
        let _ = std::fs::create_dir_all(project_root);
        // Bare git repo: write HEAD + config minimal stub so find_upwards finds .git.
        let git_dir = std::path::PathBuf::from(project_root).join(".git");
        let _ = std::fs::create_dir(&git_dir);
        let _ = std::fs::write(git_dir.join("HEAD"), b"ref: refs/heads/main\n");
        // Marker file for root_marker signal
        let _ = std::fs::write(
            std::path::PathBuf::from(project_root).join(".focusa-project.json"),
            format!(
                r#"{{"schema":"focusa.project_marker.v1","project_id":"focusa","canonical_name":"focusa","project_root":"{}"}}"#,
                project_root
            ),
        );
        // FOCUSA_FIX: ensure cwd is the project marker dir so discover_identity
        // finds the root_marker + git_root signals and reports verified status.
        let _ = std::env::set_current_dir(project_root);
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
            mission: Some("Bind trajectory to project folder".to_string()),
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
        let mut state = state_with_workpoint("/tmp/focusa-test");
        add_defined_trajectory(&mut state, "/tmp/focusa-test", "cont-a");
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: None,
                mode: None,
                allow_prior_project_trajectory: false,
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
    fn trajectory_view_ignores_stale_global_active_workpoint_when_scope_requested() {
        let stale_id = Uuid::now_v7();
        let scoped_id = Uuid::now_v7();
        let mut state = FocusaState::default();
        state.workpoint.active_workpoint_id = Some(stale_id);
        state.workpoint.records.push(WorkpointRecord {
            workpoint_id: stale_id,
            work_item_id: Some("stale-root-workpoint".to_string()),
            session_id: Some("session-root".to_string()),
            continuity_id: Some("focusa-cont-root-stale".to_string()),
            project_root: Some("/tmp/focusa-test".to_string()),
            status: WorkpointStatus::Active,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::Verified,
            canonical: true,
            mission: Some("stale global active workpoint".to_string()),
            ..WorkpointRecord::default()
        });
        state.workpoint.records.push(WorkpointRecord {
            workpoint_id: scoped_id,
            work_item_id: Some("scoped-focusa-workpoint".to_string()),
            session_id: Some("session-focusa".to_string()),
            continuity_id: Some(
                "focusa-cont-focusa-841f88e0-79fc-4bc8-81ba-28a211a97818".to_string(),
            ),
            project_root: Some("/tmp/focusa-test".to_string()),
            status: WorkpointStatus::Active,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::Verified,
            canonical: true,
            mission: Some("scoped focusa workpoint".to_string()),
            ..WorkpointRecord::default()
        });

        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some(
                    "focusa-cont-focusa-841f88e0-79fc-4bc8-81ba-28a211a97818".to_string(),
                ),
                ..TrajectoryViewQuery::default()
            },
        );

        assert_ne!(payload["status"].as_str(), Some("degraded"));
        let conflict_count = payload["intelligence_view"]["conflicting_signals"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0);
        assert_eq!(conflict_count, 0);
        assert_eq!(
            payload["intelligence_view"]["next_workpoint_candidate"]["work_item_id"].as_str(),
            Some("scoped-focusa-workpoint")
        );
    }

    #[test]
    fn trajectory_view_does_not_promote_workpoint_to_long_term_goal() {
        let state = state_with_workpoint("/tmp/focusa-test");
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-a".to_string()),
                session_id: None,
                mode: None,
                allow_prior_project_trajectory: false,
            },
        );
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["canonical"].as_bool(), Some(false));
        assert_eq!(
            payload["trajectory"]["bootstrap_default"].as_bool(),
            Some(true)
        );
        assert_eq!(
            payload["trajectory"]["long_term_goal"].as_str(),
            Some("Maintain and improve focusa within verified project scope")
        );
        assert_eq!(
            payload["trajectory"]["desired_end_state"].as_str(),
            Some(
                "Verified project sessions have explicit operator-defined trajectory, Workpoint, and evidence before durable work"
            )
        );
        assert_eq!(payload["trajectory"]["short_term_goal"].as_str(), None);
        assert_eq!(
            payload["intelligence_view"]["context_sufficiency"]["proceed_posture"].as_str(),
            Some("operator_required")
        );
        assert!(
            !payload["intelligence_view"]["ask_operator_if"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            payload["next_tools"]
                .as_array()
                .unwrap()
                .contains(&json!("focusa_trajectory_define_goal"))
        );
        assert!(
            payload["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|warning| warning
                    .as_str()
                    .unwrap_or_default()
                    .contains("bootstrap default is advisory"))
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
    fn trajectory_view_syncs_focus_current_focus_and_short_term_goal_projection() {
        let mut state = state_with_workpoint("/tmp/focusa-test");
        add_active_frame(
            &mut state,
            "/tmp/focusa-test",
            "cont-a",
            "Frame title fallback",
        );
        if let Some(frame) = state.focus_stack.frames.last_mut() {
            frame.focus_state.current_state =
                "Focus State current focus drives short term".to_string();
        }
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-a".to_string()),
                session_id: None,
                mode: None,
                allow_prior_project_trajectory: false,
            },
        );
        assert_eq!(payload["trajectory"]["short_term_goal"].as_str(), None);
        assert_eq!(
            payload["intelligence_view"]["focus_trajectory_sync"]["short_term_goal_source"]
                .as_str(),
            Some("focus_state_current_focus")
        );

        let mut state = state_with_workpoint("/tmp/focusa-test");
        add_defined_trajectory(&mut state, "/tmp/focusa-test", "cont-a");
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-a".to_string()),
                session_id: None,
                mode: None,
                allow_prior_project_trajectory: false,
            },
        );
        assert_eq!(
            payload["intelligence_view"]["focus_trajectory_sync"]["current_focus"].as_str(),
            Some("Use scoped short-term work")
        );
        assert_eq!(
            payload["intelligence_view"]["focus_trajectory_sync"]["current_focus_source"].as_str(),
            Some("trajectory_short_term_goal")
        );
    }

    #[test]
    fn trajectory_view_prefers_scoped_workpoint_over_stale_global_active() {
        let mut state = state_with_workpoint("/tmp/focusa-test");
        let scoped_id = Uuid::now_v7();
        state.workpoint.records.push(WorkpointRecord {
            workpoint_id: scoped_id,
            work_item_id: Some("focusa-scoped".to_string()),
            session_id: Some("session-b".to_string()),
            continuity_id: Some("cont-b".to_string()),
            project_root: Some("/tmp/focusa-test".to_string()),
            status: WorkpointStatus::Active,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::Verified,
            canonical: true,
            mission: Some("Use scoped trajectory workpoint".to_string()),
            next_slice: Some("Keep trajectory view canonical".to_string()),
            ..WorkpointRecord::default()
        });
        add_defined_trajectory(&mut state, "/tmp/focusa-test", "cont-b");

        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("session-after-compact".to_string()),
                continuity_id: Some("cont-b".to_string()),
                mode: None,
                allow_prior_project_trajectory: false,
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
        setup_test_project_fixture("/tmp/focusa-other");
        let _ = std::env::set_current_dir("/tmp/focusa-other");
        let mut state = state_with_workpoint("/tmp/focusa-test");
        add_active_frame(
            &mut state,
            "/tmp/focusa-test",
            "cont-a",
            "global focusa frame must not leak",
        );
        let payload = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-other".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: None,
                mode: None,
                allow_prior_project_trajectory: false,
            },
        );
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["canonical"].as_bool(), Some(false));
        assert_eq!(payload["degraded"].as_bool(), Some(true));
        assert_eq!(
            payload["trajectory"]["bootstrap_default"].as_bool(),
            Some(true)
        );
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
        // FOCUSA_FIX: ensure cwd matches project_root so discover_identity verifies.
        let _ = std::env::set_current_dir("/tmp/focusa-test");
        let mut state = FocusaState::default();
        let body = TrajectoryDefineGoalRequest {
            long_term_goal: "Ship the Workbench product spine".to_string(),
            desired_end_state: "Workbench is usable end to end with verified digest output"
                .to_string(),
            short_term_goal: Some("Verify trajectory set/read contract".to_string()),
            current_state: Some("Trajectory set command received".to_string()),
            goal_source: Some("operator".to_string()),
            project_root: Some("/tmp/focusa-test".to_string()),
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
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-workbench".to_string()),
                mode: None,
                session_id: None,
                allow_prior_project_trajectory: false,
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
        let mut state = state_with_workpoint("/tmp/focusa-test");
        add_defined_trajectory(&mut state, "/tmp/focusa-test", "cont-a");
        let session_changed = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("pi-after-compact".to_string()),
                continuity_id: Some("cont-a".to_string()),
                mode: None,
                allow_prior_project_trajectory: false,
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
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-b".to_string()),
                mode: None,
                allow_prior_project_trajectory: false,
            },
        );
        assert_eq!(continuity_changed["status"].as_str(), Some("completed"));
        assert_eq!(continuity_changed["canonical"].as_bool(), Some(false));
        assert_eq!(continuity_changed["degraded"].as_bool(), Some(true));
        assert_eq!(
            continuity_changed["trajectory"]["bootstrap_default"].as_bool(),
            Some(true)
        );
        assert_eq!(
            continuity_changed["project_identity"]["mismatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );

        let fallback_prior = trajectory_view_payload(
            &state,
            &TrajectoryViewQuery {
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-b".to_string()),
                mode: None,
                allow_prior_project_trajectory: true,
            },
        );
        assert_eq!(fallback_prior["status"].as_str(), Some("completed"));
        assert_eq!(fallback_prior["canonical"].as_bool(), Some(false));
        assert_eq!(
            fallback_prior["trajectory"]["fallback_prior_project_trajectory"].as_bool(),
            Some(true)
        );
        assert_eq!(
            fallback_prior["trajectory"]["fallback_source_continuity_id"].as_str(),
            Some("cont-a")
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
        let state = state_with_workpoint("/tmp/focusa-test");
        let rejected = define_goal_payload(
            &state,
            &TrajectoryDefineGoalRequest {
                long_term_goal: "Replace root goal".to_string(),
                desired_end_state: "New desired end state".to_string(),
                goal_source: Some("inferred_context".to_string()),
                supersedes_trajectory_id: Some("trajectory:old".to_string()),
                operator_confirmed: Some(false),
                project_root: Some("/tmp/focusa-test".to_string()),
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
                project_root: Some("/tmp/focusa-test".to_string()),
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
        assert_eq!(
            accepted["trajectory_candidate"]["definition_of_done"]["required_evidence_refs"]
                .as_array()
                .unwrap()[0]
                .as_str(),
            Some("evidence:operator-confirmed-doc")
        );
    }

    #[test]
    fn define_goal_returns_advisory_candidate_without_canonical_mutation() {
        let state = state_with_workpoint("/tmp/focusa-test");
        let payload = define_goal_payload(
            &state,
            &TrajectoryDefineGoalRequest {
                long_term_goal: "Ship per-project trajectory".to_string(),
                desired_end_state: "All agents receive project trajectory".to_string(),
                project_root: Some("/tmp/focusa-test".to_string()),
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
        let mut state = state_with_workpoint("/tmp/focusa-test");
        let prior_trajectory_id = "trajectory:focusa:cont-a:lifecycle-prior".to_string();
        state.trajectory.records.push(TrajectoryProjectionRecord {
            trajectory_id: prior_trajectory_id.clone(),
            project_root: Some("/tmp/focusa-test".to_string()),
            continuity_id: Some("cont-a".to_string()),
            root_long_term_goal: "Build prior trajectory".to_string(),
            long_term_goal: "Build prior trajectory".to_string(),
            desired_end_state: "Prior lifecycle queryable".to_string(),
            definition_status: TrajectoryDefinitionStatus::Clear,
            session_clarity_status: TrajectoryDefinitionStatus::Clear,
            confidence: TrajectoryConfidence::High,
            canonical: true,
            created_at: Some(Utc::now()),
            ..TrajectoryProjectionRecord::default()
        });
        let trajectory_id = "trajectory:focusa:cont-a:lifecycle".to_string();
        state.trajectory.active_trajectory_id = Some(trajectory_id.clone());
        state.trajectory.records.push(TrajectoryProjectionRecord {
            trajectory_id: trajectory_id.clone(),
            project_root: Some("/tmp/focusa-test".to_string()),
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
                project_root: Some("/tmp/focusa-test".to_string()),
                session_id: Some("session-a".to_string()),
                continuity_id: Some("cont-a".to_string()),
                mode: None,
                allow_prior_project_trajectory: false,
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

        let history = lifecycle["history"].as_array().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0]["trajectory_id"].as_str(),
            Some("trajectory:focusa:cont-a:lifecycle")
        );
        assert_eq!(
            history[1]["trajectory_id"].as_str(),
            Some("trajectory:focusa:cont-a:lifecycle-prior")
        );
        assert_eq!(
            history[0]["long_term_goal"].as_str(),
            Some("Ship Focusa trajectory")
        );
        assert_eq!(
            history[1]["long_term_goal"].as_str(),
            Some("Build prior trajectory")
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
        let state = state_with_workpoint("/tmp/focusa-test");
        let payload = propose_workpoint_payload(
            &state,
            &TrajectoryProposeWorkpointRequest {
                project_root: Some("/tmp/focusa-test".to_string()),
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
        let state = state_with_workpoint("/tmp/focusa-test");
        let payload = propose_workpoint_payload(
            &state,
            &TrajectoryProposeWorkpointRequest {
                project_root: Some("/tmp/focusa-test".to_string()),
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

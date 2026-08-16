//! Spec88 Workpoint continuity API routes.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, budgeted_default_limit, budgeted_hard_limit,
    lowmem_caps_active, resource_mode_status,
};
use crate::routes::permissions::{forbid, permission_context};
use crate::scope::ScopeContext;
use crate::server::AppState;
use focusa_core::types::FocusaState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::reducer;
use focusa_core::scope_safety::classify_project_root_option;
use focusa_core::types::{
    Action, EventLogEntry, FocusaEvent, FocusaSessionIdentity, SignalOrigin,
    WorkpointActionIntentRecord, WorkpointCheckpointReason, WorkpointConfidence,
    WorkpointDriftSeverity, WorkpointRecord, WorkpointStatus, WorkpointVerificationRecord,
};
use focusa_core::working_subpath::resolve_git_working_context;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointCheckpointRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub working_subpath_id: Option<String>,
    pub workpoint_id: Option<Uuid>,
    pub work_item_id: Option<String>,
    pub continuity_id: Option<String>,
    pub session_id: Option<String>,
    pub project_root: Option<String>,
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
    #[serde(default, alias = "dry_run")]
    pub preview: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointResumeRequest {
    pub session_identity: Option<FocusaSessionIdentity>,
    pub working_subpath_id: Option<String>,
    pub workpoint_id: Option<Uuid>,
    pub mode: Option<String>,
    pub continuity_id: Option<String>,
    pub session_id: Option<String>,
    pub project_root: Option<String>,
    pub work_item_id: Option<String>,
    pub trajectory_id: Option<String>,
    pub current_ask: Option<String>,
    #[serde(default)]
    pub frame_tags: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointRolloverTargetMaterializeRequest {
    pub source_continuity_id: Option<String>,
    pub target_continuity_id: Option<String>,
    pub target_session_id: Option<String>,
    pub source_session_id: Option<String>,
    pub project_root: Option<String>,
    pub working_subpath_id: Option<String>,
    pub checkpoint_ref: Option<String>,
    pub workpoint_packet_ref: Option<String>,
    pub compaction_packet_ref: Option<String>,
}

fn rollover_required_ref(
    req: &WorkpointRolloverTargetMaterializeRequest,
    field: &str,
) -> Option<String> {
    match field {
        "source_continuity_id" => clean_resume_scope_value(req.source_continuity_id.as_deref()),
        "target_continuity_id" => clean_resume_scope_value(req.target_continuity_id.as_deref()),
        "source_session_id" => clean_resume_scope_value(req.source_session_id.as_deref()),
        "target_session_id" => clean_resume_scope_value(req.target_session_id.as_deref()),
        "project_root" => clean_resume_scope_value(req.project_root.as_deref()),
        "checkpoint_ref" => req
            .checkpoint_ref
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        "workpoint_packet_ref" => req
            .workpoint_packet_ref
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        "compaction_packet_ref" => req
            .compaction_packet_ref
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        _ => None,
    }
}

fn rollout_target_materialization_refs(
    source_refs: Vec<String>,
    checkpoint_ref: &str,
    workpoint_packet_ref: &str,
    compaction_packet_ref: &str,
) -> Vec<String> {
    let mut refs = source_refs;
    let mut seen: HashSet<String> = HashSet::new();
    for value in &refs {
        seen.insert(value.clone());
    }
    for value in [
        format!("checkpoint_ref:{checkpoint_ref}"),
        format!("workpoint_packet_ref:{workpoint_packet_ref}"),
        format!("compaction_packet_ref:{compaction_packet_ref}"),
    ] {
        if seen.insert(value.clone()) {
            refs.push(value);
        }
    }
    refs
}

fn rollover_target_materialization_idempotency_key(
    source_continuity_id: &str,
    target_continuity_id: &str,
    source_session_id: &str,
    target_session_id: &str,
    checkpoint_ref: &str,
    workpoint_packet_ref: &str,
    compaction_packet_ref: &str,
) -> String {
    format!(
        "workpoint-target-materialize:{source_continuity_id}:{target_continuity_id}:{source_session_id}:{target_session_id}:{checkpoint_ref}:{workpoint_packet_ref}:{compaction_packet_ref}",
    )
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkpointCurrentQuery {
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
    pub working_subpath_id: Option<String>,
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
    pub session_identity: Option<FocusaSessionIdentity>,
    pub working_subpath_id: Option<String>,
    pub workpoint_id: Option<Uuid>,
    pub target_ref: String,
    pub result: String,
    pub evidence_ref: Option<String>,
    #[serde(default, alias = "dry_run")]
    pub preview: bool,
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

#[derive(Debug, Clone, Default)]
struct ResumeScopeDecision {
    rejection: Option<Value>,
    canonical_scope_ok: bool,
    warnings: Vec<String>,
    expected_working_subpath_id: Option<String>,
    actual_working_subpath_id: Option<String>,
    session_changed: bool,
    expected_session_id: Option<String>,
    packet_session_id: Option<String>,
    expected_continuity_id: Option<String>,
    packet_continuity_id: Option<String>,
}

fn clean_resume_scope_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn session_identity_working_subpath_id(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    let identity = identity?;
    clean_resume_scope_value(identity.working_subpath_id.as_deref()).or_else(|| {
        identity
            .project_identity
            .as_ref()
            .and_then(|project| project.working_context.as_ref())
            .and_then(|context| context.pointer("/working_subpath/working_subpath_id"))
            .and_then(Value::as_str)
            .and_then(|value| clean_resume_scope_value(Some(value)))
    })
}

fn record_working_subpath_id(record: &WorkpointRecord) -> String {
    session_identity_working_subpath_id(record.session_identity.as_ref())
        .unwrap_or_else(|| "primary".to_string())
}

fn session_identity_project_root(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| {
        clean_resume_scope_value(Some(identity.project_root.as_str())).or_else(|| {
            identity
                .project_identity
                .as_ref()
                .and_then(|project| clean_resume_scope_value(Some(project.project_root.as_str())))
        })
    })
}

fn session_identity_continuity_id(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| clean_resume_scope_value(identity.continuity_id.as_deref()))
}

fn session_identity_session_id(identity: Option<&FocusaSessionIdentity>) -> Option<String> {
    identity.and_then(|identity| {
        clean_resume_scope_value(identity.pi_session_id.as_deref())
            .or_else(|| clean_resume_scope_value(Some(identity.session_frame_key.as_str())))
    })
}

fn apply_checkpoint_session_identity(req: &mut WorkpointCheckpointRequest) {
    if req.session_identity.is_none() && req.working_subpath_id.is_some() {
        req.session_identity = Some(FocusaSessionIdentity {
            working_subpath_id: req.working_subpath_id.clone(),
            ..FocusaSessionIdentity::default()
        });
    }
    if let Some(project_root) = session_identity_project_root(req.session_identity.as_ref()) {
        req.project_root = Some(project_root);
    }
    if let Some(continuity_id) = session_identity_continuity_id(req.session_identity.as_ref()) {
        req.continuity_id = Some(continuity_id);
    }
    if let Some(session_id) = session_identity_session_id(req.session_identity.as_ref()) {
        req.session_id = Some(session_id);
    }
}

fn apply_resume_session_identity(req: &mut WorkpointResumeRequest) {
    if req.session_identity.is_none() && req.working_subpath_id.is_some() {
        req.session_identity = Some(FocusaSessionIdentity {
            working_subpath_id: req.working_subpath_id.clone(),
            ..FocusaSessionIdentity::default()
        });
    }
    if let Some(project_root) = session_identity_project_root(req.session_identity.as_ref()) {
        req.project_root = Some(project_root);
    }
    if let Some(continuity_id) = session_identity_continuity_id(req.session_identity.as_ref()) {
        req.continuity_id = Some(continuity_id);
    }
    if let Some(session_id) = session_identity_session_id(req.session_identity.as_ref()) {
        req.session_id = Some(session_id);
    }
}

fn normalize_project_root_authority(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "".to_string()
    } else {
        trimmed.to_string()
    }
}

/// #125 migration pattern: resolve the partitioned workstream state for a
/// scope, falling back to the global state when the scope is unmigrated.
/// Owned guards keep lifetimes simple across both branches.
async fn workstream_scoped_state(
    state: Arc<AppState>,
    scope: &crate::scope::ScopeContext,
) -> tokio::sync::OwnedRwLockReadGuard<FocusaState> {
    match (&scope.project_root, &scope.continuity_id) {
        (Some(root), Some(continuity)) => {
            let partition = state
                .workstream_states
                .get_or_create(root, continuity)
                .await;
            partition.read_owned().await
        }
        _ => state.focusa.clone().read_owned().await,
    }
}

fn unsafe_project_root_reason(value: Option<&str>) -> Option<&'static str> {
    classify_project_root_option(value).reason()
}

struct WrongIdTaxonomy {
    status: &'static str,
    workpoint_id: Option<Uuid>,
    requested_workpoint_id: Option<Uuid>,
    requested_found: bool,
    scope_found: bool,
    fallback_used: bool,
    canonical_for_requested_scope: bool,
    canonical_for_fallback_scope: bool,
}

fn wrong_id_taxonomy_payload(taxonomy: WrongIdTaxonomy) -> Value {
    json!({
        "schema": "focusa.wrong_id_taxonomy.v1",
        "WrongIdConsistency": true,
        "status": taxonomy.status,
        "workpoint_id": taxonomy.workpoint_id,
        "requested_workpoint_id": taxonomy.requested_workpoint_id,
        "requested_found": taxonomy.requested_found,
        "scope_found": taxonomy.scope_found,
        "fallback_used": taxonomy.fallback_used,
        "canonical_for_requested_scope": taxonomy.canonical_for_requested_scope,
        "canonical_for_fallback_scope": taxonomy.canonical_for_fallback_scope,
    })
}

fn unsafe_project_root_rejection(
    record: &WorkpointRecord,
    reason: &'static str,
    expected_project_root: Option<&str>,
) -> Value {
    json!({
        "status": "rejected_unsafe_project_root",
        "canonical": false,
        "failure_class": "scope_mismatch",
        "workpoint_id": record.workpoint_id,
        "warnings": [if reason == "agent_runtime_directory" { "workpoint project_root is an agent/runtime directory, not a project — never treat as project scope" } else { "workpoint project_root is missing or too broad to be an authority boundary" }],
        "unsafe_reason": reason,
        "expected_project_root": expected_project_root,
        "packet_project_root": record.project_root,
        "safe_recovery": if reason == "agent_runtime_directory" { "ignore this resume packet; cd to the actual project/repo and bind Focusa to that root" } else { "ignore this resume packet; bind Focusa to a specific project/repo root and checkpoint a fresh Workpoint" },
        "requested_found": true,
        "scope_found": false,
        "fallback_used": false,
        "canonical_for_requested_scope": false,
        "canonical_for_fallback_scope": false,
        "wrong_id_taxonomy": wrong_id_taxonomy_payload(WrongIdTaxonomy {
            status: "scope_mismatch_for_requested_id",
            workpoint_id: Some(record.workpoint_id),
            requested_workpoint_id: Some(record.workpoint_id),
            requested_found: true,
            scope_found: false,
            fallback_used: false,
            canonical_for_requested_scope: false,
            canonical_for_fallback_scope: false,
        }),
        "next_step_hint": "cd into the exact project/repo or pass an explicit safe project_root before trusting resume"
    })
}

fn unsafe_checkpoint_rejection(
    reason: &'static str,
    field: &'static str,
    value: Option<&str>,
) -> (StatusCode, Json<Value>) {
    let hint = if reason == "agent_runtime_directory" {
        "project_root is an agent/runtime directory, not a project — cd to the actual project/repo"
    } else {
        "provide project_root for the exact project/repo and continuity_id before creating a canonical Workpoint"
    };
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({
            "status": "validation_rejected",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "field": field,
            "rejected_value": value.unwrap_or(""),
            "unsafe_reason": reason,
            "retry_posture": "do_not_retry_unchanged",
            "next_step_hint": hint
        })),
    )
}

fn unconfirmed_project_root_rejection(
    identity: &FocusaSessionIdentity,
) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!({
            "status": "project_root_confirmation_required",
            "canonical": false,
            "failure_class": "scope_mismatch",
            "project_root": &identity.project_root,
            "confidence": identity.project_root_confidence.as_deref(),
            "confidence_score": identity.project_root_confidence_score,
            "resolution_source": identity.project_root_resolution_source.as_deref(),
            "requires_operator_confirmation": identity.requires_operator_confirmation.unwrap_or(true),
            "scope_failure": identity.scope_failure.as_deref(),
            "candidates": &identity.project_root_candidates,
            "retry_posture": "operator_required",
            "next_tools": ["interview", "focusa_project_identity", "focusa_workpoint_checkpoint"],
            "next_step_hint": "ask the operator to confirm the exact project_root before mutating Workpoint/evidence state"
        })),
    )
}

fn session_identity_requires_project_root_confirmation(
    identity: Option<&FocusaSessionIdentity>,
) -> Option<(StatusCode, Json<Value>)> {
    let identity = identity?;
    if identity.canonical_scope == Some(false)
        || identity.scope_failure.is_some()
        || identity.requires_operator_confirmation.unwrap_or(false)
    {
        return Some(unconfirmed_project_root_rejection(identity));
    }
    if identity
        .project_root_confidence_score
        .is_some_and(|score| score < 0.90)
    {
        return Some(unconfirmed_project_root_rejection(identity));
    }
    None
}

fn evaluate_resume_scope(
    record: &WorkpointRecord,
    expected_project_root: Option<&str>,
    expected_continuity_id: Option<&str>,
    expected_session_id: Option<&str>,
    expected_working_subpath_id: Option<&str>,
) -> ResumeScopeDecision {
    let mut decision = ResumeScopeDecision {
        canonical_scope_ok: true,
        ..ResumeScopeDecision::default()
    };
    if let Some(reason) =
        unsafe_project_root_reason(expected_project_root.or(record.project_root.as_deref()))
    {
        decision.canonical_scope_ok = false;
        decision.rejection = Some(unsafe_project_root_rejection(
            record,
            reason,
            expected_project_root,
        ));
        return decision;
    }

    if let Some(expected) = clean_resume_scope_value(expected_project_root) {
        let actual = record.project_root.as_deref().unwrap_or("").trim();
        if actual.is_empty() || actual != expected {
            decision.canonical_scope_ok = false;
            decision.rejection = Some(json!({
                "status": "rejected_scope_mismatch",
                "canonical": false,
                "workpoint_id": record.workpoint_id,
                "warnings": ["workpoint project_root does not match current Pi project/root"],
                "expected_project_root": expected,
                "packet_project_root": actual,
                "safe_recovery": "ignore this resume packet; follow latest operator instruction and local git/beads for the current project",
                "requested_found": true,
                "scope_found": false,
                "fallback_used": false,
                "canonical_for_requested_scope": false,
                "canonical_for_fallback_scope": false,
                "wrong_id_taxonomy": wrong_id_taxonomy_payload(WrongIdTaxonomy {
                    status: "scope_mismatch_for_requested_id",
                    workpoint_id: Some(record.workpoint_id),
                    requested_workpoint_id: Some(record.workpoint_id),
                    requested_found: true,
                    scope_found: false,
                    fallback_used: false,
                    canonical_for_requested_scope: false,
                    canonical_for_fallback_scope: false,
                }),
                "next_step_hint": "create a new Workpoint checkpoint in the current project before trusting resume"
            }));
            return decision;
        }
    } else if record.project_root.is_none() {
        decision.canonical_scope_ok = false;
        decision.warnings.push(
            "resume requested without project_root and packet has no project_root; project folder is unbound"
                .to_string(),
        );
    }

    let actual_working_subpath_id = record_working_subpath_id(record);
    decision.actual_working_subpath_id = Some(actual_working_subpath_id.clone());
    let expected_working_subpath_id = clean_resume_scope_value(expected_working_subpath_id)
        .unwrap_or_else(|| "primary".to_string());
    decision.expected_working_subpath_id = Some(expected_working_subpath_id.clone());
    if actual_working_subpath_id != expected_working_subpath_id {
        decision.canonical_scope_ok = false;
        decision.rejection = Some(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "failure_class": "working_subpath_mismatch",
            "workpoint_id": record.workpoint_id,
            "expected_working_subpath_id": expected_working_subpath_id,
            "actual_working_subpath_id": actual_working_subpath_id,
            "recovery_hint": "resume the Workpoint from its exact working context or explicitly transfer it; uncommitted state is not transferable",
            "next_step_hint": "checkpoint a new Workpoint in the active working subpath or use focusa_session_transfer"
        }));
        return decision;
    }

    if let Some(expected) = clean_resume_scope_value(expected_continuity_id) {
        let actual = record.continuity_id.as_deref().unwrap_or("").trim();
        if actual.is_empty() || actual != expected {
            decision.canonical_scope_ok = false;
            decision.rejection = Some(json!({
                "status": "rejected_continuity_mismatch",
                "canonical": false,
                "workpoint_id": record.workpoint_id,
                "warnings": ["workpoint continuity_id does not match current logical session"],
                "expected_continuity_id": expected,
                "packet_continuity_id": actual,
                "safe_recovery": "select the matching SilentSession/Pi continuity_id or checkpoint a fresh Workpoint for this logical session",
                "requested_found": true,
                "scope_found": false,
                "fallback_used": false,
                "canonical_for_requested_scope": false,
                "canonical_for_fallback_scope": false,
                "wrong_id_taxonomy": wrong_id_taxonomy_payload(WrongIdTaxonomy {
                    status: "scope_mismatch_for_requested_id",
                    workpoint_id: Some(record.workpoint_id),
                    requested_workpoint_id: Some(record.workpoint_id),
                    requested_found: true,
                    scope_found: false,
                    fallback_used: false,
                    canonical_for_requested_scope: false,
                    canonical_for_fallback_scope: false,
                }),
                "next_step_hint": "list/reopen the correct SilentSession or create a checkpoint carrying this continuity_id"
            }));
            return decision;
        }
        decision.expected_continuity_id = Some(expected);
        decision.packet_continuity_id = Some(actual.to_string());
    } else if record.continuity_id.is_none() {
        decision.canonical_scope_ok = false;
        decision.warnings.push(
            "resume requested without continuity_id and packet has no continuity_id; logical session identity is unbound"
                .to_string(),
        );
    }

    if let Some(expected) = clean_resume_scope_value(expected_session_id) {
        let actual = record.session_id.as_deref().unwrap_or("").trim();
        if !actual.is_empty() && actual != expected {
            decision.session_changed = true;
            decision.expected_session_id = Some(expected);
            decision.packet_session_id = Some(actual.to_string());
            decision.warnings.push("workpoint session_id differs from current Pi session; project_root matched, preserving post-compaction continuity".to_string());
        }
    }
    decision
}

fn identity_confidence_payload(
    record: &WorkpointRecord,
    scope: &ResumeScopeDecision,
    req: &WorkpointResumeRequest,
) -> Value {
    let mut score: u8 = 0;
    let mut factors: Vec<Value> = Vec::new();
    let project_match = clean_resume_scope_value(req.project_root.as_deref())
        .zip(clean_resume_scope_value(record.project_root.as_deref()))
        .map(|(expected, actual)| expected == actual)
        .unwrap_or(false);
    if project_match {
        score = score.saturating_add(30);
    }
    factors
        .push(json!({"factor":"project_root","matched":project_match,"weight":30,"required":true}));

    let continuity_match = clean_resume_scope_value(req.continuity_id.as_deref())
        .zip(clean_resume_scope_value(record.continuity_id.as_deref()))
        .map(|(expected, actual)| expected == actual)
        .unwrap_or(false);
    if continuity_match {
        score = score.saturating_add(40);
    }
    factors.push(
        json!({"factor":"continuity_id","matched":continuity_match,"weight":40,"required":true}),
    );

    let work_item_match = clean_resume_scope_value(req.work_item_id.as_deref())
        .zip(clean_resume_scope_value(record.work_item_id.as_deref()))
        .map(|(expected, actual)| expected == actual)
        .unwrap_or(false);
    if work_item_match {
        score = score.saturating_add(10);
    }
    factors.push(
        json!({"factor":"work_item_id","matched":work_item_match,"weight":10,"required":false}),
    );

    let trajectory_match = clean_resume_scope_value(req.trajectory_id.as_deref())
        .map(|expected| {
            record
                .active_object_refs
                .iter()
                .any(|item| item == &expected)
                || record
                    .action_intent
                    .as_ref()
                    .and_then(|intent| intent.target_ref.as_deref())
                    == Some(expected.as_str())
        })
        .unwrap_or(false);
    if trajectory_match {
        score = score.saturating_add(10);
    }
    factors.push(json!({"factor":"trajectory_id","matched":trajectory_match,"weight":10,"required":false,"role":"corroborating_only"}));

    let frame_tag_match = record
        .continuity_id
        .as_deref()
        .map(|continuity| {
            req.frame_tags
                .iter()
                .any(|tag| tag == continuity || tag == &format!("continuity_id:{continuity}"))
        })
        .unwrap_or(false);
    if frame_tag_match {
        score = score.saturating_add(5);
    }
    factors.push(json!({"factor":"frame_continuity_tag","matched":frame_tag_match,"weight":5,"required":false}));

    let session_ok = !scope.session_changed || (project_match && continuity_match);
    if session_ok {
        score = score.saturating_add(5);
    }
    factors.push(json!({"factor":"session_id_temporal_continuity","matched":session_ok,"weight":5,"required":false}));

    let percent = score.min(100);
    let level = if percent >= 95 {
        "very_high"
    } else if percent >= 80 {
        "high"
    } else if percent >= 60 {
        "medium"
    } else {
        "low"
    };
    json!({
        "percent": percent,
        "level": level,
        "hard_gates": {
            "project_root_match": project_match,
            "continuity_id_match": continuity_match,
            "policy": "hard_gates_required_before_corroborating_signals_count"
        },
        "factors": factors
    })
}

pub(crate) fn idempotency_cache_status_payload() -> Value {
    json!({
        "schema": "focusa.workpoint_idempotency_cache.v1",
        "status": "eliminated",
        "cache_enabled": false,
        "authority": "scope-matched Workpoint reducer records",
        "cross_scope_fallback": false,
    })
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
    object_tokens(object_ref).iter().any(|token| {
        latest_tokens
            .iter()
            .any(|latest_token| latest_token == token)
    })
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
        .or_else(|| {
            record
                .action_intent
                .as_ref()
                .map(|intent| intent.action_type.as_str())
        })
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

    let notes_only_markers = [
        "note",
        "notes",
        "document",
        "docs",
        "breadcrumb",
        "summary",
        "handoff",
    ];
    let implementation_markers = [
        "implement",
        "patch",
        "edit",
        "verify",
        "test",
        "run",
        "inspect",
        "fix",
    ];
    let action_requires_execution = action_norm.contains("patch")
        || action_norm.contains("implement")
        || action_norm.contains("verify")
        || action_norm.contains("binding")
        || action_norm.contains("resume workpoint");
    if action_requires_execution
        && notes_only_markers
            .iter()
            .any(|marker| latest_norm.contains(marker))
        && !implementation_markers
            .iter()
            .any(|marker| latest_norm.contains(marker))
    {
        classes.push("notes_only_drift".to_string());
        reasons.push("latest action appears notes-only while Workpoint requires implementation or verification".to_string());
    }

    let mut active_objects = record.active_object_refs.clone();
    active_objects.extend(request_objects.iter().cloned());
    if let Some(target) = record
        .action_intent
        .as_ref()
        .and_then(|intent| intent.target_ref.clone())
    {
        active_objects.push(target);
    }
    active_objects.sort();
    active_objects.dedup();
    if !active_objects.is_empty()
        && !active_objects
            .iter()
            .any(|object| latest_mentions_object(latest_action, object))
    {
        classes.push("wrong_object_drift".to_string());
        reasons.push(
            "latest action does not mention any active target object or action target".to_string(),
        );
    }

    let mut boundaries: Vec<String> = request_boundaries.to_vec();
    if let Some(next) = &record.next_slice {
        boundaries.extend(next.lines().filter_map(|line| {
            line.split_once("DO_NOT_DRIFT:")
                .map(|(_, rhs)| rhs.trim().to_string())
        }));
    }
    for boundary in boundaries
        .iter()
        .filter(|boundary| !boundary.trim().is_empty())
    {
        if latest_mentions_object(latest_action, boundary)
            || latest_norm.contains(&normalize_for_match(boundary))
        {
            classes.push("do_not_drift_boundary".to_string());
            reasons.push(format!(
                "latest action touches prohibited boundary: {boundary}"
            ));
            break;
        }
    }

    if !action_norm.is_empty() && !latest_norm.contains(&action_norm) {
        let action_terms: Vec<_> = action_norm
            .split_whitespace()
            .filter(|term| term.len() >= 4)
            .collect();
        if !action_terms.is_empty() && !action_terms.iter().any(|term| latest_norm.contains(term)) {
            classes.push("action_intent_ignored".to_string());
            reasons.push(format!(
                "latest action does not align with expected action {action}"
            ));
        }
    }

    let drift_detected = !classes.is_empty();
    DriftDecision {
        drift_detected,
        severity: if classes
            .iter()
            .any(|class| class == "do_not_drift_boundary" || class == "wrong_object_drift")
        {
            WorkpointDriftSeverity::High
        } else if drift_detected {
            WorkpointDriftSeverity::Medium
        } else {
            WorkpointDriftSeverity::Info
        },
        reason: if reasons.is_empty() {
            "latest action aligns with active Workpoint".to_string()
        } else {
            reasons.join("; ")
        },
        recovery_hint: if drift_detected {
            "call /v1/workpoint/resume and continue the packet next_slice before adjacent work"
                .to_string()
        } else {
            "continue current action".to_string()
        },
        drift_classes: classes,
    }
}

fn active_workpoint(state: &focusa_core::types::FocusaState) -> Option<&WorkpointRecord> {
    state.workpoint.active_workpoint_id.and_then(|id| {
        state.workpoint.records.iter().find(|record| {
            record.workpoint_id == id
                && unsafe_project_root_reason(record.project_root.as_deref()).is_none()
        })
    })
}

pub(crate) fn active_workpoint_for_scope<'a>(
    state: &'a focusa_core::types::FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> Option<&'a WorkpointRecord> {
    let clean_project = clean_resume_scope_value(project_root)?;
    if unsafe_project_root_reason(Some(clean_project.as_str())).is_some() {
        return None;
    }
    let clean_continuity = clean_resume_scope_value(continuity_id)?;
    state.workpoint.records.iter().rev().find(|record| {
        record.status == WorkpointStatus::Active
            && record.canonical
            && unsafe_project_root_reason(record.project_root.as_deref()).is_none()
            && record.project_root.as_deref().map(str::trim) == Some(clean_project.as_str())
            && record.continuity_id.as_deref().map(str::trim) == Some(clean_continuity.as_str())
    })
}

pub(crate) fn active_workpoint_for_context<'a>(
    state: &'a focusa_core::types::FocusaState,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    working_subpath_id: Option<&str>,
) -> Option<&'a WorkpointRecord> {
    let expected =
        clean_resume_scope_value(working_subpath_id).unwrap_or_else(|| "primary".to_string());
    state.workpoint.records.iter().rev().find(|record| {
        record.status == WorkpointStatus::Active
            && record.canonical
            && unsafe_project_root_reason(record.project_root.as_deref()).is_none()
            && record.project_root.as_deref().map(str::trim)
                == clean_resume_scope_value(project_root).as_deref()
            && record.continuity_id.as_deref().map(str::trim)
                == clean_resume_scope_value(continuity_id).as_deref()
            && record_working_subpath_id(record) == expected
    })
}

fn parse_checkpoint_reason(
    reason: Option<&str>,
) -> Result<WorkpointCheckpointReason, (StatusCode, Json<Value>)> {
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
        "operator-checkpoint" | "operator_checkpoint" => {
            Ok(WorkpointCheckpointReason::OperatorCheckpoint)
        }
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

fn workpoint_array_bounds(
    total: usize,
    returned: usize,
    default_limit: usize,
    full_limit: usize,
) -> Value {
    json!(bounded_metadata(
        total,
        returned,
        BoundedReadOptions {
            requested_limit: None,
            include_full_payload: false,
            summary_only: true,
            cursor: None,
            next_cursor: (returned < total).then(|| returned.to_string()),
            default_limit,
            full_limit,
        },
    ))
}

fn workpoint_packet(record: &WorkpointRecord) -> Value {
    let object_default = budgeted_default_limit("FOCUSA_WORKPOINT_PACKET_OBJECT_LIMIT", 16);
    let object_full = budgeted_hard_limit(
        "FOCUSA_WORKPOINT_PACKET_OBJECT_FULL_LIMIT",
        128,
        object_default,
    );
    let evidence_default = budgeted_default_limit("FOCUSA_WORKPOINT_PACKET_EVIDENCE_LIMIT", 16);
    let evidence_full = budgeted_hard_limit(
        "FOCUSA_WORKPOINT_PACKET_EVIDENCE_FULL_LIMIT",
        128,
        evidence_default,
    );
    let active_object_refs = record
        .active_object_refs
        .iter()
        .take(object_default)
        .cloned()
        .collect::<Vec<_>>();
    let verification_records = record
        .verification_records
        .iter()
        .take(evidence_default)
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "workpoint_id": record.workpoint_id,
        "work_item_id": record.work_item_id,
        "session_identity": record.session_identity,
        "continuity_id": record.continuity_id,
        "session_id": record.session_id,
        "project_root": record.project_root,
        "frame_id": record.frame_id,
        "status": record.status,
        "checkpoint_reason": record.checkpoint_reason,
        "confidence": record.confidence,
        "canonical": record.canonical,
        // FOCUSA_FIX-nzru: annotate freshness so the agent knows next_slice
        // may be stale (items closed since checkpoint).
        "stale_note": match record.updated_at.as_ref() {
            Some(ts) => {
                let age_secs = chrono::Utc::now().signed_duration_since(*ts).num_seconds().max(0);
                if age_secs > 3600 {
                    Some(format!("packet age={}min — next_slice may reference items closed since checkpoint. Re-checkpoint if mission changed.", age_secs / 60))
                } else {
                    None
                }
            }
            None => None,
        },
        "mission": record.mission,
        "active_object_refs": active_object_refs,
        "active_object_refs_metadata": workpoint_array_bounds(record.active_object_refs.len(), active_object_refs.len(), object_default, object_full),
        "action_intent": record.action_intent,
        "verification_records": verification_records,
        "verification_records_metadata": workpoint_array_bounds(record.verification_records.len(), verification_records.len(), evidence_default, evidence_full),
        "blockers": record.blockers,
        "next_slice": record.next_slice,
        "source_turn_id": record.source_turn_id,
        "idempotency_key": record.idempotency_key,
        "updated_at": record.updated_at,
    })
}

fn current_workpoint_payload(record: &WorkpointRecord) -> Value {
    json!({
        "status": record.status,
        "operation_status": "completed",
        "workpoint_id": record.workpoint_id,
        "canonical": record.canonical,
        "project_root": record.project_root,
        "continuity_id": record.continuity_id,
        "session_id": record.session_id,
        "scope": {
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "session_id": record.session_id,
            "scope_status": if record.project_root.as_deref().is_some_and(|value| !value.trim().is_empty())
                && record.continuity_id.as_deref().is_some_and(|value| !value.trim().is_empty()) { "verified" } else { "partial" },
        },
        "workpoint": workpoint_packet(record),
        "warnings": [],
        "next_step_hint": record.next_slice,
    })
}

fn checkpoint_summary(record: &WorkpointRecord) -> Value {
    let action_type = record
        .action_intent
        .as_ref()
        .map(|intent| intent.action_type.as_str())
        .unwrap_or("unspecified_action");
    let target_ref = record
        .action_intent
        .as_ref()
        .and_then(|intent| intent.target_ref.as_deref())
        .or_else(|| record.active_object_refs.first().map(String::as_str))
        .unwrap_or("unspecified_target");
    let mission = record.mission.as_deref().unwrap_or("unspecified mission");
    let next_slice = record
        .next_slice
        .as_deref()
        .unwrap_or("resume packet next_slice");
    json!({
        "one_line": format!(
            "checkpointed mission={}; action={}; target={}; next={}",
            mission, action_type, target_ref, next_slice
        ),
        "mission": mission,
        "action_type": action_type,
        "target_ref": target_ref,
        "next_slice": next_slice,
        "work_item_id": record.work_item_id,
        "project_root": record.project_root,
        "continuity_id": record.continuity_id,
        "canonical": record.canonical,
    })
}

fn resume_summary(record: &WorkpointRecord) -> String {
    let action = record
        .action_intent
        .as_ref()
        .map(|intent| intent.action_type.as_str())
        .unwrap_or("unknown_action");
    let next = record
        .next_slice
        .as_deref()
        .unwrap_or("continue from active workpoint");
    let handoff = if record.canonical
        && record
            .next_slice
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        && record.verification_records.iter().any(|verification| {
            verification
                .evidence_ref
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        }) {
        "handoff: ready"
    } else {
        "handoff: partial"
    };
    format!(
        "WORKPOINT {}: mission={}; action={}; next={}; canonical={}; {}",
        record.workpoint_id,
        record.mission.as_deref().unwrap_or("unknown"),
        action,
        next,
        record.canonical,
        handoff
    )
}

fn handoff_quality_payload(
    record: &WorkpointRecord,
    canonical: bool,
    action_authority: bool,
) -> Value {
    let next_exact = record
        .next_slice
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty() && value != "continue from active workpoint");
    let proof_linked = record.verification_records.iter().any(|verification| {
        verification
            .evidence_ref
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    });
    let mut missing = Vec::<String>::new();
    if !canonical || !action_authority {
        missing.push("canonical_authority".to_string());
    }
    if !next_exact {
        missing.push("exact_next_action".to_string());
    }
    if !proof_linked {
        missing.push("linked_proof".to_string());
    }
    let stale = if canonical {
        Vec::<String>::new()
    } else {
        vec!["authority".to_string()]
    };
    let mut score: i64 = 100;
    if !canonical || !action_authority {
        score -= 45;
    }
    if !next_exact {
        score -= 25;
    }
    if !proof_linked {
        score -= 20;
    }
    let score = score.clamp(0, 100) as u64;
    let status = if score >= 90 && missing.is_empty() && stale.is_empty() {
        "ready"
    } else if score >= 50 {
        "partial"
    } else {
        "unsafe"
    };
    json!({
        "score": score,
        "status": status,
        "missing": missing,
        "stale": stale,
        "authority": if canonical && action_authority { "canonical" } else { "degraded" },
        "next_action_quality": if next_exact { "exact" } else { "missing_or_generic" },
        "proof_quality": if proof_linked { "linked" } else { "missing" },
        "exact_next_action": safest_next_action(record),
    })
}

fn checkpoint_mutation_preview(
    req: &WorkpointCheckpointRequest,
    workpoint_id: Uuid,
    safe_to_apply: bool,
) -> Value {
    json!({
        "route": "POST /v1/workpoint/checkpoint",
        "would_create": [{"type": "workpoint", "workpoint_id": workpoint_id, "work_item_id": req.work_item_id, "mission": req.mission}],
        "would_update": if req.promote.unwrap_or(true) && req.canonical.unwrap_or(true) { json!([{"type": "active_workpoint", "workpoint_id": workpoint_id}]) } else { json!([]) },
        "would_link": req.verification_records.as_ref().map(|records| json!(records.iter().map(|record| json!({"target_ref": record.target_ref, "evidence_ref": record.evidence_ref})).collect::<Vec<_>>())).unwrap_or_else(|| json!([])),
        "authority_scope": {"project_root": req.project_root, "continuity_id": req.continuity_id, "session_id": req.session_id},
        "risk": if safe_to_apply { "low" } else { "unsafe_scope" },
        "irreversible": false,
        "safe_to_apply": safe_to_apply,
    })
}

const TRUST_BADGE_VOCABULARY: &[&str] = &[
    "canonical",
    "advisory",
    "projected",
    "stale",
    "degraded",
    "blocked",
    "spec_only",
    "partial",
    "verified",
    "unsafe_scope",
];

fn route_recommendation_payload(canonical: bool, action_authority: bool) -> Value {
    json!({
        "recommended_tool": if canonical && action_authority { "focusa_trajectory_view" } else { "focusa_project_identity" },
        "why": if canonical && action_authority { "bounded next route refreshes goal/state/gap without broad or cold reads" } else { "project scope must be verified before durable continuation" },
        "expected_output": if canonical && action_authority { "current goal, verified state, active gap, and next Workpoint candidate" } else { "verified project_root, continuity_id, repo identity, and safe scope" },
        "confidence": if canonical && action_authority { "high" } else { "medium" },
        "alternatives": if canonical && action_authority { vec!["focusa_traverse", "focusa_workpoint_resume", "focusa_active_object_resolve"] } else { vec!["focusa_project_verify", "focusa_workpoint_checkpoint", "focusa_tool_doctor"] },
        "avoid": ["full lineage tree", "full ontology graph", "full telemetry logs", "transcript tail as authority"],
    })
}

fn trust_badges(
    canonical: bool,
    degraded: bool,
    blocked: bool,
    projected: bool,
    partial: bool,
    unsafe_scope: bool,
) -> Vec<&'static str> {
    let _ = TRUST_BADGE_VOCABULARY;
    if blocked {
        return vec!["blocked", "degraded"];
    }
    if unsafe_scope {
        return vec!["unsafe_scope", "degraded"];
    }
    if degraded {
        return vec!["degraded"];
    }
    if partial {
        return vec!["partial", "advisory"];
    }
    if projected {
        return vec!["projected", "advisory"];
    }
    if canonical {
        vec!["canonical", "verified"]
    } else {
        vec!["advisory"]
    }
}

fn rollback_card_payload(
    latest_safe_snapshot: Value,
    workpoint_id: Option<Uuid>,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    reversible_action: &str,
    expected_after_restore: &str,
) -> Value {
    json!({
        "latest_safe_snapshot": latest_safe_snapshot,
        "reversible_actions": [reversible_action],
        "irreversible_actions": [],
        "restore_tool": "focusa_tree_restore_state",
        "restore_scope": {"project_root": project_root, "continuity_id": continuity_id, "workpoint_id": workpoint_id},
        "expected_after_restore": expected_after_restore,
    })
}

fn evidence_link_mutation_preview(
    record: &WorkpointRecord,
    verification: &WorkpointVerificationRecord,
) -> Value {
    json!({
        "route": "POST /v1/workpoint/evidence/link",
        "would_create": [],
        "would_update": [{"type": "workpoint.verification_records", "workpoint_id": record.workpoint_id}],
        "would_link": [{"workpoint_id": record.workpoint_id, "target_ref": verification.target_ref, "evidence_ref": verification.evidence_ref}],
        "authority_scope": {"project_root": record.project_root, "continuity_id": record.continuity_id, "session_id": record.session_id},
        "risk": "low",
        "irreversible": false,
        "safe_to_apply": true,
    })
}

fn workpoint_visibility_wait_attempts() -> usize {
    match resource_mode_status().mode {
        "emergency" => 1,
        "lowmem" => 2,
        "constrained" => 8,
        _ => 40,
    }
}

async fn wait_for_workpoint_record(
    _scope: ScopeContext,
    state: &Arc<AppState>,
    workpoint_id: Uuid,
) -> Option<WorkpointRecord> {
    let attempts = workpoint_visibility_wait_attempts();
    for attempt in 0..attempts {
        {
            let focusa = workstream_scoped_state(state.clone(), &_scope).await;
            if let Some(record) = focusa
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == workpoint_id)
            {
                return Some(record.clone());
            }
        }
        if attempt + 1 < attempts {
            sleep(Duration::from_millis(50)).await;
        }
    }
    None
}

fn workpoint_failure(
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
        "validation_rejected" | "not_found" | "scope_mismatch" | "permission_denied"
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
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "reflex_suggestions": reflex_suggestions,
            "details": {
                "tool_result_v1": {
                    "ok": false,
                    "status": "blocked",
                    "canonical": false,
                    "degraded": true,
                    "failure_class": failure_class,
                    "summary": why,
                    "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                    "recovery_hint": recovery_hint,
                    "misuse_hint": misuse_hint,
                    "side_effects": [],
                    "evidence_refs": [],
                    "next_tools": next_tools_value,
                    "reflex_suggestions": reflex_suggestions,
                    "error": {"code": failure_class, "message": error}
                }
            }
        })),
    )
}

fn workpoint_reducer_rejected(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    workpoint_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("workpoint reducer rejected event: {error}"),
        "reducer_rejected",
        format!("Workpoint event could not be reduced into Focusa state: {error}"),
        "Inspect reducer constraints and current Workpoint packet before retrying the same mutation.",
        "Likely stale Workpoint state, invalid event payload, or reducer invariant mismatch.",
        vec![
            "focusa_workpoint_resume",
            "focusa_tool_doctor",
            "focusa_trajectory_view",
        ],
    )
}

fn workpoint_persistence_failed(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    workpoint_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to persist workpoint event: {error}"),
        "persistence_failed",
        format!("Workpoint event was reduced but could not be persisted: {error}"),
        "Check persistence health before retrying; do not rely on transcript-only proof.",
        "Likely SQLite/file permission/resource pressure or event-log persistence outage.",
        vec![
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_workpoint_resume",
        ],
    )
}

fn workpoint_dispatch_failed(error: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    workpoint_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("dispatch failed: {error}"),
        "daemon_unavailable",
        format!("Workpoint event could not be dispatched to daemon command channel: {error}"),
        "Check daemon health and retry only after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec![
            "focusa_tool_doctor",
            "focusa_work_loop_status",
            "focusa_workpoint_resume",
        ],
    )
}

fn workpoint_dispatch_timeout() -> (StatusCode, Json<Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "pending",
            "canonical": false,
            "degraded": true,
            "failure_class": "resource_exhausted",
            "retry_posture": "safe_retry",
            "retry": {"safe": true, "posture": "safe_retry", "reason": "daemon command channel is saturated"},
            "side_effects": [],
            "next_tools": ["focusa_resource_mode", "focusa_workpoint_resume", "focusa_traverse"],
            "next_step_hint": "retry after command backlog drains; workpoint event was not enqueued"
        })),
    )
}

fn workpoint_no_active_to_link() -> (StatusCode, Json<Value>) {
    workpoint_failure(
        StatusCode::NOT_FOUND,
        "no active Workpoint to link evidence",
        "not_found",
        "No canonical active Workpoint was available for evidence linking.",
        "Create or resume a canonical project-scoped Workpoint before linking evidence.",
        "Likely stale/missing Workpoint packet, unsafe project scope, or wrong continuity_id.",
        vec![
            "focusa_project_identity",
            "focusa_workpoint_checkpoint",
            "focusa_workpoint_resume",
        ],
    )
}

async fn materialize_workpoint_events(
    _scope: ScopeContext,
    state: &Arc<AppState>,
    events: Vec<FocusaEvent>,
    correlation_id: &'static str,
) -> Result<focusa_core::types::FocusaState, (StatusCode, Json<Value>)> {
    let _guard = state.write_serial_lock.lock().await;
    let mut current = workstream_scoped_state(state.clone(), &_scope).await.clone();

    for event in events {
        let result = reducer::reduce_with_meta(current, event, None, None, false).map_err(|error| {
            tracing::warn!(error = %error, correlation_id, "workpoint event rejected by reducer");
            workpoint_reducer_rejected(error)
        })?;
        current = result.new_state;

        for emitted in result.emitted_events {
            let entry = EventLogEntry {
                id: Uuid::now_v7(),
                timestamp: Utc::now(),
                event: emitted,
                correlation_id: Some(correlation_id.to_string()),
                origin: SignalOrigin::Adapter,
                machine_id: None,
                instance_id: None,
                session_id: current.session.as_ref().map(|session| session.session_id),
                thread_id: None,
                is_observation: false,
            };
            if let Err(error) = state.append_events_checkpoint(vec![entry.clone()]).await {
                tracing::error!(error = %error, correlation_id, "failed to persist workpoint event");
                return Err(workpoint_persistence_failed(error));
            } else if let Ok(serialized) = serde_json::to_string(&entry) {
                let _ = state.events_tx.send(serialized);
            }
        }
    }

    // This route reduces Workpoint events directly instead of sending them
    // through the daemon action loop. Persist the resulting canonical state
    // before publishing it in memory, otherwise checkpoints disappear after
    // a daemon restart even though the request returned canonical=true.
    state.persist_checkpoint(current.clone()).await.map_err(|error| {
        tracing::error!(error = %error, correlation_id, "failed to persist canonical workpoint state");
        workpoint_persistence_failed(error)
    })?;
    *state.focusa.write().await = current.clone();
    state.mark_external_mutation();
    Ok(current)
}

async fn dispatch_event(
    _scope: ScopeContext,
    state: &Arc<AppState>,
    event: FocusaEvent,
) -> Result<(), (StatusCode, Json<Value>)> {
    if lowmem_caps_active() {
        match state.command_tx.try_send(Action::EmitEvent { event }) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(action)) => {
                match tokio::time::timeout(
                    Duration::from_millis(1500),
                    state.command_tx.send(action),
                )
                .await
                {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(_)) => {
                        return Err(workpoint_dispatch_failed("daemon command channel closed"));
                    }
                    Err(_) => {
                        return Err((
                            StatusCode::ACCEPTED,
                            Json(json!({
                                "status": "pending",
                                "canonical": false,
                                "degraded": true,
                                "failure_class": "resource_exhausted",
                                "retry_posture": "safe_retry",
                                "retry": {"safe": true, "posture": "safe_retry", "reason": "daemon command channel is saturated under LowMem"},
                                "side_effects": [],
                                "resource_mode": resource_mode_status(),
                                "next_tools": ["focusa_resource_mode", "focusa_workpoint_resume", "focusa_traverse"],
                                "next_step_hint": "retry after LowMem command backlog drains; evidence payload was not enqueued"
                            })),
                        ));
                    }
                }
            }
            Err(TrySendError::Closed(_)) => {
                return Err(workpoint_dispatch_failed("daemon command channel closed"));
            }
        }
    }

    match tokio::time::timeout(
        Duration::from_millis(1500),
        state.command_tx.send(Action::EmitEvent { event }),
    )
    .await
    {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(workpoint_dispatch_failed(error)),
        Err(_) => Err(workpoint_dispatch_timeout()),
    }
}

async fn rollover_target_materialize(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointRolloverTargetMaterializeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }

    let Some(source_continuity_id) = rollover_required_ref(&req, "source_continuity_id") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "source_continuity_id",
                "error": "required for target Workpoint materialization",
                "next_step_hint": "pass source continuity id from the source transfer scope"
            })),
        ));
    };

    let Some(target_continuity_id) = rollover_required_ref(&req, "target_continuity_id") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "target_continuity_id",
                "error": "required for target Workpoint continuity rotation",
                "next_step_hint": "generate and pass a new target continuity id for rollover"
            })),
        ));
    };

    if source_continuity_id == target_continuity_id {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "blocked",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "field": "target_continuity_id",
                "expected": "different from source continuity",
                "source_continuity_id": source_continuity_id,
                "target_continuity_id": target_continuity_id,
                "next_step_hint": "rollover requires a rotated continuity, not reuse source continuity"
            })),
        ));
    }

    let Some(source_session_id) = rollover_required_ref(&req, "source_session_id") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "source_session_id",
                "error": "required for source-target continuity transfer traceability",
                "next_step_hint": "pass source Pi session id used by the transfer"
            })),
        ));
    };

    let Some(target_session_id) = rollover_required_ref(&req, "target_session_id") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "target_session_id",
                "error": "required for target Workpoint continuity transfer traceability",
                "next_step_hint": "pass target Pi session id created for the target attachment"
            })),
        ));
    };

    let Some(project_root) = rollover_required_ref(&req, "project_root") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "project_root",
                "error": "required for reducer-approved target Workpoint materialization",
                "next_step_hint": "pass the typed source project root"
            })),
        ));
    };

    if !project_root.starts_with('/') {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "project_root",
                "error": "must be an absolute project root path",
                "next_step_hint": "derive project_root from typed scope, never from cwd fingerprint"
            })),
        ));
    }

    if let Some(reason) = unsafe_project_root_reason(Some(project_root.as_str())) {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "rejected_scope_mismatch",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "expected": "verified project-safe project root",
                "reason": reason,
                "next_step_hint": "confirm project scope via project_identity tools before transfer"
            })),
        ));
    }

    let Some(checkpoint_ref) = rollover_required_ref(&req, "checkpoint_ref") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "checkpoint_ref",
                "error": "required provenance reference for target materialization",
                "next_step_hint": "pass source checkpoint ref from prepareCompactionRollover"
            })),
        ));
    };

    let Some(workpoint_packet_ref) = rollover_required_ref(&req, "workpoint_packet_ref") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "workpoint_packet_ref",
                "error": "required provenance reference for target materialization",
                "next_step_hint": "pass source Workpoint packet ref from prepareCompactionRollover"
            })),
        ));
    };

    let Some(compaction_packet_ref) = rollover_required_ref(&req, "compaction_packet_ref") else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "compaction_packet_ref",
                "error": "required provenance reference for target materialization",
                "next_step_hint": "pass source compaction packet ref from prepareCompactionRollover"
            })),
        ));
    };

    let working_subpath_id = clean_resume_scope_value(req.working_subpath_id.as_deref())
        .unwrap_or_else(|| "primary".to_string());
    let focusa_guard;
    let focusa = match (Some(project_root.as_str()), Some(source_continuity_id.as_str())) {
        (Some(root), Some(continuity)) => {
            let partition = state
                .workstream_states
                .get_or_create(root, continuity)
                .await;
            focusa_guard = partition.read_owned().await;
            &focusa_guard
        }
        _ => {
            focusa_guard = state.focusa.clone().read_owned().await;
            &focusa_guard
        }
    };
    let source_record = active_workpoint_for_context(
        focusa,
        Some(project_root.as_str()),
        Some(source_continuity_id.as_str()),
        Some(working_subpath_id.as_str()),
    )
    .cloned();

    let Some(source_record) = source_record else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "not_found",
                "canonical": false,
                "failure_class": "not_found",
                "field": "source continuity",
                "expected_project_root": project_root,
                "expected_continuity_id": source_continuity_id,
                "next_step_hint": "checkpoint Workpoint in source continuity first"
            })),
        ));
    };

    if source_record.session_id.as_deref() != Some(source_session_id.as_str()) {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "rejected_scope_mismatch",
                "canonical": false,
                "failure_class": "scope_mismatch",
                "field": "source_session_id",
                "expected_source_session_id": source_session_id,
                "packet_source_session_id": source_record.session_id,
                "next_step_hint": "use the source session id from prepare and native source migration"
            })),
        ));
    }

    let target_record_idempotency_key = rollover_target_materialization_idempotency_key(
        source_continuity_id.as_str(),
        target_continuity_id.as_str(),
        source_session_id.as_str(),
        target_session_id.as_str(),
        checkpoint_ref.as_str(),
        workpoint_packet_ref.as_str(),
        compaction_packet_ref.as_str(),
    );

    let maybe_existing = focusa.workpoint.records.iter().rev().find(|record| {
        record.idempotency_key.as_deref() == Some(target_record_idempotency_key.as_str())
            && record.continuity_id.as_deref() == Some(target_continuity_id.as_str())
            && record.project_root.as_deref() == Some(project_root.as_str())
    });

    if let Some(existing) = maybe_existing {
        if existing.status == WorkpointStatus::Active && existing.canonical {
            return Ok(Json(json!({
                "status": "completed",
                "schema": "focusa.workpoint_rollover_target_materialize.v1",
                "canonical": true,
                "source_workpoint_id": source_record.workpoint_id,
                "target_workpoint_id": existing.workpoint_id,
                "target_continuity_id": target_continuity_id,
                "target_session_id": target_session_id,
                "workpoint_id": existing.workpoint_id,
                "workpoint": workpoint_packet(existing),
                "required_refs": {
                    "checkpoint_ref": checkpoint_ref,
                    "workpoint_packet_ref": workpoint_packet_ref,
                    "compaction_packet_ref": compaction_packet_ref,
                },
                "status_hint": "idempotent_replay",
                "next_tools": ["focusa_workpoint_resume", "focusa_project_session_transfer"],
            })));
        }
        return Err((
            StatusCode::ACCEPTED,
            Json(json!({
                "status": "pending",
                "canonical": existing.canonical,
                "failure_class": "read_model_lag",
                "workpoint_id": existing.workpoint_id,
                "required_refs": {
                    "checkpoint_ref": checkpoint_ref,
                    "workpoint_packet_ref": workpoint_packet_ref,
                    "compaction_packet_ref": compaction_packet_ref,
                },
                "next_step_hint": "retry /v1/workpoint/rollover/target-materialize until target Workpoint is materialized and promoted"
            })),
        ));
    }
    let _ = focusa;

    let mut target_session_identity = source_record.session_identity.clone();
    if let Some(identity) = target_session_identity.as_mut() {
        identity.pi_session_id = Some(target_session_id.clone());
        identity.session_frame_key = target_session_id.clone();
        identity.session_incarnation_id =
            format!("{target_session_id}:rollover-target-materialize");
        identity.continuity_id = Some(target_continuity_id.clone());
        identity.project_root = project_root.clone();
        identity.cwd = project_root.clone();
        identity.workspace_id = project_root.clone();
        identity.canonical_scope = Some(true);
    }

    let source_workpoint_id = source_record.workpoint_id;
    let mut target_record = source_record;
    target_record.workpoint_id = Uuid::now_v7();
    target_record.continuity_id = Some(target_continuity_id.clone());
    target_record.session_id = Some(target_session_id);
    target_record.project_root = Some(project_root.clone());
    target_record.session_identity = target_session_identity;
    target_record.status = WorkpointStatus::Proposed;
    target_record.canonical = true;
    target_record.idempotency_key = Some(target_record_idempotency_key.clone());
    target_record.active_object_refs = rollout_target_materialization_refs(
        target_record.active_object_refs,
        checkpoint_ref.as_str(),
        workpoint_packet_ref.as_str(),
        compaction_packet_ref.as_str(),
    );
    target_record.supersedes = Some(source_workpoint_id);
    target_record.created_at = None;
    target_record.updated_at = None;

    let materialized_state = materialize_workpoint_events(
        _scope.clone(),
        &state,
        vec![
            FocusaEvent::WorkpointCheckpointProposed {
                workpoint: target_record.clone(),
            },
            FocusaEvent::WorkpointCheckpointPromoted {
                workpoint_id: target_record.workpoint_id,
                confidence: target_record.confidence,
                reason: "rollover target materialization".to_string(),
            },
        ],
        "workpoint_rollover_target_materialize",
    )
    .await?;

    let Some(materialized_target_record) =
        materialized_state.workpoint.records.iter().find(|record| {
            record.workpoint_id == target_record.workpoint_id
                && record.canonical
                && record.status == WorkpointStatus::Active
        })
    else {
        return Err((
            StatusCode::ACCEPTED,
            Json(json!({
                "status": "pending",
                "canonical": true,
                "failure_class": "read_model_lag",
                "workpoint_id": target_record.workpoint_id,
                "required_refs": {
                    "checkpoint_ref": checkpoint_ref,
                    "workpoint_packet_ref": workpoint_packet_ref,
                    "compaction_packet_ref": compaction_packet_ref,
                },
                "next_step_hint": "retry /v1/workpoint/rollover/target-materialize until target Workpoint record is visible"
            })),
        ));
    };

    Ok(Json(json!({
        "status": "completed",
        "schema": "focusa.workpoint_rollover_target_materialize.v1",
        "canonical": true,
        "degraded": false,
        "source_workpoint_id": source_workpoint_id,
        "target_workpoint_id": materialized_target_record.workpoint_id,
        "workpoint_id": materialized_target_record.workpoint_id,
        "target_continuity_id": target_continuity_id,
        "target_session_id": materialized_target_record.session_id,
        "workpoint": workpoint_packet(materialized_target_record),
        "required_refs": {
            "checkpoint_ref": checkpoint_ref,
            "workpoint_packet_ref": workpoint_packet_ref,
            "compaction_packet_ref": compaction_packet_ref,
        },
        "next_tools": ["focusa_workpoint_resume", "focusa_project_session_transfer", "focusa_resource_mode"],
    })))
}

// BAD-005 fix: Field-level validation with detailed errors
// Returns Err with field-level validation_errors if request is malformed
fn validate_workpoint_checkpoint_request(
    req: &WorkpointCheckpointRequest,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut validation_errors: Vec<Value> = Vec::new();

    // Validate mission and next_slice (at least one must be present)
    if req.mission.as_deref().unwrap_or("").trim().is_empty()
        && req.next_slice.as_deref().unwrap_or("").trim().is_empty()
    {
        validation_errors.push(json!({
            "field": "mission",
            "error": "required when next_slice is missing",
            "value": req.mission,
            "severity": "error"
        }));
        validation_errors.push(json!({
            "field": "next_slice",
            "error": "required when mission is missing",
            "value": req.next_slice,
            "severity": "error"
        }));
    }

    // Validate project_root format (must be absolute path if provided)
    if let Some(root) = req.project_root.as_deref()
        && !root.trim().is_empty()
        && !root.starts_with('/')
    {
        validation_errors.push(json!({
            "field": "project_root",
            "error": "must be an absolute path starting with '/'",
            "value": root,
            "severity": "error"
        }));
    }

    // Validate mission length (max 500 chars)
    if let Some(mission) = req.mission.as_deref()
        && mission.len() > 500
    {
        validation_errors.push(json!({
            "field": "mission",
            "error": "must be <= 500 characters",
            "value_length": mission.len(),
            "severity": "error"
        }));
    }

    // Validate next_slice length (max 2000 chars)
    if let Some(next) = req.next_slice.as_deref()
        && next.len() > 2000
    {
        validation_errors.push(json!({
            "field": "next_slice",
            "error": "must be <= 2000 characters",
            "value_length": next.len(),
            "severity": "error"
        }));
    }

    // Validate continuity_id format if provided
    if let Some(cont_id) = req.continuity_id.as_deref()
        && !cont_id.trim().is_empty()
        && cont_id.len() > 256
    {
        validation_errors.push(json!({
            "field": "continuity_id",
            "error": "must be <= 256 characters",
            "value_length": cont_id.len(),
            "severity": "error"
        }));
    }

    if validation_errors.is_empty() {
        Ok(())
    } else {
        Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_error",
                "canonical": false,
                "error": "request validation failed",
                "validation_errors": validation_errors,
                "next_step_hint": "fix validation_errors and retry"
            })),
        ))
    }
}

async fn checkpoint(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut req): Json<WorkpointCheckpointRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    // BAD-005 fix: Run field-level validation before legacy checks
    validate_workpoint_checkpoint_request(&req)?;
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

    if req.preview {
        if let Some(rejection) =
            session_identity_requires_project_root_confirmation(req.session_identity.as_ref())
        {
            return Err(rejection);
        }
        apply_checkpoint_session_identity(&mut req);
        let workpoint_id = req.workpoint_id.unwrap_or_else(Uuid::now_v7);
        let requested_canonical = req.canonical.unwrap_or(true);
        let safe_scope = !requested_canonical
            || (unsafe_project_root_reason(req.project_root.as_deref()).is_none()
                && clean_resume_scope_value(req.continuity_id.as_deref()).is_some());
        return Ok(Json(json!({
            "status": "preview",
            "canonical": false,
            "preview": true,
            "side_effects": [],
            "trust_badges": trust_badges(false, false, false, false, true, !safe_scope),
            "workpoint_id": workpoint_id,
            "mutation_preview": checkpoint_mutation_preview(&req, workpoint_id, safe_scope),
            "next_step_hint": "preview only; repeat without preview/dry_run to apply this Workpoint checkpoint"
        })));
    }

    if let Some(key) = req
        .idempotency_key
        .as_ref()
        .filter(|key| !key.trim().is_empty())
    {
        // #125 migration pattern: partitioned workstream state first.
        let scoped_state = match (req.project_root.as_deref(), req.continuity_id.as_deref()) {
            (Some(root), Some(continuity)) => Some(
                state.workstream_states.get_or_create(root, continuity).await,
            ),
            _ => None,
        };
        let focusa_guard;
        let focusa = match &scoped_state {
            Some(partition) => {
                focusa_guard = partition.read().await;
                &focusa_guard
            }
            None => {
                focusa_guard = state.focusa.read().await;
                &focusa_guard
            }
        };
        if let Some(existing) = focusa.workpoint.records.iter().find(|record| {
            record.idempotency_key.as_deref() == Some(key.as_str())
                && record.project_root.as_deref() == req.project_root.as_deref()
                && record.continuity_id.as_deref() == req.continuity_id.as_deref()
        }) {
            return Ok(Json(json!({
                "status": "completed",
                "workpoint_id": existing.workpoint_id,
                "canonical": existing.canonical,
                "idempotent_replay": true,
                "idempotency_cache": idempotency_cache_status_payload(),
                "workpoint": workpoint_packet(existing),
                "warnings": [],
                "next_step_hint": "idempotency key already recorded; call /v1/workpoint/resume to render the packet"
            })));
        }
    }

    if let Some(rejection) =
        session_identity_requires_project_root_confirmation(req.session_identity.as_ref())
    {
        return Err(rejection);
    }
    apply_checkpoint_session_identity(&mut req);
    let workpoint_id = req.workpoint_id.unwrap_or_else(Uuid::now_v7);
    let promote = req.promote.unwrap_or(true);
    let requested_canonical = req.canonical.unwrap_or(true);
    if requested_canonical {
        // #89 slice 6: a verified/stale RemoteWorkspaceBinding owning this
        // remote project root satisfies the bootstrap precondition — the
        // controller daemon manages the workstream without a local checkout.
        let binding_satisfied = match req.project_root.as_deref() {
            Some(root) => {
                let db = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
                rusqlite::Connection::open(&db)
                    .ok()
                    .and_then(|conn| {
                        focusa_core::remote_workspace::resolve_binding_for_root(&conn, root).ok()
                    })
                    .flatten()
                    .is_some()
            }
            None => false,
        };
        if !binding_satisfied {
            if let Some(reason) = unsafe_project_root_reason(req.project_root.as_deref()) {
                return Err(unsafe_checkpoint_rejection(
                    reason,
                    "project_root",
                    req.project_root.as_deref(),
                ));
            }
        }
        if clean_resume_scope_value(req.continuity_id.as_deref()).is_none() {
            return Err(unsafe_checkpoint_rejection(
                "missing_continuity_id",
                "continuity_id",
                req.continuity_id.as_deref(),
            ));
        }
    }
    let idempotency_key = req.idempotency_key.clone();
    let record = WorkpointRecord {
        workpoint_id,
        work_item_id: req.work_item_id,
        session_identity: req.session_identity,
        continuity_id: req.continuity_id,
        session_id: req.session_id,
        project_root: req.project_root,
        frame_id: req.frame_id,
        status: WorkpointStatus::Proposed,
        checkpoint_reason: parse_checkpoint_reason(req.checkpoint_reason.as_deref())?,
        confidence: req.confidence.unwrap_or(WorkpointConfidence::High),
        canonical: requested_canonical,
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
    let checkpoint_summary = checkpoint_summary(&record);

    let mut events = vec![FocusaEvent::WorkpointCheckpointProposed { workpoint: record }];
    if promote && canonical {
        events.push(FocusaEvent::WorkpointCheckpointPromoted {
            workpoint_id,
            confidence: req.confidence.unwrap_or(WorkpointConfidence::High),
            reason: "checkpoint API promote=true".to_string(),
        });
    }

    let materialized_state =
        materialize_workpoint_events(_scope.clone(), &state, events, "workpoint_checkpoint")
            .await?;
    let promoted_record = if promote && canonical {
        materialized_state
            .workpoint
            .records
            .iter()
            .find(|record| {
                record.workpoint_id == workpoint_id && record.status == WorkpointStatus::Active
            })
            .cloned()
    } else {
        None
    };
    if promote && canonical && promoted_record.is_none() {
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
    if let (Some(key), Some(record)) = (
        idempotency_key
            .as_ref()
            .filter(|key| !key.trim().is_empty()),
        promoted_record.as_ref(),
    ) {}

    Ok(Json(json!({
        "status": if promote && canonical { "accepted" } else { "partial" },
        "workpoint_id": workpoint_id,
        "canonical": canonical,
        "trust_badges": trust_badges(canonical, false, false, false, !promote || !canonical, false),
        "idempotent_replay": false,
        "idempotency_cache": idempotency_cache_status_payload(),
        "workpoint": promoted_record.as_ref().map(workpoint_packet),
        "checkpoint_summary": checkpoint_summary.clone(),
        "rendered_summary": checkpoint_summary.get("one_line").cloned().unwrap_or(Value::Null),
        "rollback_card": rollback_card_payload(
            json!({"snapshot_id": materialized_state.clt.head_id, "source": "current_clt_head"}),
            Some(workpoint_id),
            promoted_record.as_ref().and_then(|record| record.project_root.as_deref()),
            promoted_record.as_ref().and_then(|record| record.continuity_id.as_deref()),
            "workpoint_checkpoint",
            "active Workpoint and linked evidence return to the selected safe snapshot scope"
        ),
        "warnings": if promote && !canonical { vec!["non-canonical checkpoint was proposed but not promoted"] } else { vec![] },
        "next_step_hint": "checkpointed typed mission/action/next_slice; call /v1/workpoint/resume to render the packet for Pi continuation"
    })))
}

async fn idempotency_cache_status(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    Ok(Json(idempotency_cache_status_payload()))
}

async fn current(
    scope: ScopeContext,
    Query(query): Query<WorkpointCurrentQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    // Auto-detect project_root from PWD when not provided (Spec 109 AX +
    // transcript gap 6: `focusa workpoint current` (no args) should
    // discover scope from .focusa-project.json walking up from CWD).
    let detected_working_context = std::env::current_dir()
        .ok()
        .and_then(|cwd| resolve_git_working_context(&cwd).ok().flatten());
    let detected_project_root = detected_working_context
        .as_ref()
        .map(|context| context.canonical_parent_root.clone())
        .or_else(detect_project_root_from_cwd);
    let effective_project_root = scope
        .project_root
        .clone()
        .or_else(|| {
            query
                .project_root
                .as_deref()
                .and_then(|value| clean_resume_scope_value(Some(value)))
        })
        .or(detected_project_root);
    let effective_working_subpath_id = query
        .working_subpath_id
        .as_deref()
        .and_then(|value| clean_resume_scope_value(Some(value)))
        .or(scope.working_subpath_id.clone())
        .or_else(|| {
            detected_working_context
                .as_ref()
                .map(|context| context.working_subpath.working_subpath_id.clone())
        });
    let effective_continuity_id = query
        .continuity_id
        .as_deref()
        .and_then(|value| clean_resume_scope_value(Some(value)));
    // If project_root was auto-detected but no continuity_id was passed,
    // look up the continuity_id from the .focusa-project.json marker.
    let effective_continuity_id =
        if effective_continuity_id.is_none() && effective_project_root.is_some() {
            read_continuity_id_from_marker(&effective_project_root.clone().unwrap())
        } else {
            effective_continuity_id
        };
    // Convenience: if no scope resolved, try selected project as fallback.
    let (effective_project_root, effective_continuity_id, selected_project_used) =
        if effective_project_root.is_none() {
            if let Some(selected) = crate::routes::project::selected_project_payload() {
                let sel_root = selected
                    .get("project_root")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let sel_cont = read_continuity_id_from_marker(&sel_root);
                (Some(sel_root), sel_cont, true)
            } else {
                (effective_project_root, effective_continuity_id, false)
            }
        } else {
            (effective_project_root, effective_continuity_id, false)
        };
    // #125 migration pattern: partitioned workstream state first, global
    // fallback for unmigrated scopes. Every migrated route follows this
    // exact pattern until the global state is retired.
    let scoped_state = match (&effective_project_root, &effective_continuity_id) {
        (Some(root), Some(continuity)) => Some(
            state
                .workstream_states
                .get_or_create(root, continuity)
                .await,
        ),
        _ => None,
    };
    let focusa_guard;
    let focusa = match &scoped_state {
        Some(partition) => {
            focusa_guard = partition.read().await;
            &focusa_guard
        }
        None => {
            focusa_guard = state.focusa.read().await;
            &focusa_guard
        }
    };
    let Some(record) = active_workpoint_for_context(
        focusa,
        effective_project_root.as_deref(),
        effective_continuity_id.as_deref(),
        effective_working_subpath_id.as_deref(),
    ) else {
        let mut payload = json!({
            "status": "not_found",
            "canonical": false,
            "workpoint_id": null,
            "warnings": ["no active workpoint matches this scope"],
            "next_step_hint": "POST /v1/workpoint/checkpoint with --project-root=<abs> and --continuity-id=<id> before compacting or resuming"
        });
        if let Some(p) = &effective_project_root {
            payload.as_object_mut().unwrap().insert(
                "detected_project_root".to_string(),
                serde_json::Value::String(p.clone()),
            );
        }
        if let Some(c) = &effective_continuity_id {
            payload.as_object_mut().unwrap().insert(
                "detected_continuity_id".to_string(),
                serde_json::Value::String(c.clone()),
            );
        }
        payload.as_object_mut().unwrap().insert(
            "recovery_hint".to_string(),
            serde_json::Value::String(
                "pass --project-root explicitly if PWD is not the project; if the workpoint was created in a different cwd, run focusa workpoint current --project-root <that-path>".to_string()
            ),
        );
        return Ok(Json(payload));
    };
    Ok(Json(current_workpoint_payload(record)))
}

/// Walk up from PWD looking for `.focusa-project.json`; return the directory
/// containing it. This is the universal scope-detection heuristic for
/// workpoint current/resume. Mirrors the daemon's project_identity
/// detection at startup.
fn detect_project_root_from_cwd() -> Option<String> {
    let mut cur = std::env::current_dir().ok()?;
    loop {
        let marker = cur.join(".focusa-project.json");
        if marker.is_file() {
            return cur.to_str().map(|s| s.to_string());
        }
        match cur.parent() {
            Some(parent) if parent != cur => cur = parent.to_path_buf(),
            _ => return None,
        }
    }
}

/// Read the continuity_id from `.focusa-project.json` if present.
fn read_continuity_id_from_marker(project_root: &str) -> Option<String> {
    let path = std::path::Path::new(project_root).join(".focusa-project.json");
    let body = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    json.get("continuity_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn resource_mode_resume_payload() -> Value {
    let status = resource_mode_status();
    json!({
        "resource_mode": status.mode,
        "resource_reason": status.reason,
        "lowmem_budget": status.budget,
        "context_posture": if status.mode == "normal" { "normal_bounded" } else { "surgical_summary_only" },
        "cold_surfaces_deferred": status.cold_surfaces_deferred,
        "pruning_order": status.pruning_order,
        "retention_policy": status.retention_policy,
        "pruned_counts": {
            "transition_omitted_count": status.transition_omitted_count,
            "rehydrate_ref_budget": status.budget.max_rehydrate_refs,
        },
        "targeted_rehydration": {
            "preferred_tool": "focusa_traverse",
            "guidance": "request narrow slices by surface/selector/anchor/cursor/fields; avoid full payloads by default"
        }
    })
}

fn trajectory_resume_projection(
    record: &WorkpointRecord,
    scope: &ResumeScopeDecision,
    canonical: bool,
) -> Value {
    let high_level_goal = record.mission.as_deref().unwrap_or("unspecified mission");
    let mid_level_goal = record
        .action_intent
        .as_ref()
        .map(|intent| intent.action_type.as_str())
        .unwrap_or("resume_workpoint");
    let low_level_goal = record
        .next_slice
        .as_deref()
        .unwrap_or("continue current bounded next action");
    json!({
        "advisory_only": true,
        "identity_policy": "trajectory similarity can group sessions, but project_root plus continuity_id controls authority",
        "project_identity": {
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "session_id": record.session_id,
            "scope_canonical": canonical,
            "scope_warnings": scope.warnings.clone(),
        },
        "hierarchy": {
            "high_level_goal": high_level_goal,
            "mid_level_goal": mid_level_goal,
            "low_level_goal": low_level_goal,
            "similarity_grouping": "advisory_only",
            "must_not_merge_on_similarity": true,
        },
        // Spec 125 §6.2/9.2: Workpoint carries trajectory warning when HLT is invalid.
        // Workpoint remains immediate authority but cannot hide invalid HLT.
        "trajectory_warning": {
            "hlt_status": "unknown_from_workpoint",
            "note": "Workpoint is immediate action authority; trajectory HLT status must be verified via focusa_trajectory_view",
            "recommended_verification": ["focusa_trajectory_view", "focusa_hlt_history"],
        },
        "active_gap": record.next_slice,
        "workpoint_candidate": {
            "workpoint_id": record.workpoint_id,
            "work_item_id": record.work_item_id,
            "advisory_only": true,
        }
    })
}

fn traversal_resume_slices(record: &WorkpointRecord) -> Vec<Value> {
    let evidence_refs = record
        .verification_records
        .iter()
        .take(8)
        .filter_map(|verification| verification.evidence_ref.clone())
        .collect::<Vec<_>>();
    vec![
        json!({
            "surface": "workpoints",
            "selector": "current",
            "anchor": record.workpoint_id,
            "returned": 1,
            "truncated": false,
            "tool": "focusa_traverse",
            "tags": [
                {"tag": format!("workpoint:{}", record.workpoint_id), "surface": "workpoints", "verified": true}
            ],
            "window_tag": format!("workpoints/current:{}", record.workpoint_id),
            "rehydrate_refs": [format!("workpoint:{}", record.workpoint_id)],
        }),
        json!({
            "surface": "lineage",
            "selector": "path",
            "anchor": record.frame_id,
            "returned": 0,
            "truncated": false,
            "tool": "focusa_traverse",
            "status": "candidate_until_resolved",
            "tags": [
                {"tag": record.frame_id.map(|frame_id| format!("frame:{}", frame_id)).unwrap_or_else(|| "frame:unbound".to_string()), "surface": "lineage", "verified": record.frame_id.is_some()}
            ],
            "window_tag": record.frame_id.map(|frame_id| format!("lineage/path:{}", frame_id)),
            "rehydrate_refs": record.frame_id.map(|frame_id| vec![format!("frame:{}", frame_id)]).unwrap_or_default(),
        }),
        json!({
            "surface": "evidence",
            "selector": "recent",
            "returned": record.verification_records.len().min(8),
            "truncated": record.verification_records.len() > 8,
            "tool": "focusa_traverse",
            "tags": evidence_refs.iter().map(|evidence_ref| json!({"tag": evidence_ref, "surface": "evidence", "verified": true})).collect::<Vec<_>>(),
            "window_tag": format!("evidence/recent:{}", record.workpoint_id),
            "rehydrate_refs": evidence_refs,
        }),
    ]
}

fn tool_affordances_v2() -> Value {
    let do_not_use = vec![
        "full lineage tree",
        "full ontology graph",
        "full telemetry logs",
        "transcript tail as authority",
    ];
    json!({
        "best_next": [
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_traverse",
            "focusa_active_object_resolve",
            "focusa_evidence_capture",
            "focusa_tool_doctor"
        ],
        "recovery": [
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_traverse",
            "focusa_tool_doctor"
        ],
        "do_not_use": do_not_use,
        "do_not_use_by_default": do_not_use,
    })
}

fn workpoint_identity_axes_payload(record: &WorkpointRecord, canonical: bool) -> Value {
    json!({
        "projection_kind": "workpoint_identity_axes_v1",
        "authority_gate": "project_root_plus_continuity_id",
        "advisory_only": true,
        "project": {"project_root": record.project_root, "authority_role": "project_folder_boundary"},
        "logical_workstream": {"continuity_id": record.continuity_id, "authority_role": "logical_session_boundary"},
        "adapter_session": {"session_id": record.session_id, "authority_role": "temporal_metadata_only"},
        "workpoint_continuation_card": {
            "workpoint_id": record.workpoint_id,
            "work_item_id": record.work_item_id,
            "canonical": canonical,
            "mission": record.mission,
            "next_slice": record.next_slice,
        },
        "do_not_use": ["session_id_as_authority_gate", "trajectory_similarity_as_resume_authority"],
        "rehydrate_refs": [
            {"tool":"focusa_workpoint_resume", "workpoint_id": record.workpoint_id},
            {"tool":"focusa_trajectory_view", "project_root": record.project_root},
            {"tool":"focusa_traverse", "surface":"ontology", "selector":"active_context"}
        ]
    })
}

fn packet_resume_source(record: &WorkpointRecord, req: &WorkpointResumeRequest) -> String {
    req.session_identity
        .as_ref()
        .or(record.session_identity.as_ref())
        .map(|identity| identity.resume_source.trim().to_string())
        .filter(|source| !source.is_empty())
        .or_else(|| {
            req.mode
                .as_ref()
                .map(|mode| format!("workpoint_resume:{mode}"))
        })
        .unwrap_or_else(|| "workpoint_resume".to_string())
}

fn project_identity_payload(
    record: &WorkpointRecord,
    req: &WorkpointResumeRequest,
    canonical: bool,
) -> Value {
    if let Some(project_identity) = req
        .session_identity
        .as_ref()
        .or(record.session_identity.as_ref())
        .and_then(|identity| identity.project_identity.as_ref())
    {
        return json!(project_identity);
    }
    let project_root = record
        .project_root
        .clone()
        .or_else(|| req.project_root.clone())
        .unwrap_or_default();
    json!({
        "schema": "focusa.project_identity.v1",
        "status": if canonical { "matched" } else { "degraded" },
        "project_root": project_root,
        "confidence": if canonical { "high" } else { "low" },
        "authority_boundary": "project_root_plus_continuity_id",
        "source": "workpoint_record_fallback"
    })
}

fn session_identity_payload(
    record: &WorkpointRecord,
    req: &WorkpointResumeRequest,
    generated_at: &str,
    canonical: bool,
) -> Value {
    if let Some(identity) = req
        .session_identity
        .as_ref()
        .or(record.session_identity.as_ref())
    {
        return json!(identity);
    }
    let project_root = record
        .project_root
        .clone()
        .or_else(|| req.project_root.clone())
        .unwrap_or_default();
    let session_frame_key = req
        .session_id
        .clone()
        .or_else(|| record.session_id.clone())
        .unwrap_or_else(|| "unknown-session".to_string());
    json!({
        "schema": "focusa.session_identity.v1",
        "project_identity": Value::Null,
        "pi_session_id": record.session_id,
        "session_frame_key": session_frame_key,
        "session_incarnation_id": format!("{}:workpoint_resume", session_frame_key),
        "continuity_id": record.continuity_id,
        "project_root": project_root,
        "cwd": project_root,
        "workspace_id": project_root,
        "process_id": Value::Null,
        "started_at": record.created_at.as_ref().map(|created_at| created_at.to_rfc3339()).unwrap_or_else(|| generated_at.to_string()),
        "resume_source": packet_resume_source(record, req),
        "canonical_scope": canonical,
        "scope_failure": if canonical { Value::Null } else { json!("scope_unbound_or_non_canonical") },
    })
}

fn workpoint_action_type(record: &WorkpointRecord) -> String {
    record
        .action_intent
        .as_ref()
        .map(|intent| intent.action_type.clone())
        .unwrap_or_else(|| "resume_workpoint".to_string())
}

fn safest_next_action(record: &WorkpointRecord) -> String {
    record
        .next_slice
        .clone()
        .unwrap_or_else(|| "continue current bounded next action".to_string())
}

fn resume_confidence_level(canonical: bool, identity_confidence: &Value) -> &'static str {
    let percent = identity_confidence
        .get("percent")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if canonical && percent >= 80 {
        "high"
    } else if canonical || percent >= 60 {
        "medium"
    } else {
        "low"
    }
}

fn workpoint_v2_payload(record: &WorkpointRecord, canonical: bool) -> Value {
    let verification_hooks = record
        .action_intent
        .as_ref()
        .map(|intent| intent.verification_hooks.clone())
        .unwrap_or_default();
    let verified_evidence_refs = record
        .verification_records
        .iter()
        .take(8)
        .filter_map(|verification| verification.evidence_ref.clone())
        .collect::<Vec<_>>();
    let blockers = record
        .blockers
        .iter()
        .map(|blocker| blocker.reason.clone())
        .collect::<Vec<_>>();
    json!({
        "status": if canonical { "active" } else { "degraded" },
        "raw_status": record.status,
        "workpoint_id": record.workpoint_id,
        "work_item_id": record.work_item_id,
        "project_root": record.project_root,
        "continuity_id": record.continuity_id,
        "session_id": record.session_id,
        "mission": record.mission.as_deref().unwrap_or("unknown mission"),
        "action_intent": record.action_intent,
        "active_object_refs": record.active_object_refs.iter().take(8).cloned().collect::<Vec<_>>(),
        "blockers": blockers,
        "blocker_details": record.blockers,
        "drift_boundaries": ["Trajectory similarity is advisory grouping only; session authority remains project_root plus continuity_id."],
        "do_not_drift": ["project_root_plus_continuity_id_authority", "trajectory_similarity_advisory_only", "transcript_tail_not_authority"],
        "verification_hooks": verification_hooks,
        "verified_evidence_refs": verified_evidence_refs,
        "next_action": safest_next_action(record),
        "next_slice": record.next_slice,
        "updated_at": record.updated_at.as_ref().map(|updated_at| updated_at.to_rfc3339()),
    })
}

fn resume_summary_v2(
    record: &WorkpointRecord,
    summary: &str,
    canonical: bool,
    scope: &ResumeScopeDecision,
) -> Value {
    let mut warnings = scope.warnings.clone();
    if !canonical {
        warnings.push(
            "resume packet is degraded until project_root plus continuity_id are canonical"
                .to_string(),
        );
    }
    json!({
        "one_line": summary,
        "mission": record.mission.as_deref().unwrap_or("unknown mission"),
        "current_action": workpoint_action_type(record),
        "short_term_goal": safest_next_action(record),
        "long_term_goal": record.mission,
        "desired_end_state": Value::Null,
        "current_verified_state": if canonical { "canonical Workpoint scope verified" } else { "Workpoint scope degraded or unbound" },
        "current_state_delta": "resume packet generated from Workpoint state with advisory Trajectory/traversal projections",
        "gap": record.next_slice,
        "why_this_next": record.next_slice.as_deref().unwrap_or("bounded Workpoint continuation is the safest known next action"),
        "safest_next_action": safest_next_action(record),
        "context_sufficiency": {
            "status": if canonical { "sufficient_for_next_action" } else { "requires_reorientation" },
            "reason": if canonical { "project_root plus continuity_id context is canonical" } else { "canonical project context not proven" },
            "next_tools": ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"]
        },
        "warnings": warnings,
        "do_not_use": ["transcript tail as authority", "session_id as authority gate", "trajectory similarity as resume authority"],
    })
}

fn explicit_project_path_from_ask(ask: &str) -> Option<String> {
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

fn current_ask_scope_conflict_reason(
    record: &WorkpointRecord,
    req: &WorkpointResumeRequest,
) -> Option<String> {
    let ask = clean_resume_scope_value(req.current_ask.as_deref())?;
    let lower = ask.to_lowercase();
    let saved_root = record
        .project_root
        .as_deref()
        .unwrap_or("")
        .trim()
        .trim_end_matches('/');
    if let Some(path) = explicit_project_path_from_ask(&ask) {
        let normalized = path.trim_end_matches('/');
        if !saved_root.is_empty() && normalized != saved_root {
            return Some(format!(
                "operator named different project path {normalized}"
            ));
        }
    }
    if [
        "wrong place",
        "not this repo",
        "not this project",
        "different project",
        "remote project",
        "switch project",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
    {
        return Some("operator text indicates current project/root may be wrong".to_string());
    }
    if (lower.contains("ptm") || lower.contains("planmarr") || lower.contains("plan-the-marriage"))
        && !saved_root.contains("planmarr")
        && !saved_root.contains("plan-the-marriage")
    {
        return Some("operator text names PTM/planmarr while saved scope is different".to_string());
    }
    None
}

fn current_ask_action_authority_payload(
    record: &WorkpointRecord,
    canonical_for_saved_scope: bool,
    req: &WorkpointResumeRequest,
) -> Value {
    let conflict = current_ask_scope_conflict_reason(record, req);
    let matches_current_ask_scope = conflict.is_none();
    json!({
        "canonical_for_saved_scope": canonical_for_saved_scope,
        "matches_current_ask_scope": matches_current_ask_scope,
        "action_authority_for_current_ask": canonical_for_saved_scope && matches_current_ask_scope,
        "scope_conflict_reason": conflict.unwrap_or_else(|| "none".to_string()),
        "current_ask_present": clean_resume_scope_value(req.current_ask.as_deref()).is_some(),
        "authority_boundary": "saved_scope_plus_current_ask_scope",
    })
}

fn current_ask_scope_rejection(
    record: &WorkpointRecord,
    req: &WorkpointResumeRequest,
) -> Option<Value> {
    let reason = current_ask_scope_conflict_reason(record, req)?;
    Some(json!({
        "status": "rejected_current_ask_scope_conflict",
        "canonical": false,
        "canonical_for_saved_scope": record.canonical,
        "matches_current_ask_scope": false,
        "action_authority_for_current_ask": false,
        "failure_class": "scope_conflict",
        "workpoint_id": record.workpoint_id,
        "project_root": record.project_root,
        "continuity_id": record.continuity_id,
        "current_ask_scope": current_ask_action_authority_payload(record, record.canonical, req),
        "scope_conflict_reason": reason,
        "warnings": ["current ask names or implies a different project scope than the resumed Workpoint"],
        "safe_recovery": "verify project identity, cd to the intended project root, then create or resume a Workpoint in that scope",
        "next_tools": ["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint"],
        "next_step_hint": "hard stop: do not execute resumed Workpoint actions until current ask scope matches saved project_root plus continuity_id"
    }))
}

fn workpoint_legacy_migration_warnings(record: &WorkpointRecord) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if record
        .project_root
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
        || record
            .continuity_id
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        warnings.push("old_workpoint_packet_missing_scope");
    }
    if record.session_identity.is_none() {
        warnings.push("old_workpoint_packet_missing_session_identity");
    }
    warnings
}

fn workpoint_scope_status(record: &WorkpointRecord, canonical: bool) -> &'static str {
    if canonical {
        "verified"
    } else if !workpoint_legacy_migration_warnings(record).is_empty() {
        "missing"
    } else {
        "mismatch_candidate"
    }
}

fn workpoint_migration_posture(record: &WorkpointRecord, canonical: bool) -> Value {
    let warnings = workpoint_legacy_migration_warnings(record);
    json!({
        "migration_class": "old_workpoint_packets",
        "read_behavior": if warnings.is_empty() { "current_packet" } else { "readable_as_degraded_advisory_recovery_packet" },
        "authority_status": if canonical { "canonical_for_verified_project_root_plus_continuity_id" } else { "canonical_false_until_project_root_plus_continuity_id_rebound" },
        "migration_warnings": warnings,
        "promotion_path": ["focusa_project_identity", "focusa_project_verify", "focusa_workpoint_checkpoint"],
    })
}

#[allow(clippy::too_many_arguments)]
fn workpoint_resume_packet_v2(
    record: &WorkpointRecord,
    _packet: Value,
    summary: &str,
    canonical: bool,
    session_continuity: Value,
    identity_confidence: Value,
    scope: &ResumeScopeDecision,
    req: &WorkpointResumeRequest,
) -> Value {
    let generated_at = Utc::now().to_rfc3339();
    let failure_class = if canonical {
        Value::Null
    } else {
        json!("scope_unbound_or_non_canonical")
    };
    let current_ask_scope = current_ask_action_authority_payload(record, canonical, req);
    let action_authority = current_ask_scope
        .get("action_authority_for_current_ask")
        .and_then(Value::as_bool)
        .unwrap_or(canonical);
    let current_action_failure_class = if canonical && !action_authority {
        json!("scope_conflict")
    } else {
        failure_class.clone()
    };
    let next_tools = if action_authority {
        json!([
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_traverse",
            "focusa_active_object_resolve"
        ])
    } else {
        json!([
            "focusa_project_verify",
            "focusa_project_identity",
            "focusa_workpoint_checkpoint",
            "focusa_workpoint_resume"
        ])
    };
    let migration_posture = workpoint_migration_posture(record, canonical);
    let handoff_quality = handoff_quality_payload(record, canonical, action_authority);
    let tool_result = json!({
        "ok": canonical && action_authority,
        "status": "completed",
        "canonical": canonical,
        "advisory": !canonical || !action_authority,
        "canonical_for_saved_scope": canonical,
        "matches_current_ask_scope": current_ask_scope.get("matches_current_ask_scope").cloned().unwrap_or(json!(true)),
        "action_authority_for_current_ask": action_authority,
        "scope_conflict_reason": current_ask_scope.get("scope_conflict_reason").cloned().unwrap_or(json!("none")),
        "degraded": !canonical || !action_authority,
        "stale": !canonical,
        "scope": {
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "workpoint_id": record.workpoint_id,
            "session_id": record.session_id,
            "scope_status": workpoint_scope_status(record, canonical),
            "scope_source": if canonical { "focusa_verified" } else { "legacy_or_request_scope" },
        },
        "failure_class": current_action_failure_class.clone(),
        "retry": {"safe": action_authority, "posture": if action_authority { "safe_retry" } else { "do_not_retry_unchanged" }},
        "migration_posture": migration_posture.clone(),
        "migration_warnings": migration_posture.get("migration_warnings").cloned().unwrap_or_else(|| json!([])),
        "side_effects": ["workpoint_resume_rendered"],
        "evidence_refs": record.verification_records.iter().take(8).filter_map(|v| v.evidence_ref.clone()).collect::<Vec<_>>(),
        "next_tools": next_tools.clone(),
    });
    json!({
        "schema_version": "focusa.workpoint_resume_packet.v2",
        "packet_id": Uuid::now_v7(),
        "generated_at": generated_at,
        "resume_source": packet_resume_source(record, req),
        "canonical": canonical,
        "degraded": !canonical || !action_authority,
        "trust_badges": trust_badges(canonical, !canonical || !action_authority, false, false, false, canonical && !action_authority),
        "route_recommendation": route_recommendation_payload(canonical, action_authority),
        "canonical_for_saved_scope": current_ask_scope.get("canonical_for_saved_scope").cloned().unwrap_or(json!(canonical)),
        "matches_current_ask_scope": current_ask_scope.get("matches_current_ask_scope").cloned().unwrap_or(json!(true)),
        "action_authority_for_current_ask": current_ask_scope.get("action_authority_for_current_ask").cloned().unwrap_or(json!(canonical)),
        "scope_conflict_reason": current_ask_scope.get("scope_conflict_reason").cloned().unwrap_or(json!("none")),
        "current_ask_scope": current_ask_scope,
        "confidence": resume_confidence_level(canonical, &identity_confidence),
        "failure_class": current_action_failure_class.clone(),
        "migration_posture": migration_posture,
        "migration_warnings": tool_result.get("migration_warnings").cloned().unwrap_or_else(|| json!([])),
        "project_identity": project_identity_payload(record, req, canonical),
        "session_identity": session_identity_payload(record, req, &generated_at, canonical),
        "project_root": record.project_root,
        "continuity_id": record.continuity_id,
        "workpoint_id": record.workpoint_id,
        "work_item_id": record.work_item_id,
        "rendered_summary": summary,
        "handoff_quality": handoff_quality,
        "resume_summary": resume_summary_v2(record, summary, canonical, scope),
        "workpoint": workpoint_v2_payload(record, canonical),
        "identity_axes": workpoint_identity_axes_payload(record, canonical),
        "trajectory": trajectory_resume_projection(record, scope, canonical),
        "traversal_slices": traversal_resume_slices(record),
        "resource_mode": resource_mode_resume_payload(),
        "tool_affordances": tool_affordances_v2(),
        "freshness": {
            "rendered_at": generated_at,
            "read_model_status": "current_snapshot",
            "source": "workpoint_resume_packet_v2_renderer",
            "stale": false,
        },
        "api_provenance": [
            {"tool_or_route": "focusa_workpoint_resume", "route": "/v1/workpoint/resume", "purpose": "render active Workpoint continuation", "status": "completed", "canonical": canonical, "failure_class": current_action_failure_class.clone(), "freshness": "live", "tool_result_v1": tool_result.clone()},
            {"tool_or_route": "focusa_trajectory_view", "route": "/v1/trajectory/view", "purpose": "advisory goal/state/gap projection", "status": "projected", "canonical": false, "advisory_only": true, "failure_class": Value::Null, "freshness": "cached", "tool_result_v1": {"ok": true, "status": "projected", "canonical": false, "advisory": true, "degraded": false, "stale": false, "scope": {"project_root": record.project_root, "continuity_id": record.continuity_id, "scope_status": if canonical { "verified" } else { "partial" }, "scope_source": "workpoint_projection"}, "failure_class": Value::Null, "retry": {"safe": true, "posture": "safe_retry"}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_trajectory_view"]}},
            {"tool_or_route": "focusa_traverse", "route": "/v1/traverse", "purpose": "bounded supporting slice descriptors", "status": "candidate_slices", "canonical": false, "advisory_only": true, "failure_class": Value::Null, "freshness": "unknown", "tool_result_v1": {"ok": true, "status": "candidate_slices", "canonical": false, "advisory": true, "degraded": false, "stale": true, "scope": {"project_root": record.project_root, "continuity_id": record.continuity_id, "scope_status": if canonical { "verified" } else { "partial" }, "scope_source": "bounded_supporting_slice"}, "failure_class": Value::Null, "retry": {"safe": true, "posture": "safe_retry"}, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_traverse"]}}
        ],
        "details": {"tool_result_v1": tool_result},
        "session_continuity": session_continuity,
        "identity_confidence": identity_confidence,
        "next_tools": next_tools,
    })
}

async fn resume(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut req): Json<WorkpointResumeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:read") {
        return Err(forbid("work-loop:read"));
    }
    apply_resume_session_identity(&mut req);
    let focusa = workstream_scoped_state(state.clone(), &_scope).await;
    let requested_workpoint_id = req.workpoint_id;
    let requested_record = requested_workpoint_id.and_then(|id| {
        focusa
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    });
    let requested_working_subpath_id =
        session_identity_working_subpath_id(req.session_identity.as_ref());
    let fallback_record = if requested_record.is_none() {
        active_workpoint_for_context(
            &focusa,
            req.project_root.as_deref(),
            req.continuity_id.as_deref(),
            requested_working_subpath_id.as_deref(),
        )
    } else {
        None
    };
    let record = requested_record.or(fallback_record);
    let requested_id_miss = requested_workpoint_id.is_some() && requested_record.is_none();
    let Some(record) = record else {
        return Ok(Json(json!({
            "status": "not_found",
            "canonical": false,
            "workpoint_id": null,
            "requested_workpoint_id": requested_workpoint_id,
            "warnings": ["no workpoint available to resume"],
            "requested_found": requested_workpoint_id.is_none(),
            "scope_found": false,
            "fallback_used": false,
            "canonical_for_requested_scope": false,
            "canonical_for_fallback_scope": false,
            "wrong_id_taxonomy": wrong_id_taxonomy_payload(WrongIdTaxonomy {
                status: "not_found_no_scope_fallback",
                workpoint_id: None,
                requested_workpoint_id,
                requested_found: requested_workpoint_id.is_none(),
                scope_found: false,
                fallback_used: false,
                canonical_for_requested_scope: false,
                canonical_for_fallback_scope: false,
            }),
            "next_step_hint": "checkpoint the current mission/action before retrying resume"
        })));
    };
    let scope = evaluate_resume_scope(
        record,
        req.project_root.as_deref(),
        req.continuity_id.as_deref(),
        req.session_id.as_deref(),
        requested_working_subpath_id.as_deref(),
    );
    if let Some(rejection) = scope.rejection {
        return Ok(Json(rejection));
    }
    if let Some(rejection) = current_ask_scope_rejection(record, &req) {
        return Ok(Json(rejection));
    }
    let workpoint_id = record.workpoint_id;
    let canonical = record.canonical && scope.canonical_scope_ok;
    let session_continuity = json!({
        "session_changed": scope.session_changed,
        "expected_session_id": scope.expected_session_id,
        "packet_session_id": scope.packet_session_id,
        "continuity_id": scope.packet_continuity_id,
        "expected_continuity_id": scope.expected_continuity_id,
        "policy": "project_root_and_continuity_id_preserve_post_compaction_continuity"
    });
    let identity_confidence = identity_confidence_payload(record, &scope, &req);
    let mismatch_warnings = scope.warnings.clone();
    let packet = workpoint_packet(record);
    let summary = resume_summary(record);
    let packet_v2 = workpoint_resume_packet_v2(
        record,
        packet.clone(),
        &summary,
        canonical,
        session_continuity.clone(),
        identity_confidence.clone(),
        &scope,
        &req,
    );
    drop(focusa);

    let resume_render_dispatch_warning = match dispatch_event(
        _scope.clone(),
        &state,
        FocusaEvent::WorkpointResumeRendered {
            workpoint_id: Some(workpoint_id),
            mode: req.mode.unwrap_or_else(|| "compact_prompt".to_string()),
            rendered_summary: summary.clone(),
        },
    )
    .await
    {
        Ok(()) => None,
        Err((_status, Json(body))) => Some(json!({
            "warning": "resume render telemetry event was not enqueued; returning the already-rendered canonical packet instead of blocking continuation",
            "dispatch_result": body,
            "recovery": "resume packet is usable; run focusa_resource_mode or focusa_tool_doctor if telemetry dispatch remains saturated"
        })),
    };

    let tool_result = packet_v2
        .pointer("/details/tool_result_v1")
        .cloned()
        .unwrap_or_else(|| json!({"ok": canonical, "status": "completed", "canonical": canonical, "degraded": !canonical}));
    let failure_class = tool_result
        .get("failure_class")
        .cloned()
        .unwrap_or(Value::Null);
    let action_authority = packet_v2
        .get("action_authority_for_current_ask")
        .and_then(Value::as_bool)
        .unwrap_or(canonical);
    let response_next_tools = packet_v2.get("next_tools").cloned().unwrap_or_else(|| {
        json!([
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_traverse",
            "focusa_active_object_resolve"
        ])
    });
    let mut warnings = mismatch_warnings;
    if !canonical {
        warnings.push("resume packet is non-canonical fallback because project folder/continuity context is unbound or packet is non-canonical".to_string());
    }
    if resume_render_dispatch_warning.is_some() {
        warnings.push("resume render telemetry dispatch degraded; packet returned from read model to preserve continuation".to_string());
    }
    let mut response = Map::new();
    response.insert("status".to_string(), json!("completed"));
    response.insert(
        "schema_version".to_string(),
        json!("focusa.workpoint_resume_packet.v2"),
    );
    response.insert("workpoint_id".to_string(), json!(workpoint_id));
    response.insert("canonical".to_string(), json!(canonical));
    response.insert(
        "degraded".to_string(),
        json!(!canonical || !action_authority),
    );
    response.insert(
        "trust_badges".to_string(),
        json!(trust_badges(
            canonical,
            !canonical || !action_authority,
            false,
            false,
            false,
            canonical && !action_authority
        )),
    );
    response.insert(
        "route_recommendation".to_string(),
        packet_v2
            .get("route_recommendation")
            .cloned()
            .unwrap_or_else(|| route_recommendation_payload(canonical, action_authority)),
    );
    response.insert(
        "canonical_for_saved_scope".to_string(),
        packet_v2
            .get("canonical_for_saved_scope")
            .cloned()
            .unwrap_or(json!(canonical)),
    );
    response.insert(
        "matches_current_ask_scope".to_string(),
        packet_v2
            .get("matches_current_ask_scope")
            .cloned()
            .unwrap_or(json!(true)),
    );
    response.insert(
        "action_authority_for_current_ask".to_string(),
        packet_v2
            .get("action_authority_for_current_ask")
            .cloned()
            .unwrap_or(json!(canonical)),
    );
    response.insert(
        "scope_conflict_reason".to_string(),
        packet_v2
            .get("scope_conflict_reason")
            .cloned()
            .unwrap_or(json!("none")),
    );
    response.insert(
        "current_ask_scope".to_string(),
        packet_v2
            .get("current_ask_scope")
            .cloned()
            .unwrap_or_else(|| json!({})),
    );
    response.insert("failure_class".to_string(), failure_class);
    response.insert("resume_packet".to_string(), packet);
    response.insert("resume_packet_v2".to_string(), packet_v2.clone());
    response.insert("rendered_summary".to_string(), json!(summary));
    response.insert(
        "handoff_quality".to_string(),
        packet_v2
            .get("handoff_quality")
            .cloned()
            .unwrap_or(Value::Null),
    );
    response.insert("warnings".to_string(), json!(warnings));
    response.insert(
        "resume_render_dispatch_warning".to_string(),
        json!(resume_render_dispatch_warning),
    );
    response.insert("session_continuity".to_string(), session_continuity);
    response.insert(
        "identity_confidence".to_string(),
        identity_confidence.clone(),
    );
    response.insert(
        "identity_confidence_percent".to_string(),
        json!(
            identity_confidence
                .get("percent")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
    );
    response.insert("next_tools".to_string(), response_next_tools);
    response.insert(
        "details".to_string(),
        json!({"tool_result_v1": tool_result}),
    );
    response.insert(
        "next_step_hint".to_string(),
        json!("inject rendered_summary plus resume_packet before the next Pi turn"),
    );
    if requested_id_miss {
        response.insert(
            "requested_workpoint_id".to_string(),
            json!(requested_workpoint_id),
        );
        response.insert("requested_found".to_string(), json!(false));
        response.insert("fallback_used".to_string(), json!(true));
        response.insert("fallback_source".to_string(), json!("active_workstream"));
        response.insert("fallback_object_id".to_string(), json!(workpoint_id));
        response.insert("canonical_for_requested_scope".to_string(), json!(false));
        response.insert("canonical_for_fallback_scope".to_string(), json!(canonical));
        response.insert("scope_found".to_string(), json!(true));
        response.insert(
            "wrong_id_taxonomy".to_string(),
            wrong_id_taxonomy_payload(WrongIdTaxonomy {
                status: "fallback_from_missing_requested_id",
                workpoint_id: Some(workpoint_id),
                requested_workpoint_id,
                requested_found: false,
                scope_found: true,
                fallback_used: true,
                canonical_for_requested_scope: false,
                canonical_for_fallback_scope: canonical,
            }),
        );
        response.insert("misuse_hint".to_string(), json!("requested Workpoint id was not found; returned same-project active Workpoint as an explicit fallback, not as canonical for requested scope"));
    }
    Ok(Json(Value::Object(response)))
}

async fn resolve_active_object(
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActiveObjectResolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let focusa = workstream_scoped_state(state.clone(), &_scope).await;
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
    _scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WorkpointEvidenceLinkRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("work-loop:write") {
        return Err(forbid("work-loop:write"));
    }
    if req.target_ref.trim().is_empty() || req.result.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "status": "validation_rejected",
                "canonical": false,
                "field": "target_ref/result",
                "retry_posture": "do_not_retry_unchanged",
                "next_step_hint": "provide target_ref and result before linking Workpoint evidence"
            })),
        ));
    }
    if let Some(rejection) =
        session_identity_requires_project_root_confirmation(req.session_identity.as_ref())
    {
        return Err(rejection);
    }
    let explicit_workpoint_id = req.workpoint_id;
    let expected_working_subpath_id =
        session_identity_working_subpath_id(req.session_identity.as_ref())
            .or_else(|| clean_resume_scope_value(req.working_subpath_id.as_deref()))
            .or_else(|| clean_resume_scope_value(_scope.working_subpath_id.as_deref()))
            .unwrap_or_else(|| "primary".to_string());
    let record = if let Some(workpoint_id) = explicit_workpoint_id {
        let visible = {
            let focusa = workstream_scoped_state(state.clone(), &_scope).await;
            focusa
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == workpoint_id)
                .cloned()
        };
        match visible {
            Some(record) => Some(record),
            None => wait_for_workpoint_record(_scope.clone(), &state, workpoint_id).await,
        }
    } else {
        let expected_project_root = session_identity_project_root(req.session_identity.as_ref());
        let expected_continuity_id = session_identity_continuity_id(req.session_identity.as_ref());
        let focusa = workstream_scoped_state(state.clone(), &_scope).await;
        if expected_project_root.is_some() || expected_continuity_id.is_some() {
            active_workpoint_for_context(
                &focusa,
                expected_project_root.as_deref(),
                expected_continuity_id.as_deref(),
                Some(expected_working_subpath_id.as_str()),
            )
            .cloned()
        } else {
            active_workpoint(&focusa).cloned()
        }
    };
    let Some(record) = record else {
        if let Some(workpoint_id) = explicit_workpoint_id {
            return Err((
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "pending",
                    "canonical": false,
                    "degraded": true,
                    "workpoint_id": workpoint_id,
                    "failure_class": "read_model_lag",
                    "retry_posture": "safe_retry",
                    "retry": {"safe": true, "posture": "safe_retry", "reason": "workpoint record accepted but not visible yet"},
                    "side_effects": [],
                    "next_tools": ["focusa_workpoint_resume", "focusa_workpoint_link_evidence"],
                    "next_step_hint": "retry evidence link after Workpoint checkpoint is visible"
                })),
            ));
        }
        return Err(workpoint_no_active_to_link());
    };
    if record_working_subpath_id(&record) != expected_working_subpath_id {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "status": "rejected_scope_mismatch",
                "canonical": false,
                "failure_class": "working_subpath_mismatch",
                "expected_working_subpath_id": expected_working_subpath_id,
                "actual_working_subpath_id": record_working_subpath_id(&record),
                "next_step_hint": "link evidence from the exact Workpoint working context or perform an explicit session transfer"
            })),
        ));
    }
    if let Some(expected_project_root) =
        session_identity_project_root(req.session_identity.as_ref())
    {
        let actual = clean_resume_scope_value(record.project_root.as_deref());
        if actual.as_deref() != Some(expected_project_root.as_str()) {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "status": "rejected_scope_mismatch",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "field": "session_identity.project_root",
                    "expected_project_root": expected_project_root,
                    "packet_project_root": record.project_root.clone(),
                    "next_step_hint": "resume/checkpoint the Workpoint in the same project before linking evidence"
                })),
            ));
        }
    }
    if let Some(expected_continuity_id) =
        session_identity_continuity_id(req.session_identity.as_ref())
    {
        let actual = clean_resume_scope_value(record.continuity_id.as_deref());
        if actual.as_deref() != Some(expected_continuity_id.as_str()) {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "status": "rejected_scope_mismatch",
                    "canonical": false,
                    "failure_class": "scope_mismatch",
                    "field": "session_identity.continuity_id",
                    "expected_continuity_id": expected_continuity_id,
                    "packet_continuity_id": record.continuity_id.clone(),
                    "next_step_hint": "resume/checkpoint the Workpoint in the same continuity scope before linking evidence"
                })),
            ));
        }
    }
    let workpoint_id = record.workpoint_id;
    let verification = WorkpointVerificationRecord {
        target_ref: req.target_ref,
        result: req.result,
        evidence_ref: req.evidence_ref,
        verified_at: None,
    };
    let rollback_snapshot = {
        let focusa = workstream_scoped_state(state.clone(), &_scope).await;
        json!({"snapshot_id": focusa.clt.head_id, "source": "current_clt_head"})
    };
    if req.preview {
        return Ok(Json(json!({
            "status": "preview",
            "canonical": false,
            "preview": true,
            "side_effects": [],
            "workpoint_id": workpoint_id,
            "verification": verification,
            "mutation_preview": evidence_link_mutation_preview(&record, &verification),
            "next_step_hint": "preview only; repeat without preview/dry_run to link evidence"
        })));
    }
    let materialized_state = materialize_workpoint_events(
        _scope.clone(),
        &state,
        vec![FocusaEvent::WorkpointEvidenceLinked {
            workpoint_id,
            verification: verification.clone(),
        }],
        "workpoint_evidence_link",
    )
    .await?;
    let linked_record = materialized_state
        .workpoint
        .records
        .iter()
        .find(|record| {
            record.workpoint_id == workpoint_id
                && record.verification_records.iter().any(|linked| {
                    linked.target_ref == verification.target_ref
                        && linked.result == verification.result
                        && linked.evidence_ref == verification.evidence_ref
                })
        })
        .cloned();
    if linked_record.is_none() {
        return Err((
            StatusCode::ACCEPTED,
            Json(json!({
                "status": "pending",
                "canonical": true,
                "failure_class": "read_model_lag",
                "workpoint_id": workpoint_id,
                "verification": verification,
                "warnings": ["evidence link accepted but is not visible in Workpoint state yet"],
                "rollback_card": rollback_card_payload(
                    rollback_snapshot,
                    Some(workpoint_id),
                    record.project_root.as_deref(),
                    record.continuity_id.as_deref(),
                    "workpoint_evidence_link",
                    "Workpoint verification_records return to the selected safe snapshot scope"
                ),
                "retry_posture": "safe_retry",
                "resource_mode": resource_mode_status(),
                "next_tools": ["focusa_workpoint_resume", "focusa_traverse", "focusa_resource_mode"],
                "next_step_hint": "retry /v1/workpoint/resume before relying on this evidence link"
            })),
        ));
    }
    let summary_only = lowmem_caps_active();
    Ok(Json(json!({
        "status": "accepted",
        "canonical": true,
        "trust_badges": trust_badges(true, false, false, false, false, false),
        "workpoint_id": workpoint_id,
        "verification": verification,
        "workpoint": if summary_only { None } else { linked_record.as_ref().map(workpoint_packet) },
        "summary_only": summary_only,
        "resource_mode": resource_mode_status(),
        "rollback_card": rollback_card_payload(
            rollback_snapshot,
            Some(workpoint_id),
            record.project_root.as_deref(),
            record.continuity_id.as_deref(),
            "workpoint_evidence_link",
            "Workpoint verification_records return to the selected safe snapshot scope"
        ),
        "next_tools": ["focusa_workpoint_resume", "focusa_traverse"],
        "next_step_hint": "call /v1/workpoint/resume to see linked evidence in the packet"
    })))
}

async fn drift_check(
    _scope: ScopeContext,
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
    let focusa = workstream_scoped_state(state.clone(), &_scope).await;
    let record = req
        .workpoint_id
        .and_then(|id| {
            focusa
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == id)
        })
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
            _scope.clone(),
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
        .route(
            "/v1/workpoint/idempotency-cache",
            get(idempotency_cache_status),
        )
        .route("/v1/workpoint/current", get(current))
        .route("/v1/workpoint/resume", post(resume))
        .route(
            "/v1/workpoint/active-object/resolve",
            post(resolve_active_object),
        )
        .route("/v1/workpoint/evidence/link", post(link_evidence))
        .route("/v1/workpoint/drift-check", post(drift_check))
        .route(
            "/v1/workpoint/rollover/target-materialize",
            post(rollover_target_materialize),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_cache_is_eliminated_in_favor_of_scope_matched_reducer_records() {
        let payload = idempotency_cache_status_payload();
        assert_eq!(payload["status"], "eliminated");
        assert_eq!(payload["cross_scope_fallback"], false);
    }

    #[test]
    fn agent_runtime_paths_rejected_as_project_root() {
        // Pi agent
        assert_eq!(
            unsafe_project_root_reason(Some("/root/pi-mono")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/root/pi-agent")),
            Some("agent_runtime_directory")
        );
        // Node/npm agent installs
        assert_eq!(
            unsafe_project_root_reason(Some("/opt/node-v22.22.3-linux-x64")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/usr/local/bin")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/usr/local/lib/node_modules/@foo/bar")),
            Some("agent_runtime_directory")
        );
        // Claude Code
        assert_eq!(
            unsafe_project_root_reason(Some("/root/.claude")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/home/user/.claude/backups")),
            Some("agent_runtime_directory")
        );
        // OpenCode
        assert_eq!(
            unsafe_project_root_reason(Some("/root/.opencode")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/home/user/.opencode/config")),
            Some("agent_runtime_directory")
        );
        // Letta
        assert_eq!(
            unsafe_project_root_reason(Some("/root/.letta")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/home/user/.letta/sessions")),
            Some("agent_runtime_directory")
        );
        // Pi config/state
        assert_eq!(
            unsafe_project_root_reason(Some("/root/.pi")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/home/user/.pi/agent")),
            Some("agent_runtime_directory")
        );
        // Python site-packages agent installs
        assert_eq!(
            unsafe_project_root_reason(Some("/usr/local/lib/python3.12/site-packages/letta")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/usr/local/lib/python3.12/site-packages/open-code")),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some(
                "/usr/local/lib/python3.12/site-packages/pi-coding-agent"
            )),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            unsafe_project_root_reason(Some("/usr/local/lib/python3.12/site-packages/claude-code")),
            Some("agent_runtime_directory")
        );
        // Actual project paths remain valid
        assert_eq!(
            unsafe_project_root_reason(Some("/workspace/focusa-project")),
            None
        );
        assert_eq!(unsafe_project_root_reason(Some("/tmp/my-project")), None);
    }

    fn test_session_identity(
        project_root: &str,
        continuity_id: &str,
        session_id: &str,
    ) -> FocusaSessionIdentity {
        FocusaSessionIdentity {
            schema: Some("focusa.session_identity.v1".to_string()),
            project_identity: None,
            pi_session_id: Some(session_id.to_string()),
            session_frame_key: session_id.to_string(),
            session_incarnation_id: format!("{session_id}:test"),
            continuity_id: Some(continuity_id.to_string()),
            project_root: project_root.to_string(),
            canonical_parent_root: Some(project_root.to_string()),
            cwd: project_root.to_string(),
            active_worktree_root: Some(project_root.to_string()),
            working_subpath_id: Some("primary".to_string()),
            working_subpath: None,
            workspace_id: project_root.to_string(),
            process_id: Some(123),
            started_at: "2026-05-21T00:00:00Z".to_string(),
            resume_source: "manual".to_string(),
            canonical_scope: Some(true),
            scope_failure: None,
            project_root_confidence: Some("high".to_string()),
            project_root_confidence_score: Some(1.0),
            project_root_resolution_source: Some("test_fixture".to_string()),
            requires_operator_confirmation: Some(false),
            project_root_candidates: Vec::new(),
        }
    }

    #[test]
    fn session_identity_overrides_flat_checkpoint_scope() {
        let mut req = WorkpointCheckpointRequest {
            session_identity: Some(test_session_identity(
                "/repo/right",
                "cont-right",
                "session-right",
            )),
            project_root: Some("/repo/wrong".to_string()),
            continuity_id: Some("cont-wrong".to_string()),
            session_id: Some("session-wrong".to_string()),
            ..WorkpointCheckpointRequest::default()
        };
        apply_checkpoint_session_identity(&mut req);
        assert_eq!(req.project_root.as_deref(), Some("/repo/right"));
        assert_eq!(req.continuity_id.as_deref(), Some("cont-right"));
        assert_eq!(req.session_id.as_deref(), Some("session-right"));
    }

    #[test]
    fn workpoint_packet_carries_session_identity_envelope() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            session_identity: Some(test_session_identity("/repo/a", "cont-a", "session-a")),
            project_root: Some("/repo/a".to_string()),
            continuity_id: Some("cont-a".to_string()),
            session_id: Some("session-a".to_string()),
            ..WorkpointRecord::default()
        };
        let packet = workpoint_packet(&record);
        assert_eq!(
            packet
                .pointer("/session_identity/session_frame_key")
                .and_then(Value::as_str),
            Some("session-a")
        );
        assert_eq!(
            packet
                .pointer("/session_identity/session_incarnation_id")
                .and_then(Value::as_str),
            Some("session-a:test")
        );
        assert_eq!(
            packet
                .pointer("/session_identity/resume_source")
                .and_then(Value::as_str),
            Some("manual")
        );
    }

    #[test]
    fn current_ask_scope_conflict_rejects_executable_workpoint_resume() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            project_root: Some("/workspace/focusa-project".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let req = WorkpointResumeRequest {
            project_root: Some("/workspace/focusa-project".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            current_ask: Some("continue work in /home/wpuiai/uiai-engine".to_string()),
            ..WorkpointResumeRequest::default()
        };

        let rejection = current_ask_scope_rejection(&record, &req).expect("conflict rejection");
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
            rejection
                .pointer("/current_ask_scope/matches_current_ask_scope")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            rejection.pointer("/failure_class").and_then(Value::as_str),
            Some("scope_conflict")
        );
    }

    #[test]
    fn resume_packet_v2_contains_trajectory_traverse_and_provenance() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            mission: Some("High-level shared product goal".to_string()),
            next_slice: Some("Low-level next step unique to this session".to_string()),
            project_root: Some("/repo/a".to_string()),
            continuity_id: Some("cont-a".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let scope = evaluate_resume_scope(
            &record,
            Some("/repo/a"),
            Some("cont-a"),
            Some("session-a"),
            None,
        );
        let packet = workpoint_packet(&record);
        let summary = resume_summary(&record);
        let req = WorkpointResumeRequest {
            project_root: Some("/repo/a".to_string()),
            continuity_id: Some("cont-a".to_string()),
            session_id: Some("session-a".to_string()),
            mode: Some("compact_prompt".to_string()),
            ..WorkpointResumeRequest::default()
        };
        let v2 = workpoint_resume_packet_v2(
            &record,
            packet,
            &summary,
            true,
            json!({"session_changed": false}),
            json!({"percent": 100}),
            &scope,
            &req,
        );
        assert_eq!(
            v2.get("schema_version").and_then(Value::as_str),
            Some("focusa.workpoint_resume_packet.v2")
        );
        assert!(
            v2.get("rendered_summary")
                .and_then(Value::as_str)
                .unwrap()
                .contains("WORKPOINT")
        );
        assert!(v2.get("resume_summary").is_some());
        assert!(v2.get("packet_id").is_some());
        assert!(v2.get("generated_at").is_some());
        assert_eq!(v2.get("degraded").and_then(Value::as_bool), Some(false));
        assert_eq!(v2.get("confidence").and_then(Value::as_str), Some("high"));
        assert!(v2.get("project_identity").is_some());
        assert!(v2.get("session_identity").is_some());
        assert!(v2.pointer("/resume_summary/safest_next_action").is_some());
        assert!(v2.pointer("/tool_affordances/recovery").is_some());
        assert!(v2.get("trajectory").is_some());
        assert!(
            v2.get("traversal_slices")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .any(
                    |slice| slice.get("surface").and_then(Value::as_str) == Some("workpoints")
                        && slice.get("tags").and_then(Value::as_array).is_some()
                        && slice
                            .get("rehydrate_refs")
                            .and_then(Value::as_array)
                            .is_some()
                )
        );
        assert!(
            v2.get("api_provenance")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .any(
                    |entry| entry.get("route").and_then(Value::as_str) == Some("/v1/traverse")
                        && entry.get("tool_or_route").is_some()
                        && entry.get("freshness").is_some()
                )
        );
        assert_eq!(
            v2.pointer("/trajectory/hierarchy/must_not_merge_on_similarity")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn resume_packet_v2_splits_saved_scope_from_current_action_authority() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            mission: Some("Continue Focusa implementation".to_string()),
            next_slice: Some("Patch Workpoint authority labels".to_string()),
            project_root: Some("/tmp/focusa-project".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let scope = evaluate_resume_scope(
            &record,
            Some("/tmp/focusa-project"),
            Some("focusa-cont"),
            Some("session-a"),
            None,
        );
        let req = WorkpointResumeRequest {
            project_root: Some("/tmp/focusa-project".to_string()),
            continuity_id: Some("focusa-cont".to_string()),
            session_id: Some("session-a".to_string()),
            current_ask: Some(
                "wrong place; this is the PTM remote project at /home/example/plan-the-marriage"
                    .to_string(),
            ),
            ..WorkpointResumeRequest::default()
        };
        let packet = workpoint_resume_packet_v2(
            &record,
            workpoint_packet(&record),
            &resume_summary(&record),
            true,
            json!({"session_changed": false}),
            json!({"percent": 100}),
            &scope,
            &req,
        );
        assert_eq!(packet.get("canonical").and_then(Value::as_bool), Some(true));
        assert_eq!(
            packet
                .get("canonical_for_saved_scope")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            packet
                .get("matches_current_ask_scope")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            packet
                .get("action_authority_for_current_ask")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert!(
            packet
                .get("scope_conflict_reason")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("/home/example/plan-the-marriage")
        );
    }

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
    fn current_payload_status_matches_nested_workpoint_status() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            status: WorkpointStatus::Active,
            canonical: true,
            next_slice: Some("Continue from active Workpoint".to_string()),
            ..WorkpointRecord::default()
        };
        let payload = current_workpoint_payload(&record);
        assert_eq!(
            payload.get("status").and_then(Value::as_str),
            Some("active")
        );
        assert_eq!(
            payload.pointer("/workpoint/status").and_then(Value::as_str),
            Some("active")
        );
        assert_eq!(
            payload.get("operation_status").and_then(Value::as_str),
            Some("completed")
        );
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
        assert_eq!(
            packet.get("idempotency_key").and_then(Value::as_str),
            Some("idem-1")
        );
    }

    #[test]
    fn workpoint_packet_includes_project_root_for_project_folder_guard() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            session_id: Some("session-a".to_string()),
            project_root: Some("/repo/a".to_string()),
            canonical: true,
            next_slice: Some("Resume only in /repo/a".to_string()),
            ..WorkpointRecord::default()
        };
        let packet = workpoint_packet(&record);
        assert_eq!(
            packet.get("project_root").and_then(Value::as_str),
            Some("/repo/a")
        );
        assert_eq!(
            packet.get("session_id").and_then(Value::as_str),
            Some("session-a")
        );
    }

    #[test]
    fn project_root_mismatch_rejects_before_resume_injection() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            project_root: Some("/repo/focusa".to_string()),
            continuity_id: Some("cont-a".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let decision = evaluate_resume_scope(
            &record,
            Some("/repo/asapdigest"),
            Some("cont-a"),
            Some("session-b"),
            None,
        );
        assert!(!decision.canonical_scope_ok);
        let rejection = decision.rejection.expect("project mismatch rejects");
        assert_eq!(
            rejection.get("status").and_then(Value::as_str),
            Some("rejected_scope_mismatch")
        );
    }

    #[test]
    fn session_id_change_preserves_canonical_when_project_root_matches() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            continuity_id: Some("cont-a".to_string()),
            session_id: Some("pi-before-compact".to_string()),
            project_root: Some("/repo/focusa".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let decision = evaluate_resume_scope(
            &record,
            Some("/repo/focusa"),
            Some("cont-a"),
            Some("pi-after-compact"),
            None,
        );
        assert!(decision.rejection.is_none());
        assert!(decision.canonical_scope_ok);
        assert!(decision.session_changed);
        assert_eq!(
            decision.packet_session_id.as_deref(),
            Some("pi-before-compact")
        );
        assert_eq!(
            decision.expected_session_id.as_deref(),
            Some("pi-after-compact")
        );
        assert_eq!(decision.packet_continuity_id.as_deref(), Some("cont-a"));
        assert!(record.canonical && decision.canonical_scope_ok);
    }

    #[test]
    fn working_subpath_mismatch_rejects_inside_same_project_and_continuity() {
        let mut identity = test_session_identity("/repo/focusa", "cont-a", "pi-worktree-a");
        identity.working_subpath_id = Some("working-subpath:a".to_string());
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            continuity_id: Some("cont-a".to_string()),
            session_id: Some("pi-worktree-a".to_string()),
            project_root: Some("/repo/focusa".to_string()),
            session_identity: Some(identity),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let decision = evaluate_resume_scope(
            &record,
            Some("/repo/focusa"),
            Some("cont-a"),
            Some("pi-worktree-b"),
            Some("working-subpath:b"),
        );
        assert!(!decision.canonical_scope_ok);
        let rejection = decision
            .rejection
            .expect("working subpath mismatch rejects");
        assert_eq!(
            rejection.get("failure_class").and_then(Value::as_str),
            Some("working_subpath_mismatch")
        );
    }

    #[test]
    fn continuity_id_mismatch_rejects_inside_same_project_root() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            continuity_id: Some("cont-a".to_string()),
            session_id: Some("pi-before-compact".to_string()),
            project_root: Some("/repo/focusa".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let decision = evaluate_resume_scope(
            &record,
            Some("/repo/focusa"),
            Some("cont-b"),
            Some("pi-after-compact"),
            None,
        );
        assert!(!decision.canonical_scope_ok);
        let rejection = decision.rejection.expect("continuity mismatch rejects");
        assert_eq!(
            rejection.get("status").and_then(Value::as_str),
            Some("rejected_continuity_mismatch")
        );
    }

    #[test]
    fn scoped_active_workpoint_prefers_latest_matching_active_record() {
        let older_id = Uuid::now_v7();
        let newer_id = Uuid::now_v7();
        let make_record = |workpoint_id| WorkpointRecord {
            workpoint_id,
            project_root: Some("/repo/focusa".to_string()),
            continuity_id: Some("cont-a".to_string()),
            status: WorkpointStatus::Active,
            canonical: true,
            ..WorkpointRecord::default()
        };
        let state = focusa_core::types::FocusaState {
            workpoint: focusa_core::types::WorkpointState {
                records: vec![make_record(older_id), make_record(newer_id)],
                ..focusa_core::types::WorkpointState::default()
            },
            ..focusa_core::types::FocusaState::default()
        };

        let selected = active_workpoint_for_scope(&state, Some("/repo/focusa"), Some("cont-a"))
            .expect("latest scoped active workpoint");
        assert_eq!(selected.workpoint_id, newer_id);
    }

    #[test]
    fn active_workpoint_ignores_unsafe_broad_root_record() {
        let unsafe_id = Uuid::now_v7();
        let state = focusa_core::types::FocusaState {
            workpoint: focusa_core::types::WorkpointState {
                active_workpoint_id: Some(unsafe_id),
                records: vec![WorkpointRecord {
                    workpoint_id: unsafe_id,
                    project_root: Some("/root".to_string()),
                    continuity_id: Some("cont-root".to_string()),
                    status: WorkpointStatus::Active,
                    canonical: true,
                    ..WorkpointRecord::default()
                }],
                ..focusa_core::types::WorkpointState::default()
            },
            ..focusa_core::types::FocusaState::default()
        };
        assert!(active_workpoint(&state).is_none());
    }

    #[test]
    fn broad_project_root_rejects_before_resume_injection() {
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            project_root: Some("/root".to_string()),
            continuity_id: Some("cont-a".to_string()),
            canonical: true,
            ..WorkpointRecord::default()
        };
        let decision = evaluate_resume_scope(
            &record,
            Some("/root"),
            Some("cont-a"),
            Some("pi-after"),
            None,
        );
        assert!(!decision.canonical_scope_ok);
        let rejection = decision.rejection.expect("unsafe root rejects");
        assert_eq!(
            rejection.get("status").and_then(Value::as_str),
            Some("rejected_unsafe_project_root")
        );
        assert_eq!(
            rejection.get("failure_class").and_then(Value::as_str),
            Some("scope_mismatch")
        );
    }

    #[test]
    fn canonical_checkpoint_requires_safe_project_identity_envelope() {
        let err =
            unsafe_checkpoint_rejection("unsafe_broad_project_root", "project_root", Some("/root"));
        assert_eq!(err.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            err.1.0.get("status").and_then(Value::as_str),
            Some("validation_rejected")
        );
        assert_eq!(
            err.1.0.get("field").and_then(Value::as_str),
            Some("project_root")
        );
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
            next_slice: Some(
                "Patch the widget binding\nDO_NOT_DRIFT: notes-only/generic validation".to_string(),
            ),
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
        assert!(
            decision
                .drift_classes
                .contains(&"notes_only_drift".to_string())
        );
        assert!(
            decision
                .drift_classes
                .contains(&"wrong_object_drift".to_string())
        );
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
        assert_eq!(
            err.1.0.get("status").and_then(Value::as_str),
            Some("validation_rejected")
        );
        assert_eq!(
            err.1.0.get("field").and_then(Value::as_str),
            Some("checkpoint_reason")
        );
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
            next_slice: Some(
                "Complete stress suite\nDO_NOT_DRIFT: Do not demote existing tools.".to_string(),
            ),
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

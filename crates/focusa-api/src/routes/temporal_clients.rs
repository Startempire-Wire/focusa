//! Canonical Spec 137 client-facing route families.
//! Projections read the signed temporal ledger; mutations append signed events.
use super::temporal::{
    append_signed_events, fail, ledger, project_active_focus_frame, read_events,
};
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::temporal::{
    TemporalClaim, TemporalClaimKind, TemporalClaimStatus, TemporalConfidence, TemporalEvent,
    TemporalEventKind, TemporalScope, project_temporal, validate_claim,
};
use focusa_core::temporal_deadline::{CivilTimeIntent, resolve_civil_time};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;
#[derive(Debug, Deserialize, Clone)]
struct ScopeQuery {
    project_root: String,
    continuity_id: String,
    #[serde(default)]
    host_id: Option<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
    #[serde(default)]
    item_id: Option<String>,
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    subject_ref: Option<String>,
}
impl ScopeQuery {
    fn scope(&self) -> TemporalScope {
        TemporalScope {
            project_root: self.project_root.clone(),
            continuity_id: self.continuity_id.clone(),
            host_id: self.host_id.clone(),
            operator_id: None,
            workpoint_id: self.workpoint_id.clone(),
            item_id: self.item_id.clone(),
            task_id: self.task_id.clone(),
        }
    }
}
#[derive(Debug, Deserialize)]
struct DeadlineMutation {
    project_root: String,
    continuity_id: String,
    subject_ref: Option<String>,
    deadline_id: Option<String>,
    deadline_at: Option<String>,
    local_time: Option<String>,
    timezone: Option<String>,
    fold_policy: Option<String>,
    gap_policy: Option<String>,
    calendar_ref: Option<String>,
    tzdb_version: Option<String>,
    calendar_version: Option<String>,
    readiness_target: Option<String>,
    completion_target_ref: Option<String>,
    expected_revision: Option<u64>,
    reason: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    idempotency_key: String,
    #[serde(default)]
    confirm: bool,
}
#[derive(Debug, Deserialize)]
struct EstimateMutation {
    project_root: String,
    continuity_id: String,
    estimate_id: Option<String>,
    subject_ref: Option<String>,
    target_state: Option<String>,
    duration_ms: Option<u64>,
    actual_event_ref: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    idempotency_key: Option<String>,
}
#[derive(Debug, Deserialize)]
struct ProgressMutation {
    project_root: String,
    continuity_id: String,
    item_id: String,
    kind: String,
    evidence_refs: Vec<String>,
    idempotency_key: String,
}
#[derive(Debug, Deserialize)]
struct CivilReresolveRequest {
    project_root: String,
    continuity_id: String,
    deadline_id: String,
    tzdb_version: String,
}
fn event(
    scope: TemporalScope,
    kind: TemporalEventKind,
    claim: Option<TemporalClaim>,
    metadata: BTreeMap<String, Value>,
    key: &str,
) -> TemporalEvent {
    TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: kind,
        scope,
        claim,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: key.into(),
        digest: String::new(),
    }
}
fn read(scope: TemporalScope) -> Result<Vec<TemporalEvent>, (StatusCode, Json<Value>)> {
    read_events(&ledger(scope)?)
}
fn matching_claim(events: &[TemporalEvent], id: &str) -> Option<TemporalClaim> {
    events
        .iter()
        .rev()
        .filter_map(|e| e.claim.as_ref())
        .find(|c| c.claim_id == id)
        .cloned()
}
fn completed(schema: &str, key: &str, value: Value) -> Json<Value> {
    Json(json!({"schema":schema,"status":"completed","canonical":true,key:value}))
}
async fn time_now() -> Json<Value> {
    let now = Utc::now();
    Json(
        json!({"schema":"focusa.time_now.v1","status":"completed","canonical":true,"wall_utc":now,"source":"host_system_clock","clock_domain":"wall_utc","confidence":"medium","uncertainty":"host clock trust requires /v1/time/trust evidence"}),
    )
}
async fn time_doctor() -> Json<Value> {
    Json(
        json!({"schema":"focusa.time_doctor.v1","status":"completed","canonical":true,"wall_clock_available":true,"captured_at":Utc::now(),"trust_status":"requires_exact_scope_samples","next_action":"inspect signed clock trust for the target scope"}),
    )
}
async fn time_status(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = q.scope();
    let events = read(scope.clone())?;
    Ok(completed(
        "focusa.time_status.v1",
        "projection",
        json!(project_temporal(scope, &events, Utc::now())),
    ))
}
async fn time_trust() -> Json<Value> {
    let clocks = focusa_core::temporal_platform::capture_platform_clocks();
    Json(json!({
        "schema":"focusa.time_trust.v1","status":"completed","canonical":false,
        "trust_status":"host_clock_observed_not_attested","captured_at":clocks.realtime_utc,
        "capabilities":clocks.capabilities,
        "next_action":"use /v1/temporal/clock/capture for a signed project-scoped sample"
    }))
}
async fn time_samples() -> Json<Value> {
    let sample = focusa_core::temporal_platform::capture_temporal_clock_sample("UTC", None);
    Json(
        json!({"schema":"focusa.time_samples.v1","status":"completed","canonical":false,"samples":[sample]}),
    )
}
async fn time_capabilities() -> Json<Value> {
    let clocks = focusa_core::temporal_platform::capture_platform_clocks();
    Json(
        json!({"schema":"focusa.time_capabilities.v1","status":"completed","canonical":true,"clock_capabilities":clocks.capabilities}),
    )
}
async fn deadlines(Query(q): Query<ScopeQuery>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let claims = read(q.scope())?
        .into_iter()
        .filter_map(|e| e.claim)
        .filter(|c| c.kind == TemporalClaimKind::ExternalCommitment)
        .collect::<Vec<_>>();
    Ok(completed(
        "focusa.deadline_list.v1",
        "deadlines",
        json!(claims),
    ))
}
async fn deadline(
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let claim = matching_claim(&read(q.scope())?, &id).ok_or_else(|| {
        fail(
            StatusCode::NOT_FOUND,
            "deadline_not_found",
            "no deadline with this id exists in the exact scope",
        )
    })?;
    if claim.kind != TemporalClaimKind::ExternalCommitment {
        return Err(fail(
            StatusCode::CONFLICT,
            "claim_is_not_deadline",
            "the requested claim is not an external commitment",
        ));
    }
    Ok(completed("focusa.deadline.v1", "deadline", json!(claim)))
}
async fn deadline_conflicts(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = q.scope();
    let events = read(scope.clone())?;
    let projection = project_temporal(scope, &events, Utc::now());
    Ok(completed(
        "focusa.deadline_conflicts.v1",
        "conflict",
        json!({"state":projection.deadline_conflict_state,"approaching_deadlines":projection.approaching_deadlines}),
    ))
}
async fn deadline_set(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeadlineMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "deadline set requires confirm=true",
        ));
    }
    if req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_evidence_required",
            "deadline authority requires evidence",
        ));
    }
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let target = req
        .deadline_at
        .as_deref()
        .ok_or_else(|| {
            fail(
                StatusCode::BAD_REQUEST,
                "deadline_at_required",
                "deadline_at must be RFC3339",
            )
        })?
        .parse()
        .map_err(|_| {
            fail(
                StatusCode::BAD_REQUEST,
                "invalid_deadline_at",
                "deadline_at must be RFC3339",
            )
        })?;
    let now = Utc::now();
    let claim = TemporalClaim {
        claim_id: req
            .deadline_id
            .unwrap_or_else(|| format!("deadline:{}", Uuid::now_v7())),
        revision: 1,
        scope: scope.clone(),
        kind: TemporalClaimKind::ExternalCommitment,
        status: TemporalClaimStatus::Canonical,
        subject_ref: req.subject_ref.ok_or_else(|| {
            fail(
                StatusCode::BAD_REQUEST,
                "subject_required",
                "subject_ref is required",
            )
        })?,
        target_at: Some(target),
        duration_ms: None,
        timezone: req.timezone.unwrap_or_else(|| "UTC".into()),
        source: "operator_confirmed_cli".into(),
        source_ref: req.completion_target_ref,
        operator_confirmed: true,
        confidence: TemporalConfidence::Verified,
        uncertainty: None,
        observed_at: now,
        effective_at: now,
        expires_at: None,
        supersedes_revision: None,
        evidence_refs: req.evidence_refs,
        reason_code: req.reason,
    };
    validate_claim(&claim, None).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_validation_failed",
            format!("{e:?}"),
        )
    })?;
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::ClaimCommitted,
            Some(claim),
            BTreeMap::new(),
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger(scope.clone())?).await?;
    Ok(completed(
        "focusa.deadline_mutation_result.v1",
        "events",
        json!(appended),
    ))
}
async fn deadline_set_civil(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeadlineMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "civil deadline set requires confirm=true",
        ));
    }
    if req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_evidence_required",
            "civil deadline authority requires evidence",
        ));
    }
    let local_text = req.local_time.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "local_time_required",
            "local_time is required",
        )
    })?;
    let local =
        chrono::NaiveDateTime::parse_from_str(local_text, "%Y-%m-%dT%H:%M:%S").map_err(|_| {
            fail(
                StatusCode::BAD_REQUEST,
                "invalid_civil_datetime",
                "local_time must use YYYY-MM-DDTHH:MM:SS",
            )
        })?;
    let timezone = req.timezone.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "timezone_required",
            "IANA timezone is required",
        )
    })?;
    let intent = CivilTimeIntent {
        intent_id: format!("civil:{}", req.idempotency_key),
        original_expression: local_text.to_string(),
        timezone: timezone.clone(),
        tzdb_version: req
            .tzdb_version
            .clone()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                fail(
                    StatusCode::PRECONDITION_FAILED,
                    "tzdb_version_required",
                    "tzdb_version is required",
                )
            })?,
        calendar: req
            .calendar_ref
            .clone()
            .unwrap_or_else(|| "gregorian".into()),
        calendar_version: req
            .calendar_version
            .clone()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                fail(
                    StatusCode::PRECONDITION_FAILED,
                    "calendar_version_required",
                    "calendar_version is required",
                )
            })?,
        jurisdiction: None,
        jurisdiction_rule_version: None,
        fold_policy: req.fold_policy.clone().unwrap_or_else(|| "reject".into()),
        gap_policy: req.gap_policy.clone().unwrap_or_else(|| "reject".into()),
        recurrence_rule: None,
        floating: false,
        resolved_instants: Vec::new(),
        resolution_receipt_refs: Vec::new(),
        supersedes_resolution_ref: None,
    };
    let resolved = resolve_civil_time(&intent, local).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "civil_time_resolution_failed",
            format!("{e:?}"),
        )
    })?;
    if resolved.len() != 1 {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "single_deadline_instant_required",
            "fold policy must resolve one deadline instant",
        ));
    }
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let now = Utc::now();
    let claim = TemporalClaim {
        claim_id: req
            .deadline_id
            .unwrap_or_else(|| format!("deadline:{}", Uuid::now_v7())),
        revision: 1,
        scope: scope.clone(),
        kind: TemporalClaimKind::ExternalCommitment,
        status: TemporalClaimStatus::Canonical,
        subject_ref: req.subject_ref.ok_or_else(|| {
            fail(
                StatusCode::BAD_REQUEST,
                "subject_required",
                "subject_ref is required",
            )
        })?,
        target_at: Some(resolved[0]),
        duration_ms: None,
        timezone,
        source: "operator_confirmed_civil_cli".into(),
        source_ref: req.completion_target_ref,
        operator_confirmed: true,
        confidence: TemporalConfidence::Verified,
        uncertainty: None,
        observed_at: now,
        effective_at: now,
        expires_at: None,
        supersedes_revision: None,
        evidence_refs: req.evidence_refs.clone(),
        reason_code: req.reason,
    };
    validate_claim(&claim, None).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_validation_failed",
            format!("{e:?}"),
        )
    })?;
    let metadata = BTreeMap::from([
        ("civil_time_intent".into(), json!(intent)),
        ("resolved_instants".into(), json!(resolved)),
    ]);
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::ClaimCommitted,
            Some(claim),
            metadata,
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger(scope.clone())?).await?;
    Ok(completed(
        "focusa.deadline_civil_mutation_result.v1",
        "events",
        json!(appended),
    ))
}
async fn deadline_resolve_civil(
    Json(req): Json<CivilReresolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let events = read(scope)?;
    let source = events
        .iter()
        .rev()
        .find(|event| {
            event
                .claim
                .as_ref()
                .is_some_and(|claim| claim.claim_id == req.deadline_id)
                && event.metadata.contains_key("civil_time_intent")
        })
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "civil_deadline_not_found",
                "deadline has no recorded civil-time intent in this exact scope",
            )
        })?;
    let mut intent: CivilTimeIntent =
        serde_json::from_value(source.metadata["civil_time_intent"].clone()).map_err(|e| {
            fail(
                StatusCode::CONFLICT,
                "civil_time_intent_invalid",
                e.to_string(),
            )
        })?;
    if req.tzdb_version.trim().is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "tzdb_version_required",
            "tzdb_version is required",
        ));
    }
    intent.tzdb_version = req.tzdb_version;
    intent.resolved_instants.clear();
    intent.resolution_receipt_refs.clear();
    let local =
        chrono::NaiveDateTime::parse_from_str(&intent.original_expression, "%Y-%m-%dT%H:%M:%S")
            .map_err(|_| {
                fail(
                    StatusCode::CONFLICT,
                    "civil_time_expression_invalid",
                    "stored civil expression is invalid",
                )
            })?;
    let resolved = resolve_civil_time(&intent, local).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "civil_time_resolution_failed",
            format!("{e:?}"),
        )
    })?;
    Ok(Json(json!({
        "schema":"focusa.deadline_civil_resolution.v1","status":"completed","canonical":false,
        "deadline_id":req.deadline_id,"intent":intent,"resolved_instants":resolved,
        "next_action":"revise the deadline with explicit confirmation if the resolved instant changed"
    })))
}
async fn deadline_revise(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeadlineMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "deadline revision requires confirm=true",
        ));
    }
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let events = read(scope.clone())?;
    let id = req.deadline_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "deadline_id_required",
            "deadline_id is required",
        )
    })?;
    let mut prior = matching_claim(&events, id).ok_or_else(|| {
        fail(
            StatusCode::NOT_FOUND,
            "deadline_not_found",
            "deadline does not exist",
        )
    })?;
    let expected = req.expected_revision.ok_or_else(|| {
        fail(
            StatusCode::PRECONDITION_REQUIRED,
            "expected_revision_required",
            "expected_revision is required",
        )
    })?;
    if prior.revision != expected {
        return Err(fail(
            StatusCode::CONFLICT,
            "revision_conflict",
            "deadline revision changed",
        ));
    }
    prior.revision += 1;
    prior.supersedes_revision = Some(expected);
    prior.reason_code = req.reason;
    prior.observed_at = Utc::now();
    if let Some(at) = req.deadline_at {
        prior.target_at = Some(at.parse().map_err(|_| {
            fail(
                StatusCode::BAD_REQUEST,
                "invalid_deadline_at",
                "deadline_at must be RFC3339",
            )
        })?);
    }
    validate_claim(&prior, matching_claim(&events, id).as_ref()).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_validation_failed",
            format!("{e:?}"),
        )
    })?;
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::ClaimRevised,
            Some(prior),
            BTreeMap::new(),
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger(scope.clone())?).await?;
    Ok(completed(
        "focusa.deadline_mutation_result.v1",
        "events",
        json!(appended),
    ))
}
async fn deadline_clear(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeadlineMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "deadline clear requires confirm=true",
        ));
    }
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let events = read(scope.clone())?;
    let id = req.deadline_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "deadline_id_required",
            "deadline_id is required",
        )
    })?;
    let mut prior = matching_claim(&events, id).ok_or_else(|| {
        fail(
            StatusCode::NOT_FOUND,
            "deadline_not_found",
            "deadline does not exist",
        )
    })?;
    let expected = req.expected_revision.ok_or_else(|| {
        fail(
            StatusCode::PRECONDITION_REQUIRED,
            "expected_revision_required",
            "expected_revision is required",
        )
    })?;
    if prior.revision != expected {
        return Err(fail(
            StatusCode::CONFLICT,
            "revision_conflict",
            "deadline revision changed",
        ));
    }
    prior.revision += 1;
    prior.supersedes_revision = Some(expected);
    prior.status = TemporalClaimStatus::Retracted;
    prior.reason_code = req.reason;
    prior.observed_at = Utc::now();
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::ClaimSuperseded,
            Some(prior),
            BTreeMap::new(),
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger(scope.clone())?).await?;
    Ok(completed(
        "focusa.deadline_mutation_result.v1",
        "events",
        json!(appended),
    ))
}
async fn estimate_request(
    Json(req): Json<EstimateMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let duration = req.duration_ms.ok_or_else(|| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "estimate_range_required",
            "duration_ms is required",
        )
    })?;
    if req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "estimate_evidence_required",
            "an estimate requires evidence",
        ));
    }
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let now = Utc::now();
    let claim = TemporalClaim {
        claim_id: req
            .estimate_id
            .unwrap_or_else(|| format!("estimate:{}", Uuid::now_v7())),
        revision: 1,
        scope: scope.clone(),
        kind: TemporalClaimKind::Estimate,
        status: TemporalClaimStatus::Canonical,
        subject_ref: req.subject_ref.ok_or_else(|| {
            fail(
                StatusCode::BAD_REQUEST,
                "subject_required",
                "subject_ref is required",
            )
        })?,
        target_at: None,
        duration_ms: Some(duration),
        timezone: "UTC".into(),
        source: "grounded_estimate_request".into(),
        source_ref: req.target_state,
        operator_confirmed: false,
        confidence: TemporalConfidence::Medium,
        uncertainty: None,
        observed_at: now,
        effective_at: now,
        expires_at: None,
        supersedes_revision: None,
        evidence_refs: req.evidence_refs,
        reason_code: None,
    };
    validate_claim(&claim, None).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "estimate_validation_failed",
            format!("{e:?}"),
        )
    })?;
    let key = req
        .idempotency_key
        .unwrap_or_else(|| format!("estimate:{}", Uuid::now_v7()));
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &key,
        vec![event(
            scope,
            TemporalEventKind::ClaimCommitted,
            Some(claim),
            BTreeMap::new(),
            &key,
        )],
    )?;
    Ok(completed(
        "focusa.estimate_request_result.v1",
        "events",
        json!(appended),
    ))
}
async fn estimate_get(
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let claim = matching_claim(&read(q.scope())?, &id).ok_or_else(|| {
        fail(
            StatusCode::NOT_FOUND,
            "estimate_not_found",
            "estimate does not exist",
        )
    })?;
    if !matches!(
        claim.kind,
        TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
    ) {
        return Err(fail(
            StatusCode::CONFLICT,
            "claim_is_not_estimate",
            "claim is not an estimate",
        ));
    }
    Ok(completed("focusa.estimate.v1", "estimate", json!(claim)))
}
async fn estimate_validate(
    Json(req): Json<EstimateMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let id = req.estimate_id.ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "estimate_id_required",
            "estimate_id is required",
        )
    })?;
    let claim = matching_claim(
        &read(TemporalScope::project(req.project_root, req.continuity_id))?,
        &id,
    )
    .ok_or_else(|| {
        fail(
            StatusCode::NOT_FOUND,
            "estimate_not_found",
            "estimate does not exist",
        )
    })?;
    validate_claim(&claim, None).map_err(|e| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "estimate_validation_failed",
            format!("{e:?}"),
        )
    })?;
    Ok(completed(
        "focusa.estimate_validation_result.v1",
        "estimate",
        json!(claim),
    ))
}
async fn estimate_evaluate(
    Json(req): Json<EstimateMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = TemporalScope::project(req.project_root, req.continuity_id);
    let id = req.estimate_id.ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "estimate_id_required",
            "estimate_id is required",
        )
    })?;
    if matching_claim(&read(scope.clone())?, &id).is_none() {
        return Err(fail(
            StatusCode::NOT_FOUND,
            "estimate_not_found",
            "estimate does not exist",
        ));
    }
    let actual = req.actual_event_ref.ok_or_else(|| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "actual_event_required",
            "actual_event_ref is required",
        )
    })?;
    let key = req
        .idempotency_key
        .unwrap_or_else(|| format!("estimate-eval:{}", Uuid::now_v7()));
    let metadata = BTreeMap::from([
        ("estimate_id".into(), json!(id)),
        ("actual_event_ref".into(), json!(actual)),
        ("evidence_refs".into(), json!(req.evidence_refs)),
    ]);
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &key,
        vec![event(
            scope,
            TemporalEventKind::ForecastEvaluated,
            None,
            metadata,
            &key,
        )],
    )?;
    Ok(completed(
        "focusa.estimate_evaluation_result.v1",
        "events",
        json!(appended),
    ))
}
async fn estimate_history(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let claims = read(q.scope())?
        .into_iter()
        .filter_map(|e| e.claim)
        .filter(|c| {
            matches!(
                c.kind,
                TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
            )
        })
        .collect::<Vec<_>>();
    Ok(completed(
        "focusa.estimate_history.v1",
        "estimates",
        json!(claims),
    ))
}
async fn progress_record(
    Json(req): Json<ProgressMutation>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "progress_evidence_required",
            "material progress requires evidence",
        ));
    }
    let scope = TemporalScope {
        item_id: Some(req.item_id.clone()),
        ..TemporalScope::project(req.project_root, req.continuity_id)
    };
    let metadata = BTreeMap::from([
        ("item_id".into(), json!(req.item_id)),
        ("kind".into(), json!(req.kind)),
        ("evidence_refs".into(), json!(req.evidence_refs)),
    ]);
    let appended = append_signed_events(
        &ledger(scope.clone())?,
        &req.idempotency_key,
        vec![event(
            scope,
            TemporalEventKind::ProgressObserved,
            None,
            metadata,
            &req.idempotency_key,
        )],
    )?;
    Ok(completed(
        "focusa.progress_record_result.v1",
        "events",
        json!(appended),
    ))
}
async fn progress_status(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let events = read(q.scope())?
        .into_iter()
        .filter(|e| e.event_kind == TemporalEventKind::ProgressObserved)
        .collect::<Vec<_>>();
    Ok(completed(
        "focusa.progress_status.v1",
        "progress_events",
        json!(events),
    ))
}
async fn incident_list(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let events = read(q.scope())?
        .into_iter()
        .filter(|e| e.event_kind == TemporalEventKind::LostTimeIncidentRecorded)
        .filter(|e| {
            q.subject_ref
                .as_ref()
                .is_none_or(|s| e.metadata.get("subject_ref").and_then(Value::as_str) == Some(s))
        })
        .collect::<Vec<_>>();
    Ok(completed(
        "focusa.lost_time_incidents.v1",
        "incidents",
        json!(events),
    ))
}
async fn incident(
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let found = read(q.scope())?
        .into_iter()
        .find(|e| {
            e.event_kind == TemporalEventKind::LostTimeIncidentRecorded
                && (e.event_id == id
                    || e.metadata.get("incident_id").and_then(Value::as_str) == Some(&id))
        })
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "lost_time_incident_not_found",
                "incident does not exist in the exact scope",
            )
        })?;
    Ok(completed(
        "focusa.lost_time_incident.v1",
        "incident",
        json!(found),
    ))
}
async fn no_progress(
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let progress = read(q.scope())?
        .into_iter()
        .filter(|e| e.event_kind == TemporalEventKind::ProgressObserved)
        .max_by_key(|e| e.recorded_at);
    let age_ms = progress.as_ref().map(|e| {
        Utc::now()
            .signed_duration_since(e.recorded_at)
            .num_milliseconds()
            .max(0)
    });
    Ok(completed(
        "focusa.no_progress_incidents.v1",
        "assessment",
        json!({"last_material_progress":progress,"no_progress_age_ms":age_ms,"state":if age_ms.is_some(){"observed"}else{"unknown"}}),
    ))
}
async fn opportunity(
    Path(subject): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let incidents = read(q.scope())?
        .into_iter()
        .filter(|e| {
            e.event_kind == TemporalEventKind::LostTimeIncidentRecorded
                && e.metadata.get("subject_ref").and_then(Value::as_str) == Some(&subject)
        })
        .collect::<Vec<_>>();
    let posture = if incidents
        .iter()
        .any(|e| e.metadata.get("posture").and_then(Value::as_str) == Some("evidence_proven_miss"))
    {
        "evidence_proven_miss"
    } else if incidents.is_empty() {
        "unknown_counterfactual"
    } else {
        "risk"
    };
    Ok(completed(
        "focusa.opportunity_posture.v1",
        "opportunity",
        json!({"subject_ref":subject,"posture":posture,"incident_refs":incidents.iter().map(|e|&e.event_id).collect::<Vec<_>>() } ),
    ))
}
async fn cancellation(
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let found = read(q.scope())?
        .into_iter()
        .find(|e| {
            matches!(
                e.event_kind,
                TemporalEventKind::CancellationRequested
                    | TemporalEventKind::CancellationAcknowledged
            ) && (e.event_id == id
                || e.metadata.values().any(|value| {
                    value.as_str() == Some(&id)
                        || value.get("cancellation_id").and_then(Value::as_str) == Some(&id)
                }))
        })
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "cancellation_not_found",
                "cancellation does not exist in the exact scope",
            )
        })?;
    Ok(completed(
        "focusa.cancellation.v1",
        "cancellation",
        json!(found),
    ))
}
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/time/now", get(time_now))
        .route("/v1/time/doctor", get(time_doctor))
        .route("/v1/time/trust", get(time_trust))
        .route("/v1/time/samples", get(time_samples))
        .route("/v1/time/capabilities", get(time_capabilities))
        .route("/v1/deadline/set", post(deadline_set))
        .route("/v1/deadline/set-civil", post(deadline_set_civil))
        .route("/v1/deadline/resolve-civil", post(deadline_resolve_civil))
        .route("/v1/deadline/revise", post(deadline_revise))
        .route("/v1/deadline/clear", post(deadline_clear))
        .route("/v1/estimate/request", post(estimate_request))
        .route("/v1/estimate/validate", post(estimate_validate))
        .route("/v1/estimate/evaluate", post(estimate_evaluate))
        .route("/v1/progress/record", post(progress_record))
        .route("/v1/lost-time/incidents/{id}", get(incident))
        .route("/v1/opportunities/{subject}", get(opportunity))
        .route("/v1/cancellation/{id}", get(cancellation))
}

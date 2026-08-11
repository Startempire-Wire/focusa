//! Canonical Spec 137 mutation and validation surfaces.

use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use focusa_core::{
    temporal::{
        TemporalClaim, TemporalClaimKind, TemporalClaimStatus, TemporalEvent, TemporalEventKind,
        validate_claim,
    },
    temporal_operations::{
        HumanCalendarContext, TemporalExecutionGuard, TemporalPriorityFrame,
        authorize_temporal_action,
    },
    temporal_progress::{ProgressSignal, is_material_progress},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

use super::temporal::{
    ApiFailure, TemporalScopeDimensions, append_signed_events, fail, ledger,
    project_active_focus_frame, read_events, scope,
};

#[derive(Debug, Deserialize)]
pub(super) struct CanonicalMutationRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) idempotency_key: String,
    #[serde(default)]
    pub(super) confirm: bool,
    #[serde(default)]
    pub(super) evidence_refs: Vec<String>,
    #[serde(default)]
    pub(super) claim: Option<TemporalClaim>,
    #[serde(default)]
    pub(super) guard: Option<TemporalExecutionGuard>,
    #[serde(default)]
    pub(super) progress_signal: Option<ProgressSignal>,
    #[serde(default)]
    pub(super) entity_id: Option<String>,
    #[serde(default)]
    pub(super) expected_revision: Option<u64>,
    #[serde(default)]
    pub(super) reason_code: Option<String>,
    #[serde(default)]
    pub(super) metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CanonicalValidationRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    #[serde(default)]
    pub(super) evidence_refs: Vec<String>,
    #[serde(default)]
    pub(super) claim: Option<TemporalClaim>,
    #[serde(default)]
    pub(super) guard: Option<TemporalExecutionGuard>,
    #[serde(default)]
    pub(super) human_calendar_context: Option<HumanCalendarContext>,
    #[serde(default)]
    pub(super) temporal_priority_frame: Option<TemporalPriorityFrame>,
    #[serde(default)]
    pub(super) operator_ask_digest: Option<String>,
    #[serde(default)]
    pub(super) authorized_action_ref: Option<String>,
    #[serde(default)]
    pub(super) claims: Vec<TemporalClaim>,
}

pub(super) fn exact_scope(
    req: &CanonicalMutationRequest,
) -> Result<focusa_core::temporal::TemporalScope, ApiFailure> {
    let exact = scope(
        req.project_root.clone(),
        req.continuity_id.clone(),
        req.dimensions.clone(),
    );
    ledger(exact.clone())?;
    if req.idempotency_key.trim().is_empty() || req.idempotency_key.len() > 256 {
        return Err(fail(
            StatusCode::BAD_REQUEST,
            "invalid_idempotency_key",
            "a stable idempotency_key of at most 256 characters is required",
        ));
    }
    if req.evidence_refs.is_empty()
        || req
            .evidence_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "evidence_required",
            "at least one non-empty evidence ref is required",
        ));
    }
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "canonical temporal mutation requires confirm=true",
        ));
    }
    Ok(exact)
}

pub(super) fn validate_claim_scope(
    claim: &TemporalClaim,
    exact: &focusa_core::temporal::TemporalScope,
) -> Result<(), ApiFailure> {
    if claim.scope != *exact {
        return Err(fail(
            StatusCode::CONFLICT,
            "scope_mismatch",
            "temporal object scope must exactly match request scope",
        ));
    }
    validate_claim(claim, None).map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "claim_validation_failed",
            format!("{error:?}"),
        )
    })
}

pub(super) fn metadata_event(
    kind: TemporalEventKind,
    exact: focusa_core::temporal::TemporalScope,
    key: &str,
    entity_type: &str,
    entity: Value,
    evidence: &[String],
    reason: Option<&String>,
) -> TemporalEvent {
    let mut metadata = BTreeMap::new();
    metadata.insert(entity_type.into(), entity);
    metadata.insert("evidence_refs".into(), json!(evidence));
    if let Some(reason) = reason {
        metadata.insert("reason_code".into(), json!(reason));
    }
    TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: kind,
        scope: exact,
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: key.into(),
        digest: String::new(),
    }
}

pub(super) async fn persist(
    state: &AppState,
    exact: focusa_core::temporal::TemporalScope,
    key: &str,
    event: TemporalEvent,
    schema: &str,
) -> Result<Json<Value>, ApiFailure> {
    persist_many(state, exact, key, vec![event], schema).await
}

pub(super) async fn persist_many(
    state: &AppState,
    exact: focusa_core::temporal::TemporalScope,
    key: &str,
    pending: Vec<TemporalEvent>,
    schema: &str,
) -> Result<Json<Value>, ApiFailure> {
    let log = ledger(exact.clone())?;
    let events = append_signed_events(&log, key, pending)?;
    project_active_focus_frame(state, &exact, &log).await?;
    let projection =
        focusa_core::temporal::project_temporal(exact, &read_events(&log)?, Utc::now());
    Ok(Json(
        json!({"schema":schema,"status":"completed","canonical":true,"events":events,
        "projection":projection,"receipt_ref":format!("temporal:{key}")}),
    ))
}

pub(super) async fn deadline_set(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let claim = req.claim.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "deadline_claim_required",
            "claim is required",
        )
    })?;
    if !matches!(
        claim.kind,
        TemporalClaimKind::ExternalCommitment | TemporalClaimKind::InternalReadinessTarget
    ) {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "deadline_kind_invalid",
            "deadline must be an external commitment or internal readiness target",
        ));
    }
    validate_claim_scope(&claim, &exact)?;
    let event = TemporalEvent {
        claim: Some(claim),
        ..metadata_event(
            TemporalEventKind::ClaimCommitted,
            exact.clone(),
            &req.idempotency_key,
            "authority_evidence",
            json!(req.evidence_refs),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.deadline_set.v1",
    )
    .await
}

pub(super) async fn deadline_revise(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let claim = req.claim.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "deadline_claim_required",
            "claim is required",
        )
    })?;
    let log = ledger(exact.clone())?;
    let previous = read_events(&log)?
        .into_iter()
        .rev()
        .filter_map(|event| event.claim)
        .find(|prior| prior.claim_id == claim.claim_id)
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "deadline_not_found",
                "cannot revise an unknown deadline",
            )
        })?;
    validate_claim(&claim, Some(&previous)).map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "revision_rejected",
            format!("{error:?}"),
        )
    })?;
    if claim.scope != exact {
        return Err(fail(
            StatusCode::CONFLICT,
            "scope_mismatch",
            "deadline scope mismatch",
        ));
    }
    let mut superseded = previous;
    superseded.status = TemporalClaimStatus::Superseded;
    let superseded_event = TemporalEvent {
        claim: Some(superseded),
        ..metadata_event(
            TemporalEventKind::ClaimSuperseded,
            exact.clone(),
            &req.idempotency_key,
            "authority_evidence",
            json!(req.evidence_refs),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    let revised_event = TemporalEvent {
        claim: Some(claim),
        ..metadata_event(
            TemporalEventKind::ClaimRevised,
            exact.clone(),
            &req.idempotency_key,
            "authority_evidence",
            json!(req.evidence_refs),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    persist_many(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        vec![superseded_event, revised_event],
        "focusa.deadline_revision.v1",
    )
    .await
}

pub(super) async fn deadline_clear(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let id = req.entity_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "deadline_id_required",
            "entity_id is required",
        )
    })?;
    let log = ledger(exact.clone())?;
    let mut claim = read_events(&log)?
        .into_iter()
        .rev()
        .filter_map(|event| event.claim)
        .find(|claim| claim.claim_id == id)
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "deadline_not_found",
                "cannot clear an unknown deadline",
            )
        })?;
    if req.expected_revision != Some(claim.revision) {
        return Err(fail(
            StatusCode::CONFLICT,
            "revision_conflict",
            "expected_revision must match the current deadline revision",
        ));
    }
    let mut superseded = claim.clone();
    superseded.status = TemporalClaimStatus::Superseded;
    claim.revision += 1;
    claim.supersedes_revision = Some(claim.revision - 1);
    claim.status = TemporalClaimStatus::Retracted;
    claim.effective_at = Utc::now();
    claim.evidence_refs.extend(req.evidence_refs.clone());
    let superseded_event = TemporalEvent {
        claim: Some(superseded),
        ..metadata_event(
            TemporalEventKind::ClaimSuperseded,
            exact.clone(),
            &req.idempotency_key,
            "clear",
            json!({"deadline_id":id}),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    let retracted_event = TemporalEvent {
        claim: Some(claim),
        ..metadata_event(
            TemporalEventKind::ClaimRevised,
            exact.clone(),
            &req.idempotency_key,
            "clear",
            json!({"deadline_id":id}),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    persist_many(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        vec![superseded_event, retracted_event],
        "focusa.deadline_clear.v1",
    )
    .await
}

pub(super) async fn guard_issue(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let guard = req.guard.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "guard_required",
            "guard is required",
        )
    })?;
    if guard.scope != exact
        || guard.guard_id.trim().is_empty()
        || guard.priority_frame_ref.trim().is_empty()
        || guard.authorized_action_refs.is_empty()
        || !guard.preauthorized
        || guard.expires_at <= Utc::now()
    {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "guard_invalid",
            "guard must be exact-scope, fresh, preauthorized, and action-bounded",
        ));
    }
    let event = metadata_event(
        TemporalEventKind::GuardIssued,
        exact.clone(),
        &req.idempotency_key,
        "temporal_execution_guard",
        json!(guard),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.temporal_guard_issue.v1",
    )
    .await
}

pub(super) async fn guard_revoke(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let id = req.entity_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "guard_id_required",
            "entity_id is required",
        )
    })?;
    let log = ledger(exact.clone())?;
    let exists = read_events(&log)?.iter().any(|event| {
        event.event_kind == TemporalEventKind::GuardIssued
            && event
                .metadata
                .get("temporal_execution_guard")
                .and_then(|v| v.get("guard_id"))
                .and_then(Value::as_str)
                == Some(id)
    });
    if !exists {
        return Err(fail(
            StatusCode::NOT_FOUND,
            "guard_not_found",
            "cannot revoke an unknown exact-scope guard",
        ));
    }
    let event = metadata_event(
        TemporalEventKind::GuardIssued,
        exact.clone(),
        &req.idempotency_key,
        "guard_revocation",
        json!({"guard_id":id}),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.temporal_guard_revoke.v1",
    )
    .await
}

//! Canonical Spec 137 cancellation, estimate, progress, propagation, and validation surfaces.

use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use focusa_core::{
    temporal::{TemporalClaim, TemporalClaimKind, TemporalEvent, TemporalEventKind},
    temporal_deadline::{CivilTimeIntent, resolve_civil_time},
    temporal_operations::authorize_temporal_action,
    temporal_progress::is_material_progress,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use super::temporal::{ApiFailure, TemporalScopeDimensions, fail, ledger, scope};
use super::temporal_canonical_mutation::{
    CanonicalMutationRequest, CanonicalValidationRequest, exact_scope, metadata_event, persist,
    validate_claim_scope,
};

#[derive(Debug, Deserialize)]
pub(super) struct CanonicalCivilResolveRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    intent: CivilTimeIntent,
    local_datetime: String,
    idempotency_key: String,
    evidence_refs: Vec<String>,
    #[serde(default)]
    confirm: bool,
}

pub(super) async fn resolve_civil(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalCivilResolveRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = scope(req.project_root, req.continuity_id, req.dimensions);
    ledger(exact.clone())?;
    if req.idempotency_key.trim().is_empty() || req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "civil_time_authority_missing",
            "civil-time resolution requires stable idempotency and evidence",
        ));
    }
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "durable civil-time resolution requires confirm=true",
        ));
    }
    let local = chrono::NaiveDateTime::parse_from_str(&req.local_datetime, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| {
            fail(
                StatusCode::BAD_REQUEST,
                "invalid_civil_datetime",
                "local_datetime must use YYYY-MM-DDTHH:MM:SS",
            )
        })?;
    let resolved = resolve_civil_time(&req.intent, local).map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "civil_time_resolution_failed",
            format!("{error:?}"),
        )
    })?;
    let event = metadata_event(
        TemporalEventKind::CivilTimeResolved,
        exact.clone(),
        &req.idempotency_key,
        "civil_resolution",
        json!({"intent":req.intent,"resolved_instants":resolved}),
        &req.evidence_refs,
        None,
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.deadline_civil_resolution.v1",
    )
    .await
}

pub(super) async fn cancellation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let id = req.entity_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "cancellation_id_required",
            "entity_id is required",
        )
    })?;
    let event = metadata_event(
        TemporalEventKind::CancellationRequested,
        exact.clone(),
        &req.idempotency_key,
        "cancellation",
        json!({"cancellation_id":id,"metadata":req.metadata}),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.cancellation_request.v1",
    )
    .await
}

pub(super) async fn estimate_request(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let claim = req.claim.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "estimate_claim_required",
            "claim is required",
        )
    })?;
    if !matches!(
        claim.kind,
        TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
    ) || claim.uncertainty.is_none()
    {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "estimate_invalid",
            "estimate must remain non-commitment and include uncertainty",
        ));
    }
    validate_claim_scope(&claim, &exact)?;
    let event = TemporalEvent {
        claim: Some(claim),
        ..metadata_event(
            TemporalEventKind::ForecastIssued,
            exact.clone(),
            &req.idempotency_key,
            "estimate",
            json!(req.metadata),
            &req.evidence_refs,
            req.reason_code.as_ref(),
        )
    };
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.estimate_request.v1",
    )
    .await
}

pub(super) async fn estimate_evaluate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let id = req.entity_id.as_deref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "estimate_id_required",
            "entity_id is required",
        )
    })?;
    if req
        .metadata
        .get("actual_ms")
        .and_then(Value::as_u64)
        .is_none()
    {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "actual_duration_required",
            "actual_ms is required to evaluate an estimate",
        ));
    }
    let event = metadata_event(
        TemporalEventKind::ForecastEvaluated,
        exact.clone(),
        &req.idempotency_key,
        "estimate_evaluation",
        json!({"estimate_id":id,"evaluation":req.metadata}),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.estimate_evaluation.v1",
    )
    .await
}

pub(super) async fn progress_record(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    let signal = req.progress_signal.clone().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "progress_signal_required",
            "progress_signal is required",
        )
    })?;
    if signal.scope != exact || signal.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "progress_signal_invalid",
            "progress must have exact scope and evidence",
        ));
    }
    let material = is_material_progress(&signal);
    let event = metadata_event(
        TemporalEventKind::ProgressObserved,
        exact.clone(),
        &req.idempotency_key,
        "progress_signal",
        json!({"signal":signal,"material":material}),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.progress_record.v1",
    )
    .await
}

pub(super) async fn propagate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CanonicalMutationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = exact_scope(&req)?;
    if req.entity_id.as_deref().is_none_or(str::is_empty) {
        return Err(fail(
            StatusCode::BAD_REQUEST,
            "deadline_id_required",
            "entity_id is required",
        ));
    }
    let event = metadata_event(
        TemporalEventKind::DeadlineCompared,
        exact.clone(),
        &req.idempotency_key,
        "deadline_propagation",
        json!({"deadline_id":req.entity_id,"propagation":req.metadata}),
        &req.evidence_refs,
        req.reason_code.as_ref(),
    );
    persist(
        state.as_ref(),
        exact,
        &req.idempotency_key,
        event,
        "focusa.deadline_propagation.v1",
    )
    .await
}

pub(super) async fn validate_guard(
    Json(req): Json<CanonicalValidationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = scope(req.project_root, req.continuity_id, req.dimensions);
    ledger(exact.clone())?;
    let guard = req.guard.as_ref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "guard_required",
            "guard is required",
        )
    })?;
    let calendar = req.human_calendar_context.as_ref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "calendar_required",
            "human_calendar_context is required",
        )
    })?;
    let frame = req.temporal_priority_frame.as_ref().ok_or_else(|| {
        fail(
            StatusCode::BAD_REQUEST,
            "priority_frame_required",
            "temporal_priority_frame is required",
        )
    })?;
    authorize_temporal_action(
        calendar,
        frame,
        Some(guard),
        &exact,
        req.operator_ask_digest.as_deref().unwrap_or_default(),
        req.authorized_action_ref.as_deref().unwrap_or_default(),
        Utc::now(),
    )
    .map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "guard_validation_failed",
            format!("{error:?}"),
        )
    })?;
    Ok(Json(
        json!({"schema":"focusa.temporal_guard_validation.v1","status":"completed","canonical":true,"valid":true}),
    ))
}

pub(super) async fn validate_claims(
    Json(req): Json<CanonicalValidationRequest>,
) -> Result<Json<Value>, ApiFailure> {
    let exact = scope(req.project_root, req.continuity_id, req.dimensions);
    ledger(exact.clone())?;
    let claims = if req.claims.is_empty() {
        req.claim.into_iter().collect()
    } else {
        req.claims
    };
    if claims.is_empty() || req.evidence_refs.is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "claims_and_evidence_required",
            "typed claims and evidence are required",
        ));
    }
    for claim in &claims {
        validate_claim_scope(claim, &exact)?;
    }
    Ok(Json(
        json!({"schema":"focusa.response_temporal_claim_validation.v1","status":"completed","canonical":true,"valid":true,"claims":claims}),
    ))
}

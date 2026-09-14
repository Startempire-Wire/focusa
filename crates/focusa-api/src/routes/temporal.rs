//! Spec137 project-scoped temporal authority API.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    temporal::{
        TemporalClaim, TemporalClaimKind, TemporalClaimStatus, TemporalEvent, TemporalEventKind,
        TemporalLedger, TemporalScope, project_temporal, validate_claim,
    },
    temporal_claims::{TemporalClaimEnvelope, revise_claim, temporal_preflight},
    temporal_deadline::{CivilTimeIntent, resolve_civil_time},
    temporal_forecast::{
        ForecastAuthorityContext, ObservedDuration, ReleasePhase, evaluate_forecast,
        forecast_phase_authorized,
    },
    temporal_high_consequence::{
        ActivationFirewall, DispatchAgeObservation, DispatchAgePolicy, SignedTemporalLedgerControl,
        TemporalDataPolicy, TemporalPrecisionProfile, authorize_activation, authorize_dispatch,
        validate_data_policy, validate_ledger_controls, validate_precision_profile,
    },
    temporal_operations::{
        HumanCalendarContext, TemporalExecutionGuard, TemporalPriorityFrame,
        authorize_temporal_action,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use super::temporal_advanced::{
    capture_clock, commit_priority, forecast, high_consequence_preflight, migrate_signatures,
    resolve_civil,
};
use super::temporal_closure::settle_closure;
use super::temporal_conformance::spec137a_conformance_surface;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct TemporalScopeDimensions {
    #[serde(default)]
    pub(super) host_id: Option<String>,
    #[serde(default)]
    pub(super) operator_id: Option<String>,
    #[serde(default)]
    pub(super) workpoint_id: Option<String>,
    #[serde(default)]
    pub(super) item_id: Option<String>,
    #[serde(default)]
    pub(super) task_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalStatusQuery {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    #[serde(default)]
    pub(super) as_of: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalClaimRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    idempotency_key: String,
    claim: TemporalClaim,
    #[serde(default)]
    confirm: bool,
}

#[derive(Debug, Deserialize)]
pub struct TemporalObserveRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    idempotency_key: String,
    phase: ReleasePhase,
    duration_ms: u64,
    outcome: String,
    #[serde(default)]
    reason_code: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalPreflightRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    #[serde(default)]
    envelope: Option<TemporalClaimEnvelope>,
}

pub(super) type ApiFailure = (StatusCode, Json<Value>);

pub(super) fn fail(status: StatusCode, code: &str, message: impl Into<String>) -> ApiFailure {
    (
        status,
        Json(json!({
            "status":"blocked", "failure_class":code, "message":message.into(),
            "next_action":"repair temporal scope, authority, evidence, or freshness before retrying",
            "recovery_tools":["focusa_project_verify","focusa_temporal_authority"]
        })),
    )
}

pub(super) fn scope(
    project_root: String,
    continuity_id: String,
    dimensions: TemporalScopeDimensions,
) -> TemporalScope {
    let mut scope = TemporalScope::project(project_root, continuity_id);
    scope.host_id = dimensions.host_id;
    scope.operator_id = dimensions.operator_id;
    scope.workpoint_id = dimensions.workpoint_id;
    scope.item_id = dimensions.item_id;
    scope.task_id = dimensions.task_id;
    scope
}

pub(crate) use super::temporal_persistence::temporal_signing_key;
pub(super) use super::temporal_persistence::{append_signed_events, idempotent_replay_matches};

pub(super) use super::focus::project_active_temporal_frame as project_active_focus_frame;

pub(super) fn ledger(scope: TemporalScope) -> Result<TemporalLedger, (StatusCode, Json<Value>)> {
    TemporalLedger::for_project(scope).map_err(|error| {
        fail(
            StatusCode::BAD_REQUEST,
            "unsafe_temporal_scope",
            format!("{error:?}"),
        )
    })
}

fn event(
    scope: TemporalScope,
    event_kind: TemporalEventKind,
    claim: TemporalClaim,
    key: &str,
) -> TemporalEvent {
    TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind,
        scope,
        claim: Some(claim),
        clock_sample: None,
        metadata: Default::default(),
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: key.to_string(),
        digest: String::new(),
    }
}

pub(super) fn read_events(
    ledger: &TemporalLedger,
) -> Result<Vec<TemporalEvent>, (StatusCode, Json<Value>)> {
    ledger.read_all().map_err(|error| {
        fail(
            StatusCode::CONFLICT,
            "temporal_ledger_invalid",
            format!("{error:?}"),
        )
    })
}

pub(super) async fn status(
    Query(query): Query<TemporalStatusQuery>,
) -> Result<Json<Value>, ApiFailure> {
    let project_root = query.project_root.clone();
    let scope = scope(query.project_root, query.continuity_id, query.dimensions);
    let ledger = ledger(scope.clone())?;
    let as_of = query
        .as_of
        .as_deref()
        .map(str::parse)
        .transpose()
        .map_err(|_| {
            fail(
                StatusCode::BAD_REQUEST,
                "invalid_as_of",
                "as_of must be RFC3339",
            )
        })?
        .unwrap_or_else(Utc::now);
    let events = ledger.as_of(as_of).map_err(|error| {
        fail(
            StatusCode::CONFLICT,
            "temporal_ledger_invalid",
            format!("{error:?}"),
        )
    })?;
    let attested_legacy_digests = events
        .iter()
        .filter(|event| event.event_kind == TemporalEventKind::LegacySignatureAttestation)
        .filter_map(|event| event.metadata.get("legacy_event_digests"))
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let unsigned_legacy_event_count = events
        .iter()
        .filter(|event| {
            event.signature.is_none() && !attested_legacy_digests.contains(event.digest.as_str())
        })
        .count();
    let projection = project_temporal(scope, &events, as_of);
    Ok(Json(json!({
        "schema":"focusa.temporal_status.v1", "status":"completed", "canonical":true,
        "projection":projection, "event_count":events.len(),
        "conformance":spec137a_conformance_surface(),
        "unsigned_legacy_event_count":unsigned_legacy_event_count,
        "integrity_status":if unsigned_legacy_event_count==0 { "signed_verified" } else { "legacy_attestation_required" },
        "next_action":if projection.deadline_status==focusa_core::temporal::DeadlineStatus::None {
            "continue without fabricated urgency; commit a deadline only with authority and evidence"
        } else { "follow the active temporal preflight and evidence policy" }
    })))
}

async fn commit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalClaimRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    if req.claim.scope != scope {
        return Err(fail(
            StatusCode::CONFLICT,
            "scope_mismatch",
            "claim project_root + continuity_id does not match request scope",
        ));
    }
    if req.claim.kind == TemporalClaimKind::ExternalCommitment && !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "external commitment requires confirm=true",
        ));
    }
    validate_claim(&req.claim, None).map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "claim_validation_failed",
            format!("{error:?}"),
        )
    })?;
    let ledger = ledger(scope.clone())?;
    let events = append_signed_events(
        &ledger,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::ClaimCommitted,
            req.claim,
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_commit_result.v1", "status":"completed", "canonical":true,
        "events":events, "receipt_ref":format!("temporal:{}",req.idempotency_key),
        "next_action":"refresh temporal status and dependent forecasts"
    })))
}

async fn revise(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalClaimRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    if req.claim.scope != scope {
        return Err(fail(
            StatusCode::CONFLICT,
            "scope_mismatch",
            "claim scope mismatch",
        ));
    }
    let ledger = ledger(scope.clone())?;
    let events = read_events(&ledger)?;
    let previous = events
        .iter()
        .rev()
        .filter_map(|event| event.claim.as_ref())
        .find(|claim| claim.claim_id == req.claim.claim_id)
        .ok_or_else(|| {
            fail(
                StatusCode::NOT_FOUND,
                "claim_not_found",
                "cannot revise an unknown temporal claim",
            )
        })?;
    let (superseded, revised) = revise_claim(previous, req.claim).map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "revision_rejected",
            format!("{error:?}"),
        )
    })?;
    let appended = append_signed_events(
        &ledger,
        &req.idempotency_key,
        vec![
            event(
                scope.clone(),
                TemporalEventKind::ClaimSuperseded,
                superseded,
                &req.idempotency_key,
            ),
            event(
                scope.clone(),
                TemporalEventKind::ClaimRevised,
                revised,
                &req.idempotency_key,
            ),
        ],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_revision_result.v1", "status":"completed", "canonical":true,
        "events":appended, "receipt_ref":format!("temporal:{}",req.idempotency_key),
        "next_action":"refresh every temporal projection"
    })))
}

async fn observe(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalObserveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    let now = Utc::now();
    let claim = TemporalClaim {
        claim_id: format!("duration:{}:{}", req.idempotency_key, req.duration_ms),
        revision: 1,
        scope: scope.clone(),
        kind: TemporalClaimKind::ObservedDuration,
        status: TemporalClaimStatus::Canonical,
        subject_ref: format!("release_phase:{:?}", req.phase),
        target_at: None,
        duration_ms: Some(req.duration_ms),
        timezone: "UTC".into(),
        source: "observed_runtime".into(),
        source_ref: Some(format!("phase:{:?}", req.phase)),
        operator_confirmed: false,
        confidence: focusa_core::temporal::TemporalConfidence::Verified,
        uncertainty: None,
        observed_at: now,
        effective_at: now,
        expires_at: None,
        supersedes_revision: None,
        evidence_refs: req.evidence_refs,
        reason_code: req.reason_code,
    };
    let ledger = ledger(scope.clone())?;
    let events = append_signed_events(
        &ledger,
        &req.idempotency_key,
        vec![event(
            scope.clone(),
            TemporalEventKind::DurationObserved,
            claim,
            &req.idempotency_key,
        )],
    )?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_observation_result.v1", "status":"completed",
        "outcome":req.outcome, "events":events,
        "next_action":"recalculate the evidence-backed phase forecast"
    })))
}

async fn preflight(Json(req): Json<TemporalPreflightRequest>) -> Json<Value> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    Json(json!({
        "schema":"focusa.temporal_preflight_result.v1", "status":"completed",
        "preflight":temporal_preflight(&scope,req.envelope.as_ref(),Utc::now())
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    use super::{
        temporal_canonical_mutation as mutation, temporal_canonical_operations as operations,
        temporal_canonical_read as read,
    };
    Router::new()
        .route("/v1/time/awareness", get(read::awareness))
        .route("/v1/time/status", get(read::time_status))
        .route("/v1/time/stream", get(read::stream))
        .route("/v1/deadlines", get(read::deadlines))
        .route("/v1/deadline/conflicts", get(read::conflicts))
        .route("/v1/deadline/propagate", post(operations::propagate))
        .route("/v1/deadline/{id}", get(read::deadline))
        .route("/v1/temporal/guard/issue", post(mutation::guard_issue))
        .route(
            "/v1/temporal/guard/validate",
            post(operations::validate_guard),
        )
        .route("/v1/temporal/guard/revoke", post(mutation::guard_revoke))
        .route("/v1/cancellation/request", post(operations::cancellation))
        .route("/v1/estimate/history", get(read::estimates))
        .route("/v1/estimate/{id}", get(read::entity))
        .route(
            "/v1/response/temporal-claims/validate",
            post(operations::validate_claims),
        )
        .route("/v1/progress/status", get(read::progress))
        .route("/v1/no-progress/incidents", get(read::no_progress))
        .route("/v1/lost-time/incidents", get(read::lost_time))
        .route("/v1/opportunities", get(read::opportunities))
        .route("/v1/temporal/status", get(status))
        .route("/v1/temporal/commit", post(commit))
        .route("/v1/temporal/revise", post(revise))
        .route("/v1/temporal/observe", post(observe))
        .route("/v1/temporal/forecast", post(forecast))
        .route("/v1/temporal/preflight", post(preflight))
        .route("/v1/temporal/clock/capture", post(capture_clock))
        .route("/v1/temporal/priority/commit", post(commit_priority))
        .route("/v1/temporal/civil/resolve", post(resolve_civil))
        .route(
            "/v1/temporal/high-consequence/preflight",
            post(high_consequence_preflight),
        )
        .route("/v1/temporal/migrate-signatures", post(migrate_signatures))
        .route("/v1/temporal/settle-closure", post(settle_closure))
}

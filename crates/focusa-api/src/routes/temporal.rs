//! Spec137 project-scoped temporal authority API.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
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

#[derive(Debug, Default, Deserialize)]
pub struct TemporalScopeDimensions {
    #[serde(default)]
    host_id: Option<String>,
    #[serde(default)]
    operator_id: Option<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
    #[serde(default)]
    item_id: Option<String>,
    #[serde(default)]
    task_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalStatusQuery {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    #[serde(default)]
    as_of: Option<String>,
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

pub(super) fn fail(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<Value>) {
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

pub(crate) fn temporal_signing_key()
-> Result<(String, ed25519_dalek::SigningKey), (StatusCode, Json<Value>)> {
    match (
        std::env::var("FOCUSA_TEMPORAL_SIGNING_KEY_ID").ok(),
        std::env::var("FOCUSA_TEMPORAL_SIGNING_KEY").ok(),
    ) {
        (Some(key_id), Some(encoded)) => {
            let bytes: [u8; 32] = STANDARD
                .decode(encoded)
                .ok()
                .and_then(|bytes| bytes.try_into().ok())
                .ok_or_else(|| {
                    fail(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporal_signing_key_invalid",
                        "temporal signing key must be base64-encoded 32-byte Ed25519 material",
                    )
                })?;
            Ok((key_id, ed25519_dalek::SigningKey::from_bytes(&bytes)))
        }
        (None, None) => focusa_core::temporal_integrity::load_or_create_temporal_signing_key()
            .map_err(|error| {
                fail(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "temporal_signing_key_unavailable",
                    format!("host temporal signing key unavailable: {error:?}"),
                )
            }),
        _ => Err(fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "temporal_signing_key_incomplete",
            "set both temporal signing key environment variables or neither",
        )),
    }
}

pub(super) fn append_signed_events(
    ledger: &TemporalLedger,
    idempotency_key: &str,
    events: Vec<TemporalEvent>,
) -> Result<Vec<TemporalEvent>, (StatusCode, Json<Value>)> {
    let (key_id, signing_key) = temporal_signing_key()?;
    ledger
        .append_signed_batch(idempotency_key, events, &key_id, &signing_key)
        .map_err(|error| {
            fail(
                StatusCode::PRECONDITION_FAILED,
                "temporal_ledger_append_failed",
                format!("{error:?}"),
            )
        })
}

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

fn spec137a_conformance_surface(project_root: &str) -> Value {
    let path =
        std::path::Path::new(project_root).join("docs/contracts/spec137a-surface-parity.v1.yaml");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|body| serde_json::from_str(&body).ok())
        .unwrap_or_else(|| json!({
            "schema":"focusa.spec137a_surface_parity.v1",
            "status":"degraded",
            "full_conformance_status":"unknown",
            "warnings":["Spec137A surface parity artifact unavailable; full conformance is blocked."],
            "recovery_tools":["focusa_project_verify","focusa_temporal_authority","focusa_tool_doctor"]
        }))
}

async fn status(
    Query(query): Query<TemporalStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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
        "conformance":spec137a_conformance_surface(&project_root),
        "unsigned_legacy_event_count":unsigned_legacy_event_count,
        "integrity_status":if unsigned_legacy_event_count==0 { "signed_verified" } else { "legacy_attestation_required" },
        "next_action":if projection.deadline_status==focusa_core::temporal::DeadlineStatus::None {
            "continue without fabricated urgency; commit a deadline only with authority and evidence"
        } else { "follow the active temporal preflight and evidence policy" }
    })))
}

async fn commit(
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
            scope,
            TemporalEventKind::ClaimCommitted,
            req.claim,
            &req.idempotency_key,
        )],
    )?;
    Ok(Json(json!({
        "schema":"focusa.temporal_commit_result.v1", "status":"completed", "canonical":true,
        "events":events, "receipt_ref":format!("temporal:{}",req.idempotency_key),
        "next_action":"refresh temporal status and dependent forecasts"
    })))
}

async fn revise(
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
                scope,
                TemporalEventKind::ClaimRevised,
                revised,
                &req.idempotency_key,
            ),
        ],
    )?;
    Ok(Json(json!({
        "schema":"focusa.temporal_revision_result.v1", "status":"completed", "canonical":true,
        "events":appended, "receipt_ref":format!("temporal:{}",req.idempotency_key),
        "next_action":"refresh every temporal projection"
    })))
}

async fn observe(
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
            scope,
            TemporalEventKind::DurationObserved,
            claim,
            &req.idempotency_key,
        )],
    )?;
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
    Router::new()
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
}

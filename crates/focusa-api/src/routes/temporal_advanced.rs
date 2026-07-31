//! Spec137 forecast, civil-time, platform-clock, and high-consequence API operations.

use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use focusa_core::{
    temporal::{
        TemporalClaim, TemporalClaimKind, TemporalClaimStatus, TemporalEvent, TemporalEventKind,
        TemporalScope, project_temporal,
    },
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

use super::temporal::{
    TemporalScopeDimensions, append_signed_events, fail, ledger, project_active_focus_frame,
    read_events, scope,
};

#[derive(Debug, Deserialize)]
pub struct ForecastEvaluationRequest {
    exact_target_event_ref: String,
    baseline_score: f64,
    #[serde(default)]
    censored_sample_count: usize,
    #[serde(default)]
    correlated_cluster_count: usize,
    #[serde(default)]
    cohort_drift: f64,
    #[serde(default)]
    decision_value: f64,
    #[serde(default)]
    evidence_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalForecastRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    idempotency_key: String,
    phase: ReleasePhase,
    authority: ForecastAuthorityContext,
    #[serde(default)]
    actual_ms: Option<u64>,
    #[serde(default)]
    evaluation: Option<ForecastEvaluationRequest>,
}

#[derive(Debug, Deserialize)]
pub struct TemporalPriorityCommitRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    human_calendar_context: HumanCalendarContext,
    temporal_priority_frame: TemporalPriorityFrame,
    temporal_execution_guard: TemporalExecutionGuard,
    operator_ask_digest: String,
    authorized_action_ref: String,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TemporalCivilTimeResolveRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    intent: CivilTimeIntent,
    local_datetime: String,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TemporalClockCaptureRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    timezone: String,
    #[serde(default)]
    tzdb_version: Option<String>,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TemporalHighConsequencePreflightRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    precision_profile: TemporalPrecisionProfile,
    dispatch_policy: DispatchAgePolicy,
    dispatch_observation: DispatchAgeObservation,
    activation_firewall: ActivationFirewall,
    data_policy: TemporalDataPolicy,
    ledger_controls: SignedTemporalLedgerControl,
}

#[derive(Debug, Deserialize)]
pub struct TemporalSignatureMigrationRequest {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    idempotency_key: String,
    #[serde(default)]
    confirm: bool,
}

fn phase_from_claim(claim: &TemporalClaim) -> Option<ReleasePhase> {
    let raw = claim.source_ref.as_deref()?.strip_prefix("phase:")?;
    serde_json::from_str(&format!("\"{}\"", raw.to_ascii_lowercase())).ok()
}

pub(super) async fn forecast(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalForecastRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    let ledger = ledger(scope.clone())?;
    let observations = read_events(&ledger)?
        .into_iter()
        .filter(|event| event.event_kind == TemporalEventKind::DurationObserved)
        .filter_map(|event| event.claim)
        .filter_map(|claim| {
            Some(ObservedDuration {
                observation_id: claim.claim_id.clone(),
                scope: claim.scope.clone(),
                phase: phase_from_claim(&claim)?,
                duration_ms: claim.duration_ms?,
                outcome: "observed".into(),
                reason_code: claim.reason_code.clone(),
                started_at: claim.observed_at,
                completed_at: claim.effective_at,
                evidence_refs: claim.evidence_refs.clone(),
            })
        })
        .collect::<Vec<_>>();
    let range =
        forecast_phase_authorized(&scope, req.phase, &observations, req.authority, Utc::now())
            .map_err(|error| {
                fail(
                    StatusCode::PRECONDITION_FAILED,
                    "forecast_history_insufficient",
                    format!("{error:?}"),
                )
            })?;
    let evaluation = match (req.actual_ms, req.evaluation) {
        (Some(actual), Some(evaluation)) => Some(
            evaluate_forecast(
                &range,
                actual,
                evaluation.exact_target_event_ref,
                evaluation.baseline_score,
                evaluation.censored_sample_count,
                evaluation.correlated_cluster_count,
                evaluation.cohort_drift,
                evaluation.decision_value,
                evaluation.evidence_refs,
            )
            .map_err(|error| {
                fail(
                    StatusCode::PRECONDITION_FAILED,
                    "forecast_evaluation_failed",
                    format!("{error:?}"),
                )
            })?,
        ),
        (Some(_), None) => {
            return Err(fail(
                StatusCode::PRECONDITION_REQUIRED,
                "forecast_evaluation_context_required",
                "actual_ms requires exact target, baseline, censoring, correlation, drift, value, and evidence",
            ));
        }
        _ => None,
    };
    let mut issued_metadata = std::collections::BTreeMap::new();
    issued_metadata.insert(
        "forecast".into(),
        serde_json::to_value(&range).unwrap_or(Value::Null),
    );
    let issued = TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: TemporalEventKind::ForecastIssued,
        scope: scope.clone(),
        claim: None,
        clock_sample: None,
        metadata: issued_metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let mut forecast_events = vec![issued];
    if let Some(evaluation) = evaluation.as_ref() {
        let mut evaluation_metadata = std::collections::BTreeMap::new();
        evaluation_metadata.insert(
            "evaluation".into(),
            serde_json::to_value(evaluation).unwrap_or(Value::Null),
        );
        forecast_events.push(TemporalEvent {
            event_id: Uuid::now_v7().to_string(),
            sequence: 0,
            event_kind: TemporalEventKind::ForecastEvaluated,
            scope: scope.clone(),
            claim: None,
            clock_sample: None,
            metadata: evaluation_metadata,
            signature: None,
            predecessor_digest: None,
            recorded_at: Utc::now(),
            idempotency_key: String::new(),
            digest: String::new(),
        });
    }
    let signed_events = append_signed_events(&ledger, &req.idempotency_key, forecast_events)?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_forecast.v1", "status":"completed", "canonical":false,
        "forecast":range, "evaluation":evaluation,"events":signed_events,
        "receipt_ref":format!("temporal-forecast:{}",req.idempotency_key),
        "next_action":"treat this as a range; never convert it into a commitment without authority"
    })))
}

pub(super) async fn commit_priority(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalPriorityCommitRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    authorize_temporal_action(
        &req.human_calendar_context,
        &req.temporal_priority_frame,
        Some(&req.temporal_execution_guard),
        &scope,
        &req.operator_ask_digest,
        &req.authorized_action_ref,
        Utc::now(),
    )
    .map_err(|error| {
        fail(
            StatusCode::PRECONDITION_FAILED,
            "temporal_priority_invalid",
            format!("{error:?}"),
        )
    })?;
    let ledger = ledger(scope.clone())?;
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "human_calendar_context".into(),
        serde_json::to_value(&req.human_calendar_context).unwrap_or(Value::Null),
    );
    metadata.insert(
        "temporal_priority_frame".into(),
        serde_json::to_value(&req.temporal_priority_frame).unwrap_or(Value::Null),
    );
    metadata.insert(
        "temporal_execution_guard".into(),
        serde_json::to_value(&req.temporal_execution_guard).unwrap_or(Value::Null),
    );
    let event = TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: TemporalEventKind::GuardIssued,
        scope: scope.clone(),
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let events = append_signed_events(&ledger, &req.idempotency_key, vec![event])?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_priority_commit.v1","status":"completed","canonical":true,
        "events":events,"receipt_ref":format!("temporal-priority:{}",req.idempotency_key)
    })))
}

pub(super) async fn resolve_civil(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalCivilTimeResolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    let ledger = ledger(scope.clone())?;
    let local = chrono::NaiveDateTime::parse_from_str(&req.local_datetime, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| {
            fail(
                StatusCode::PRECONDITION_FAILED,
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
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "civil_time_intent".into(),
        serde_json::to_value(&req.intent).unwrap_or(Value::Null),
    );
    metadata.insert("resolved_instants".into(), json!(resolved));
    let event = TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: TemporalEventKind::CivilTimeResolved,
        scope: scope.clone(),
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let events = append_signed_events(&ledger, &req.idempotency_key, vec![event])?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_civil_resolution.v1","status":"completed","canonical":true,
        "resolved_instants":resolved,"events":events,
        "receipt_ref":format!("temporal-civil:{}",req.idempotency_key)
    })))
}

pub(super) async fn capture_clock(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalClockCaptureRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    let ledger = ledger(scope.clone())?;
    let sample = focusa_core::temporal_platform::capture_temporal_clock_sample(
        req.timezone,
        req.tzdb_version,
    );
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "platform_capabilities".into(),
        serde_json::to_value(
            focusa_core::temporal_platform::capture_platform_clocks().capabilities,
        )
        .unwrap_or(Value::Null),
    );
    let capture = TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: TemporalEventKind::ClockSampleObserved,
        scope: scope.clone(),
        claim: None,
        clock_sample: Some(sample.clone()),
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let events = append_signed_events(&ledger, &req.idempotency_key, vec![capture])?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_clock_capture.v1","status":"completed","canonical":true,
        "sample":sample,"events":events,"receipt_ref":format!("temporal-clock:{}",req.idempotency_key)
    })))
}

pub(super) async fn high_consequence_preflight(
    Json(req): Json<TemporalHighConsequencePreflightRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    if req.precision_profile.profile_id.trim().is_empty() || scope.project_root.trim().is_empty() {
        return Err(fail(
            StatusCode::PRECONDITION_FAILED,
            "high_consequence_scope_invalid",
            "high-consequence temporal scope and precision profile are required",
        ));
    }
    validate_precision_profile(&req.precision_profile)
        .and_then(|_| authorize_dispatch(&req.dispatch_policy, &req.dispatch_observation))
        .and_then(|_| authorize_activation(&req.activation_firewall))
        .and_then(|_| validate_data_policy(&req.data_policy))
        .and_then(|_| validate_ledger_controls(&req.ledger_controls))
        .map_err(|error| {
            fail(
                StatusCode::PRECONDITION_FAILED,
                "high_consequence_temporal_preflight_failed",
                format!("{error:?}"),
            )
        })?;
    Ok(Json(json!({
        "schema":"focusa.temporal_high_consequence_preflight.v1",
        "status":"completed","canonical":false,"dispatch_authorized":true,
        "scope":scope,"precision_profile":req.precision_profile,
        "next_action":"preserve this preflight receipt through dispatch, acknowledgement, and reconciliation"
    })))
}

pub(super) async fn migrate_signatures(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TemporalSignatureMigrationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !req.confirm {
        return Err(fail(
            StatusCode::PRECONDITION_REQUIRED,
            "confirmation_required",
            "legacy temporal signature attestation requires confirm=true",
        ));
    }
    let scope = scope(req.project_root, req.continuity_id, req.dimensions);
    let ledger = ledger(scope.clone())?;
    let events = read_events(&ledger)?;
    let unsigned_digests = events
        .iter()
        .filter(|event| event.signature.is_none())
        .map(|event| event.digest.clone())
        .collect::<Vec<_>>();
    if unsigned_digests.is_empty() {
        return Ok(Json(json!({
            "schema":"focusa.temporal_signature_migration.v1",
            "status":"completed","idempotent_replay":true,"unsigned_event_count":0
        })));
    }
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("legacy_event_digests".into(), json!(unsigned_digests));
    metadata.insert(
        "migration_policy".into(),
        json!("append_only_attestation_no_history_rewrite"),
    );
    let attestation = TemporalEvent {
        event_id: Uuid::now_v7().to_string(),
        sequence: 0,
        event_kind: TemporalEventKind::LegacySignatureAttestation,
        scope: scope.clone(),
        claim: None,
        clock_sample: None,
        metadata,
        signature: None,
        predecessor_digest: None,
        recorded_at: Utc::now(),
        idempotency_key: String::new(),
        digest: String::new(),
    };
    let appended = append_signed_events(&ledger, &req.idempotency_key, vec![attestation])?;
    project_active_focus_frame(state.as_ref(), &scope, &ledger).await?;
    Ok(Json(json!({
        "schema":"focusa.temporal_signature_migration.v1",
        "status":"completed","canonical":true,"events":appended,
        "unsigned_event_count":events.iter().filter(|event| event.signature.is_none()).count(),
        "receipt_ref":format!("temporal-signature-migration:{}",req.idempotency_key)
    })))
}

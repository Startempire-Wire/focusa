//! Request contracts for advanced temporal authority routes.

use focusa_core::{
    temporal_deadline::CivilTimeIntent,
    temporal_forecast::{ForecastAuthorityContext, ReleasePhase},
    temporal_high_consequence::{
        ActivationFirewall, DispatchAgeObservation, DispatchAgePolicy, SignedTemporalLedgerControl,
        TemporalDataPolicy, TemporalPrecisionProfile,
    },
    temporal_operations::{HumanCalendarContext, TemporalExecutionGuard, TemporalPriorityFrame},
};
use serde::Deserialize;

use super::temporal::TemporalScopeDimensions;

#[derive(Debug, Deserialize)]
pub(super) struct ForecastEvaluationRequest {
    pub(super) exact_target_event_ref: String,
    pub(super) baseline_score: f64,
    #[serde(default)]
    pub(super) censored_sample_count: usize,
    #[serde(default)]
    pub(super) correlated_cluster_count: usize,
    #[serde(default)]
    pub(super) cohort_drift: f64,
    #[serde(default)]
    pub(super) decision_value: f64,
    #[serde(default)]
    pub(super) evidence_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalForecastRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) idempotency_key: String,
    pub(super) phase: ReleasePhase,
    pub(super) authority: ForecastAuthorityContext,
    #[serde(default)]
    pub(super) actual_ms: Option<u64>,
    #[serde(default)]
    pub(super) evaluation: Option<ForecastEvaluationRequest>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalPriorityCommitRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) human_calendar_context: HumanCalendarContext,
    pub(super) temporal_priority_frame: TemporalPriorityFrame,
    pub(super) temporal_execution_guard: TemporalExecutionGuard,
    pub(super) operator_ask_digest: String,
    pub(super) authorized_action_ref: String,
    pub(super) idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalCivilTimeResolveRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) intent: CivilTimeIntent,
    pub(super) local_datetime: String,
    pub(super) idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalClockCaptureRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) timezone: String,
    #[serde(default)]
    pub(super) tzdb_version: Option<String>,
    pub(super) idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalHighConsequencePreflightRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) precision_profile: TemporalPrecisionProfile,
    pub(super) dispatch_policy: DispatchAgePolicy,
    pub(super) dispatch_observation: DispatchAgeObservation,
    pub(super) activation_firewall: ActivationFirewall,
    pub(super) data_policy: TemporalDataPolicy,
    pub(super) ledger_controls: SignedTemporalLedgerControl,
}

#[derive(Debug, Deserialize)]
pub(super) struct TemporalSignatureMigrationRequest {
    pub(super) project_root: String,
    pub(super) continuity_id: String,
    #[serde(flatten)]
    pub(super) dimensions: TemporalScopeDimensions,
    pub(super) idempotency_key: String,
    #[serde(default)]
    pub(super) confirm: bool,
}

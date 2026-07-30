//! Spec137 temporal authority primitives.
//! Clock fact, commitment, estimate, forecast, urgency, and presentation are distinct planes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClockDomain {
    WallUtc,
    MonotonicActive,
    SuspendAwareElapsed,
    CivilTimeIntent,
    TaiProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClaimKind {
    ClockFact,
    ExternalCommitment,
    InternalReadinessTarget,
    Estimate,
    Forecast,
    UrgencySignal,
    PresentationHint,
    NoDeadline,
    ObservedDuration,
    MissedTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClaimStatus {
    Proposed,
    Canonical,
    Superseded,
    Satisfied,
    Breached,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalConfidence {
    Unavailable,
    Low,
    Medium,
    High,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalUncertainty {
    pub earliest_at: Option<DateTime<Utc>>,
    pub latest_at: Option<DateTime<Utc>>,
    pub coverage_probability: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalScope {
    pub project_root: String,
    pub continuity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

impl TemporalScope {
    pub fn project(project_root: impl Into<String>, continuity_id: impl Into<String>) -> Self {
        Self {
            project_root: project_root.into(),
            continuity_id: continuity_id.into(),
            host_id: None,
            operator_id: None,
            workpoint_id: None,
            item_id: None,
            task_id: None,
        }
    }

    pub fn same_workstream(&self, other: &Self) -> bool {
        self.project_root == other.project_root && self.continuity_id == other.continuity_id
    }

    pub fn matches_filter(&self, filter: &Self) -> bool {
        self.same_workstream(filter)
            && [
                (&self.host_id, &filter.host_id),
                (&self.operator_id, &filter.operator_id),
                (&self.workpoint_id, &filter.workpoint_id),
                (&self.item_id, &filter.item_id),
                (&self.task_id, &filter.task_id),
            ]
            .into_iter()
            .all(|(actual, expected)| expected.is_none() || actual.is_none() || actual == expected)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClockSample {
    pub sample_id: String,
    pub domain: TemporalClockDomain,
    pub wall_utc: DateTime<Utc>,
    pub monotonic_ns: Option<u128>,
    pub suspend_aware_ns: Option<u128>,
    pub boot_id: Option<String>,
    pub timezone: String,
    pub tzdb_version: Option<String>,
    pub source: String,
    pub observed_offset_ns: Option<i128>,
    pub measurement_uncertainty_ns: u128,
    pub confidence: TemporalConfidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalClaim {
    pub claim_id: String,
    pub revision: u64,
    pub scope: TemporalScope,
    pub kind: TemporalClaimKind,
    pub status: TemporalClaimStatus,
    pub subject_ref: String,
    pub target_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub timezone: String,
    pub source: String,
    pub source_ref: Option<String>,
    pub operator_confirmed: bool,
    pub confidence: TemporalConfidence,
    pub uncertainty: Option<TemporalUncertainty>,
    pub observed_at: DateTime<Utc>,
    pub effective_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub supersedes_revision: Option<u64>,
    pub evidence_refs: Vec<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalEventKind {
    ClockSampleObserved,
    ClaimProposed,
    ClaimCommitted,
    ClaimRevised,
    ClaimSuperseded,
    DurationObserved,
    TargetSatisfied,
    TargetBreached,
    MissedTargetRecorded,
    ClockCorrectionRecorded,
    SourceQuarantined,
    SourceRecovered,
    CivilTimeResolved,
    DeadlineCompared,
    GuardIssued,
    CancellationRequested,
    CancellationAcknowledged,
    ClosurePostureRecorded,
    ReceiptLinked,
    ProgressObserved,
    TemporalPulseEvaluated,
    LostTimeIncidentRecorded,
    ForecastIssued,
    ForecastEvaluated,
    LegacySignatureAttestation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub event_id: String,
    pub sequence: u64,
    pub event_kind: TemporalEventKind,
    pub scope: TemporalScope,
    pub claim: Option<TemporalClaim>,
    pub clock_sample: Option<TemporalClockSample>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub metadata: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<crate::temporal_integrity::TemporalEventSignature>,
    pub predecessor_digest: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub idempotency_key: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineStatus {
    None,
    Committed,
    ForecastOnly,
    Conflicted,
    Satisfied,
    Breached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalProjection {
    pub scope: TemporalScope,
    pub as_of: DateTime<Utc>,
    pub deadline_status: DeadlineStatus,
    pub approaching_deadlines: Vec<TemporalClaim>,
    pub deadline_conflict_state: String,
    pub human_calendar_context: Option<crate::temporal_operations::HumanCalendarContext>,
    pub temporal_priority_frame: Option<crate::temporal_operations::TemporalPriorityFrame>,
    pub temporal_execution_guard: Option<crate::temporal_operations::TemporalExecutionGuard>,
    pub authorized_forecast_range: Option<crate::temporal_forecast::ForecastRange>,
    pub latest_forecast_evaluation: Option<crate::temporal_forecast::ForecastEvaluation>,
    pub active_commitment: Option<TemporalClaim>,
    pub active_forecast: Option<TemporalClaim>,
    pub observed_duration_count: usize,
    pub critical_path_ms: Option<u64>,
    pub slack_ms: Option<i64>,
    pub urgency: Option<TemporalClaim>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalValidationError {
    UnsafeProjectRoot,
    MissingContinuity,
    MissingTimezone,
    MissingSource,
    MissingSubject,
    CommitmentRequiresConfirmation,
    CommitmentRequiresTarget,
    EstimateCannotBecomeCommitment,
    InvalidUncertaintyRange,
    InvalidCoverageProbability,
    RevisionMustAdvance,
    SupersessionRequired,
}

pub fn validate_claim(
    claim: &TemporalClaim,
    previous: Option<&TemporalClaim>,
) -> Result<(), TemporalValidationError> {
    let root = claim.scope.project_root.trim();
    if !root.starts_with('/') || matches!(root, "/" | "/root" | "/home" | "/tmp") {
        return Err(TemporalValidationError::UnsafeProjectRoot);
    }
    if claim.scope.continuity_id.trim().is_empty() {
        return Err(TemporalValidationError::MissingContinuity);
    }
    if claim.timezone.trim().is_empty() {
        return Err(TemporalValidationError::MissingTimezone);
    }
    if claim.source.trim().is_empty() {
        return Err(TemporalValidationError::MissingSource);
    }
    if claim.subject_ref.trim().is_empty() {
        return Err(TemporalValidationError::MissingSubject);
    }
    if claim.kind == TemporalClaimKind::ExternalCommitment {
        if !claim.operator_confirmed {
            return Err(TemporalValidationError::CommitmentRequiresConfirmation);
        }
        if claim.target_at.is_none() {
            return Err(TemporalValidationError::CommitmentRequiresTarget);
        }
    }
    if matches!(
        claim.kind,
        TemporalClaimKind::Estimate | TemporalClaimKind::Forecast
    ) && claim.status == TemporalClaimStatus::Canonical
        && claim.operator_confirmed
    {
        return Err(TemporalValidationError::EstimateCannotBecomeCommitment);
    }
    if let Some(uncertainty) = &claim.uncertainty {
        if uncertainty
            .earliest_at
            .zip(uncertainty.latest_at)
            .is_some_and(|(earliest, latest)| earliest > latest)
        {
            return Err(TemporalValidationError::InvalidUncertaintyRange);
        }
        if uncertainty
            .coverage_probability
            .is_some_and(|probability| !(0.0..=1.0).contains(&probability))
        {
            return Err(TemporalValidationError::InvalidCoverageProbability);
        }
    }
    if let Some(previous) = previous {
        if claim.claim_id != previous.claim_id || claim.revision <= previous.revision {
            return Err(TemporalValidationError::RevisionMustAdvance);
        }
        if claim.supersedes_revision != Some(previous.revision) {
            return Err(TemporalValidationError::SupersessionRequired);
        }
    }
    Ok(())
}

pub fn temporal_event_digest(event: &TemporalEvent) -> String {
    let mut canonical = event.clone();
    canonical.digest.clear();
    canonical.signature = None;
    let bytes = serde_json::to_vec(&canonical).expect("TemporalEvent serializes");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn seal_event(mut event: TemporalEvent) -> TemporalEvent {
    event.digest = temporal_event_digest(&event);
    event
}

pub fn verify_event_chain(events: &[TemporalEvent]) -> bool {
    events.iter().enumerate().all(|(index, event)| {
        event.sequence == index as u64 + 1
            && event.digest == temporal_event_digest(event)
            && event.predecessor_digest.as_deref()
                == index
                    .checked_sub(1)
                    .map(|previous| events[previous].digest.as_str())
    })
}

pub use crate::temporal_ledger::{TemporalLedger, TemporalLedgerError};

pub fn project_temporal(
    scope: TemporalScope,
    events: &[TemporalEvent],
    as_of: DateTime<Utc>,
) -> TemporalProjection {
    let mut latest_by_claim = HashMap::<&str, (usize, &TemporalClaim)>::new();
    for (index, event) in events.iter().enumerate() {
        if event.scope.matches_filter(&scope)
            && event.recorded_at <= as_of
            && let Some(claim) = event.claim.as_ref()
        {
            latest_by_claim.insert(&claim.claim_id, (index, claim));
        }
    }
    let mut latest = latest_by_claim.into_values().collect::<Vec<_>>();
    latest.sort_by_key(|(index, _)| *index);
    let active = latest
        .into_iter()
        .map(|(_, claim)| claim)
        .filter(|claim| {
            matches!(
                claim.status,
                TemporalClaimStatus::Canonical
                    | TemporalClaimStatus::Breached
                    | TemporalClaimStatus::Satisfied
            ) && claim.effective_at <= as_of
                && claim
                    .expires_at
                    .is_none_or(|expires_at| expires_at >= as_of)
        })
        .collect::<Vec<_>>();
    let commitment = active
        .iter()
        .rev()
        .find(|claim| claim.kind == TemporalClaimKind::ExternalCommitment)
        .cloned()
        .cloned();
    let forecast = active
        .iter()
        .rev()
        .find(|claim| claim.kind == TemporalClaimKind::Forecast)
        .cloned()
        .cloned();
    let deadline_status = match commitment.as_ref() {
        Some(claim) if claim.status == TemporalClaimStatus::Breached => DeadlineStatus::Breached,
        Some(claim) if claim.status == TemporalClaimStatus::Satisfied => DeadlineStatus::Satisfied,
        Some(_) => DeadlineStatus::Committed,
        None if forecast.is_some() => DeadlineStatus::ForecastOnly,
        None => DeadlineStatus::None,
    };
    let mut approaching_deadlines = active
        .iter()
        .filter(|claim| {
            matches!(
                claim.kind,
                TemporalClaimKind::ExternalCommitment | TemporalClaimKind::InternalReadinessTarget
            ) && claim.target_at.is_some()
        })
        .cloned()
        .cloned()
        .collect::<Vec<_>>();
    approaching_deadlines.sort_by_key(|claim| claim.target_at);
    approaching_deadlines.truncate(5);
    let deadline_conflict_state = if approaching_deadlines.len() > 1 {
        "unknown"
    } else {
        "feasible"
    };
    let priority_event = events.iter().rev().find(|event| {
        event.scope.matches_filter(&scope)
            && event.recorded_at <= as_of
            && event.event_kind == TemporalEventKind::GuardIssued
    });
    let human_calendar_context = priority_event
        .and_then(|event| event.metadata.get("human_calendar_context"))
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let temporal_priority_frame = priority_event
        .and_then(|event| event.metadata.get("temporal_priority_frame"))
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let temporal_execution_guard = priority_event
        .and_then(|event| event.metadata.get("temporal_execution_guard"))
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let authorized_forecast_range = events
        .iter()
        .rev()
        .find(|event| {
            event.scope.matches_filter(&scope)
                && event.event_kind == TemporalEventKind::ForecastIssued
        })
        .and_then(|event| event.metadata.get("forecast"))
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let latest_forecast_evaluation = events
        .iter()
        .rev()
        .find(|event| {
            event.scope.matches_filter(&scope)
                && event.event_kind == TemporalEventKind::ForecastEvaluated
        })
        .and_then(|event| event.metadata.get("evaluation"))
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    TemporalProjection {
        scope,
        as_of,
        deadline_status,
        approaching_deadlines,
        deadline_conflict_state: deadline_conflict_state.into(),
        human_calendar_context,
        temporal_priority_frame,
        temporal_execution_guard,
        authorized_forecast_range,
        latest_forecast_evaluation,
        active_commitment: commitment,
        active_forecast: forecast,
        observed_duration_count: events
            .iter()
            .filter(|event| event.event_kind == TemporalEventKind::DurationObserved)
            .count(),
        critical_path_ms: None,
        slack_ms: None,
        urgency: active
            .iter()
            .rev()
            .find(|claim| claim.kind == TemporalClaimKind::UrgencySignal)
            .cloned()
            .cloned(),
        warnings: Vec::new(),
    }
}

#[cfg(test)]
#[path = "temporal_tests.rs"]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::temporal::TemporalScope;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationPredictionBaseline {
    pub estimate_ns: u128,
    pub lower_bound_ns: u128,
    pub upper_bound_ns: Option<u128>,
    pub source: String,
    pub sample_count: u64,
    pub cohort_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationPredictionBaselineError {
    MissingSource,
    InvalidBounds,
    FabricatedColdStartPrecision,
}

pub fn validate_duration_prediction_baseline(
    baseline: &DurationPredictionBaseline,
) -> Result<(), DurationPredictionBaselineError> {
    if baseline.source.trim().is_empty() || baseline.cohort_key.trim().is_empty() {
        return Err(DurationPredictionBaselineError::MissingSource);
    }
    if baseline.estimate_ns < baseline.lower_bound_ns
        || baseline
            .upper_bound_ns
            .is_some_and(|upper| upper < baseline.lower_bound_ns || baseline.estimate_ns > upper)
    {
        return Err(DurationPredictionBaselineError::InvalidBounds);
    }
    if baseline.sample_count == 0
        && (baseline.estimate_ns != 0
            || baseline.lower_bound_ns != 0
            || baseline.upper_bound_ns.is_some())
    {
        return Err(DurationPredictionBaselineError::FabricatedColdStartPrecision);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LookupLocationKind {
    MemoryCache,
    ProcessCache,
    Sqlite,
    Filesystem,
    Network,
    Provider,
    Peer,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheDisposition {
    Hit,
    Miss,
    Stale,
    Revalidated,
    Bypassed,
    Unavailable,
    NotApplicable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupLatencyComponents {
    pub queue_ns: Option<u128>,
    pub network_ns: Option<u128>,
    pub storage_ns: Option<u128>,
    pub deserialize_ns: Option<u128>,
    pub compute_ns: Option<u128>,
}

impl LookupLatencyComponents {
    fn observed_sum(&self) -> u128 {
        [
            self.queue_ns,
            self.network_ns,
            self.storage_ns,
            self.deserialize_ns,
            self.compute_ns,
        ]
        .into_iter()
        .flatten()
        .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupTimingSpan {
    pub span_id: String,
    pub action_id: String,
    pub parent_span_id: Option<String>,
    pub parallel_group_id: Option<String>,
    pub location_kind: LookupLocationKind,
    pub location_ref: String,
    pub provider_ref: Option<String>,
    pub storage_tier: Option<String>,
    pub cache_disposition: CacheDisposition,
    pub cache_age_ns: Option<u128>,
    pub started_monotonic_ns: u128,
    pub ended_monotonic_ns: u128,
    pub elapsed_ns: u128,
    pub expected_elapsed_ns: Option<u128>,
    pub expected_actual_delta_ns: Option<i128>,
    pub components: LookupLatencyComponents,
    pub critical_path_contribution_ns: u128,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionTimingTrace {
    pub trace_id: String,
    pub action_id: String,
    pub prediction_id: String,
    pub started_temporal_envelope_ref: String,
    pub completed_temporal_envelope_ref: String,
    pub started_monotonic_ns: u128,
    pub completed_monotonic_ns: u128,
    pub total_elapsed_ns: u128,
    pub spans: Vec<LookupTimingSpan>,
    pub attributed_union_ns: u128,
    pub unattributed_ns: u128,
    pub reconciliation_delta_ns: i128,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTimingTraceError {
    EmptyIdentity,
    InvalidActionInterval,
    DuplicateSpanId,
    ForeignActionSpan,
    MissingParentSpan,
    InvalidSpanInterval,
    InvalidSpanElapsed,
    InvalidExpectedActualDelta,
    ComponentsExceedSpan,
    SpanOutsideAction,
    ParallelCriticalPathConflict,
    CriticalPathExceedsAction,
    InvalidAttributedUnion,
    InvalidReconciliation,
}

fn interval_union_ns(spans: &[LookupTimingSpan]) -> u128 {
    let mut intervals = spans
        .iter()
        .map(|span| (span.started_monotonic_ns, span.ended_monotonic_ns))
        .collect::<Vec<_>>();
    intervals.sort_unstable_by_key(|interval| interval.0);
    let mut total = 0_u128;
    let mut current: Option<(u128, u128)> = None;
    for (start, end) in intervals {
        match current {
            None => current = Some((start, end)),
            Some((current_start, current_end)) if start <= current_end => {
                current = Some((current_start, current_end.max(end)));
            }
            Some((current_start, current_end)) => {
                total = total.saturating_add(current_end.saturating_sub(current_start));
                current = Some((start, end));
            }
        }
    }
    if let Some((start, end)) = current {
        total = total.saturating_add(end.saturating_sub(start));
    }
    total
}

pub fn validate_action_timing_trace(
    trace: &ActionTimingTrace,
) -> Result<(), ActionTimingTraceError> {
    if trace.trace_id.trim().is_empty()
        || trace.action_id.trim().is_empty()
        || trace.prediction_id.trim().is_empty()
        || trace.started_temporal_envelope_ref.trim().is_empty()
        || trace.completed_temporal_envelope_ref.trim().is_empty()
    {
        return Err(ActionTimingTraceError::EmptyIdentity);
    }
    if trace.completed_monotonic_ns < trace.started_monotonic_ns
        || trace.total_elapsed_ns
            != trace
                .completed_monotonic_ns
                .saturating_sub(trace.started_monotonic_ns)
    {
        return Err(ActionTimingTraceError::InvalidActionInterval);
    }
    let ids = trace
        .spans
        .iter()
        .map(|span| span.span_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    if ids.len() != trace.spans.len() {
        return Err(ActionTimingTraceError::DuplicateSpanId);
    }
    let mut critical_parallel_groups = std::collections::HashSet::new();
    let mut critical_sum = 0_u128;
    for span in &trace.spans {
        if span.action_id != trace.action_id {
            return Err(ActionTimingTraceError::ForeignActionSpan);
        }
        if span
            .parent_span_id
            .as_deref()
            .is_some_and(|parent| !ids.contains(parent))
        {
            return Err(ActionTimingTraceError::MissingParentSpan);
        }
        if span.ended_monotonic_ns < span.started_monotonic_ns {
            return Err(ActionTimingTraceError::InvalidSpanInterval);
        }
        if span.elapsed_ns
            != span
                .ended_monotonic_ns
                .saturating_sub(span.started_monotonic_ns)
        {
            return Err(ActionTimingTraceError::InvalidSpanElapsed);
        }
        if span
            .expected_elapsed_ns
            .map(|expected| span.elapsed_ns as i128 - expected as i128)
            != span.expected_actual_delta_ns
        {
            return Err(ActionTimingTraceError::InvalidExpectedActualDelta);
        }
        if span.components.observed_sum() > span.elapsed_ns {
            return Err(ActionTimingTraceError::ComponentsExceedSpan);
        }
        if span.started_monotonic_ns < trace.started_monotonic_ns
            || span.ended_monotonic_ns > trace.completed_monotonic_ns
        {
            return Err(ActionTimingTraceError::SpanOutsideAction);
        }
        if span.critical_path_contribution_ns > span.elapsed_ns {
            return Err(ActionTimingTraceError::CriticalPathExceedsAction);
        }
        if span.critical_path_contribution_ns > 0 {
            critical_sum = critical_sum.saturating_add(span.critical_path_contribution_ns);
            if let Some(group) = span.parallel_group_id.as_deref()
                && !critical_parallel_groups.insert(group)
            {
                return Err(ActionTimingTraceError::ParallelCriticalPathConflict);
            }
        }
    }
    if critical_sum > trace.total_elapsed_ns {
        return Err(ActionTimingTraceError::CriticalPathExceedsAction);
    }
    let attributed_union_ns = interval_union_ns(&trace.spans);
    if trace.attributed_union_ns != attributed_union_ns
        || attributed_union_ns > trace.total_elapsed_ns
    {
        return Err(ActionTimingTraceError::InvalidAttributedUnion);
    }
    let reconciled = trace
        .attributed_union_ns
        .saturating_add(trace.unattributed_ns);
    if trace.reconciliation_delta_ns != trace.total_elapsed_ns as i128 - reconciled as i128
        || trace.reconciliation_delta_ns != 0
    {
        return Err(ActionTimingTraceError::InvalidReconciliation);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressSignalKind {
    TargetStateAdvanced,
    AcceptanceEvidenceAdded,
    BlockerRemoved,
    ActivityOnly,
    EquivalentAction,
    UnchangedReread,
    ResearchWithoutBound,
    RepeatedFullProof,
    SilentTool,
    CompactionChurn,
    DuplicatedHandoff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressSignal {
    pub signal_id: String,
    pub scope: TemporalScope,
    pub target_ref: String,
    pub target_revision_before: Option<String>,
    pub target_revision_after: Option<String>,
    pub kind: ProgressSignalKind,
    pub evidence_refs: Vec<String>,
    pub observed_at: DateTime<Utc>,
    pub equivalence_digest: Option<String>,
    pub false_positive_guard_refs: Vec<String>,
}

pub fn is_material_progress(signal: &ProgressSignal) -> bool {
    matches!(
        signal.kind,
        ProgressSignalKind::TargetStateAdvanced
            | ProgressSignalKind::AcceptanceEvidenceAdded
            | ProgressSignalKind::BlockerRemoved
    ) && !signal.evidence_refs.is_empty()
        && signal.target_revision_before != signal.target_revision_after
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSilencePosture {
    Healthy,
    Delayed,
    Silent,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongRunningProcessStatus {
    pub process_ref: String,
    pub scope: TemporalScope,
    pub started_at: DateTime<Utc>,
    pub elapsed_lower_ms: u64,
    pub elapsed_upper_ms: Option<u64>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub heartbeat_interval_ms: u64,
    pub silence_posture: ProcessSilencePosture,
    pub timeout_at: Option<DateTime<Utc>>,
    pub cancellable: bool,
    pub cancellation_receipt_ref: Option<String>,
    pub cleanup_receipt_refs: Vec<String>,
    pub partial_result_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatusError {
    MissingElapsedBound,
    MissingHeartbeatPolicy,
    CancelledWithoutReceipt,
    MissingCleanup,
}

pub fn validate_process_status(
    status: &LongRunningProcessStatus,
) -> Result<(), ProcessStatusError> {
    if status
        .elapsed_upper_ms
        .is_some_and(|upper| upper < status.elapsed_lower_ms)
    {
        return Err(ProcessStatusError::MissingElapsedBound);
    }
    if status.heartbeat_interval_ms == 0 {
        return Err(ProcessStatusError::MissingHeartbeatPolicy);
    }
    if status.silence_posture == ProcessSilencePosture::Cancelled
        && status.cancellation_receipt_ref.is_none()
    {
        return Err(ProcessStatusError::CancelledWithoutReceipt);
    }
    if matches!(
        status.silence_posture,
        ProcessSilencePosture::TimedOut | ProcessSilencePosture::Cancelled
    ) && status.cleanup_receipt_refs.is_empty()
    {
        return Err(ProcessStatusError::MissingCleanup);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalPulsePolicy {
    pub policy_id: String,
    pub minimum_dwell_ms: u64,
    pub debounce_ms: u64,
    pub hysteresis_ms: u64,
    pub maximum_notifications_per_hour: u32,
    pub maximum_pending_notifications: u32,
    pub protected_focus: bool,
    pub safety_authority_immutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TemporalPulseState {
    pub last_transition_at: Option<DateTime<Utc>>,
    pub last_notification_at: Option<DateTime<Utc>>,
    pub notifications_this_hour: u32,
    pub pending_notifications: u32,
    pub urgency_level: u8,
    pub backpressure_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PulseDecision {
    RecomputeSilently,
    NotifyCalmly,
    HoldForDwell,
    SuppressForBudget,
    SuppressForBackpressure,
}

pub fn temporal_pulse_decision(
    policy: &TemporalPulsePolicy,
    state: &TemporalPulseState,
    now: DateTime<Utc>,
) -> PulseDecision {
    if state.backpressure_active
        || state.pending_notifications >= policy.maximum_pending_notifications
    {
        return PulseDecision::SuppressForBackpressure;
    }
    if state.notifications_this_hour >= policy.maximum_notifications_per_hour {
        return PulseDecision::SuppressForBudget;
    }
    if state.last_transition_at.is_some_and(|last| {
        now.signed_duration_since(last).num_milliseconds() < policy.minimum_dwell_ms as i64
    }) {
        return PulseDecision::HoldForDwell;
    }
    if state.urgency_level == 0 || policy.protected_focus {
        PulseDecision::RecomputeSilently
    } else {
        PulseDecision::NotifyCalmly
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityPosture {
    UnknownCounterfactual,
    Risk,
    EvidenceProvenMiss,
    SettledNoMiss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LostTimeClassification {
    Avoidable,
    External,
    OperatorWait,
    Contention,
    Uncertainty,
    Recovery,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentVerificationStatus {
    Proposed,
    Verified,
    Disputed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LostTimeIncident {
    pub incident_id: String,
    pub scope: TemporalScope,
    pub revision: u64,
    pub predecessor_ref: Option<String>,
    pub subject_ref: String,
    pub detected_at: DateTime<Utc>,
    pub interval_start: DateTime<Utc>,
    pub interval_end: DateTime<Utc>,
    pub wall_clock_lost_ms: u64,
    pub classification: LostTimeClassification,
    pub cause_code: String,
    pub action_refs: Vec<String>,
    pub progress_refs: Vec<String>,
    pub deadline_refs: Vec<String>,
    pub opportunity_risk_refs: Vec<String>,
    pub material_impact: Option<String>,
    pub detection_delay_ms: u64,
    pub recovery_action_ref: Option<String>,
    pub prevention_candidate_ref: Option<String>,
    pub verification_status: IncidentVerificationStatus,
    pub settlement_ref: Option<String>,
    pub posture: OpportunityPosture,
    pub cause_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub prediction_evaluation_refs: Vec<String>,
    pub reflection_refs: Vec<String>,
    pub fixed_eval_refs: Vec<String>,
    pub learning_candidate_refs: Vec<String>,
    pub promotion_or_rejection_receipt_refs: Vec<String>,
    pub rollback_refs: Vec<String>,
    pub dispute_refs: Vec<String>,
    pub settlement_receipt_ref: Option<String>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LostTimeIncidentError {
    MissingEvidence,
    MissingPredecessor,
    MissingSubject,
    InvalidInterval,
    InvalidWallClockLoss,
    VerifiedUnknownClassification,
    MissWithoutPredictionEvaluation,
    SettlementWithoutReceipt,
}

pub fn validate_lost_time_incident(
    incident: &LostTimeIncident,
) -> Result<(), LostTimeIncidentError> {
    if incident.evidence_refs.is_empty() {
        return Err(LostTimeIncidentError::MissingEvidence);
    }
    if incident.subject_ref.trim().is_empty() || incident.cause_code.trim().is_empty() {
        return Err(LostTimeIncidentError::MissingSubject);
    }
    if incident.interval_end < incident.interval_start
        || incident.detected_at < incident.interval_end
    {
        return Err(LostTimeIncidentError::InvalidInterval);
    }
    let measured_ms = incident
        .interval_end
        .signed_duration_since(incident.interval_start)
        .num_milliseconds()
        .max(0) as u64;
    if incident.wall_clock_lost_ms != measured_ms {
        return Err(LostTimeIncidentError::InvalidWallClockLoss);
    }
    if incident.verification_status == IncidentVerificationStatus::Verified
        && incident.classification == LostTimeClassification::Unknown
    {
        return Err(LostTimeIncidentError::VerifiedUnknownClassification);
    }
    if incident.revision > 1 && incident.predecessor_ref.is_none() {
        return Err(LostTimeIncidentError::MissingPredecessor);
    }
    if incident.posture == OpportunityPosture::EvidenceProvenMiss
        && incident.prediction_evaluation_refs.is_empty()
    {
        return Err(LostTimeIncidentError::MissWithoutPredictionEvaluation);
    }
    if incident.posture == OpportunityPosture::SettledNoMiss
        && incident.settlement_receipt_ref.is_none()
    {
        return Err(LostTimeIncidentError::SettlementWithoutReceipt);
    }
    Ok(())
}

#[cfg(test)]
mod lost_time_tests {
    use super::*;

    fn incident() -> LostTimeIncident {
        let interval_start = Utc::now() - chrono::Duration::seconds(10);
        let interval_end = interval_start + chrono::Duration::seconds(10);
        LostTimeIncident {
            incident_id: "lost-time:test".into(),
            scope: TemporalScope::project("/project", "continuity"),
            revision: 1,
            predecessor_ref: None,
            subject_ref: "workpoint:test".into(),
            detected_at: interval_end,
            interval_start,
            interval_end,
            wall_clock_lost_ms: 10_000,
            classification: LostTimeClassification::External,
            cause_code: "provider_wait".into(),
            action_refs: vec!["action:test".into()],
            progress_refs: vec![],
            deadline_refs: vec![],
            opportunity_risk_refs: vec![],
            material_impact: None,
            detection_delay_ms: 0,
            recovery_action_ref: Some("retry:test".into()),
            prevention_candidate_ref: None,
            verification_status: IncidentVerificationStatus::Proposed,
            settlement_ref: None,
            posture: OpportunityPosture::UnknownCounterfactual,
            cause_refs: vec!["cause:provider".into()],
            evidence_refs: vec!["evidence:test".into()],
            prediction_evaluation_refs: vec![],
            reflection_refs: vec![],
            fixed_eval_refs: vec![],
            learning_candidate_refs: vec![],
            promotion_or_rejection_receipt_refs: vec![],
            rollback_refs: vec![],
            dispute_refs: vec![],
            settlement_receipt_ref: None,
            observed_at: interval_end,
        }
    }

    #[test]
    fn lost_time_validation_preserves_counterfactual_and_settlement_truth() {
        assert_eq!(validate_lost_time_incident(&incident()), Ok(()));

        let mut proven_miss = incident();
        proven_miss.posture = OpportunityPosture::EvidenceProvenMiss;
        assert_eq!(
            validate_lost_time_incident(&proven_miss),
            Err(LostTimeIncidentError::MissWithoutPredictionEvaluation)
        );
        proven_miss
            .prediction_evaluation_refs
            .push("prediction-evaluation:test".into());
        proven_miss.verification_status = IncidentVerificationStatus::Verified;
        assert_eq!(validate_lost_time_incident(&proven_miss), Ok(()));

        let mut settled = incident();
        settled.posture = OpportunityPosture::SettledNoMiss;
        assert_eq!(
            validate_lost_time_incident(&settled),
            Err(LostTimeIncidentError::SettlementWithoutReceipt)
        );
        settled.settlement_receipt_ref = Some("receipt:settlement".into());
        settled.settlement_ref = Some("settlement:test".into());
        assert_eq!(validate_lost_time_incident(&settled), Ok(()));
    }
}

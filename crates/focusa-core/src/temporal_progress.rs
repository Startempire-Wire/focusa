use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::temporal::TemporalScope;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LostTimeIncident {
    pub incident_id: String,
    pub scope: TemporalScope,
    pub revision: u64,
    pub predecessor_ref: Option<String>,
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
    MissWithoutPredictionEvaluation,
    SettlementWithoutReceipt,
}

pub fn validate_lost_time_incident(
    incident: &LostTimeIncident,
) -> Result<(), LostTimeIncidentError> {
    if incident.evidence_refs.is_empty() {
        return Err(LostTimeIncidentError::MissingEvidence);
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

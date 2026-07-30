use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{temporal::TemporalScope, temporal_deadline::DeadlineComparison};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanCalendarContext {
    pub context_id: String,
    pub operator_id: String,
    pub timezone: String,
    pub tzdb_version: String,
    pub availability_policy_ref: String,
    pub quiet_hours_policy_ref: String,
    pub resolved_boundary_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub private_detail_rehydrate_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalPriorityFrame {
    pub frame_id: String,
    pub scope: TemporalScope,
    pub operator_ask_digest: String,
    pub primary_objective_ref: String,
    pub approaching_deadline_refs: Vec<String>,
    pub conflict_state: DeadlineConflictState,
    pub consequence_summary: String,
    pub safer_sequence_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalExecutionGuard {
    pub guard_id: String,
    pub scope: TemporalScope,
    pub priority_frame_ref: String,
    pub authorized_action_refs: Vec<String>,
    pub deterministic_critical_path: bool,
    pub preauthorized: bool,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub policy_version: String,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalContextError {
    StaleCalendar,
    StalePriority,
    ScopeMismatch,
    AskMismatch,
    GuardMissing,
    GuardNotPreauthorized,
    ActionNotAuthorized,
}

pub fn authorize_temporal_action(
    calendar: &HumanCalendarContext,
    frame: &TemporalPriorityFrame,
    guard: Option<&TemporalExecutionGuard>,
    scope: &TemporalScope,
    ask_digest: &str,
    action_ref: &str,
    now: DateTime<Utc>,
) -> Result<(), TemporalContextError> {
    if calendar.expires_at <= now {
        return Err(TemporalContextError::StaleCalendar);
    }
    if frame.expires_at <= now {
        return Err(TemporalContextError::StalePriority);
    }
    if !frame.scope.matches_filter(scope) || !scope.matches_filter(&frame.scope) {
        return Err(TemporalContextError::ScopeMismatch);
    }
    if frame.operator_ask_digest != ask_digest {
        return Err(TemporalContextError::AskMismatch);
    }
    let guard = guard.ok_or(TemporalContextError::GuardMissing)?;
    if !guard.preauthorized || guard.expires_at <= now {
        return Err(TemporalContextError::GuardNotPreauthorized);
    }
    if !guard
        .authorized_action_refs
        .iter()
        .any(|item| item == action_ref)
    {
        return Err(TemporalContextError::ActionNotAuthorized);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildTimeoutBudget {
    pub budget_id: String,
    pub parent_budget_ref: String,
    pub original_deadline_monotonic_ns: u128,
    pub dispatched_at_monotonic_ns: u128,
    pub remaining_ns: u128,
    pub elapsed_deducted_ns: u128,
    pub retry_count: u32,
    pub cancellation_requested: bool,
    pub cancellation_acknowledged: bool,
    pub cancellation_effective: bool,
    pub possible_effect_requires_reconciliation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildBudgetError {
    ParentCapExceeded,
    ElapsedNotDeducted,
    RetryDeadlineReset,
    CancellationUnacknowledged,
    ReconciliationRequired,
}

pub fn validate_child_budget(budget: &ChildTimeoutBudget) -> Result<(), ChildBudgetError> {
    let expected = budget
        .original_deadline_monotonic_ns
        .saturating_sub(budget.dispatched_at_monotonic_ns);
    if budget.remaining_ns > expected {
        return Err(ChildBudgetError::ParentCapExceeded);
    }
    if budget.elapsed_deducted_ns == 0 && budget.dispatched_at_monotonic_ns > 0 {
        return Err(ChildBudgetError::ElapsedNotDeducted);
    }
    if budget.retry_count > 0 && budget.remaining_ns == expected {
        return Err(ChildBudgetError::RetryDeadlineReset);
    }
    if budget.cancellation_requested && !budget.cancellation_acknowledged {
        return Err(ChildBudgetError::CancellationUnacknowledged);
    }
    if budget.possible_effect_requires_reconciliation && budget.retry_count > 0 {
        return Err(ChildBudgetError::ReconciliationRequired);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlineConflictState {
    Feasible,
    Infeasible,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadlineConflictResolution {
    pub state: DeadlineConflictState,
    pub primary_objective_ref: String,
    pub non_preemptible_obligation_refs: Vec<String>,
    pub displaced_objective_refs: Vec<String>,
    pub displacement_disclosure: String,
    pub operator_escalation_required: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PressureNarrowingChecklist {
    pub safety_preserved: bool,
    pub security_preserved: bool,
    pub authority_preserved: bool,
    pub proof_preserved: bool,
    pub reconciliation_preserved: bool,
    pub disconfirming_evidence_preserved: bool,
    pub independent_review_ref: Option<String>,
    pub workload_fatigue_posture: String,
    pub fresh_reviewer_handoff_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalDisplayState {
    pub confidence_label: String,
    pub confidence_basis: String,
    pub slack_status: String,
    pub boundary_comparison: DeadlineComparison,
    pub severity_label: String,
    pub non_color_indicator: String,
    pub breach_overrides_success: bool,
    pub review_burden_ms: u64,
    pub deduplication_key: String,
    pub rate_limit_policy_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureTemporalPosture {
    pub factual_status: String,
    pub operator_disposition: Option<String>,
    pub amendment_ref: Option<String>,
    pub degraded_posture: Option<String>,
    pub rollup_eligible: bool,
    pub temporal_failure: bool,
    pub spec131_closure_ref: String,
    pub spec137_temporal_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosurePostureError {
    NonCompletionMasqueradesAsCompletion,
    MissingSpec131Authority,
    TemporalFailureHidden,
    RollupWithoutClosure,
}

pub fn validate_closure_posture(
    posture: &ClosureTemporalPosture,
) -> Result<(), ClosurePostureError> {
    if [
        "cancelled",
        "accepted_risk",
        "variance",
        "abandoned",
        "scope_amended",
    ]
    .contains(&posture.factual_status.as_str())
        && posture.rollup_eligible
    {
        return Err(ClosurePostureError::NonCompletionMasqueradesAsCompletion);
    }
    if posture.spec131_closure_ref.trim().is_empty() {
        return Err(ClosurePostureError::MissingSpec131Authority);
    }
    if posture.temporal_failure && posture.spec137_temporal_refs.is_empty() {
        return Err(ClosurePostureError::TemporalFailureHidden);
    }
    if posture.rollup_eligible && posture.factual_status != "completed" {
        return Err(ClosurePostureError::RollupWithoutClosure);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyActivationContract {
    pub dependency_name: String,
    pub source_commit: String,
    pub document_hash: String,
    pub schema_ref: String,
    pub ownership_ref: String,
    pub migration_ref: String,
    pub conformance_ref: String,
    pub approval_receipt_ref: String,
    pub immutable: bool,
}

pub fn dependency_is_activated(contract: &DependencyActivationContract) -> bool {
    contract.immutable
        && [
            &contract.source_commit,
            &contract.document_hash,
            &contract.schema_ref,
            &contract.ownership_ref,
            &contract.migration_ref,
            &contract.conformance_ref,
            &contract.approval_receipt_ref,
        ]
        .iter()
        .all(|value| !value.trim().is_empty())
}

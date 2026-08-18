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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn test_scope() -> TemporalScope {
        TemporalScope {
            project_root: "/tmp/test".into(),
            continuity_id: "test".into(),
            host_id: None,
            operator_id: None,
            workpoint_id: None,
            item_id: None,
            task_id: None,
        }
    }

    fn test_calendar(expires_at: DateTime<Utc>) -> HumanCalendarContext {
        HumanCalendarContext {
            context_id: "cal-1".into(),
            operator_id: "op-1".into(),
            timezone: "America/Los_Angeles".into(),
            tzdb_version: "2024a".into(),
            availability_policy_ref: "avail-1".into(),
            quiet_hours_policy_ref: "quiet-1".into(),
            resolved_boundary_refs: vec![],
            generated_at: chrono::Utc::now(),
            expires_at,
            private_detail_rehydrate_refs: vec![],
        }
    }

    fn test_frame(expires_at: DateTime<Utc>) -> TemporalPriorityFrame {
        TemporalPriorityFrame {
            frame_id: "f-1".into(),
            scope: test_scope(),
            operator_ask_digest: "ask-1".into(),
            primary_objective_ref: "obj-1".into(),
            approaching_deadline_refs: vec![],
            conflict_state: DeadlineConflictState::Feasible,
            consequence_summary: "test".into(),
            safer_sequence_refs: vec![],
            generated_at: chrono::Utc::now(),
            expires_at,
            evidence_refs: vec![],
        }
    }

    #[test]
    fn authorize_rejects_stale_calendar() {
        let now = chrono::Utc::now();
        let calendar = test_calendar(now - Duration::hours(1));
        let frame = test_frame(now + Duration::hours(1));
        let guard = TemporalExecutionGuard {
            guard_id: "g-1".into(),
            scope: test_scope(),
            priority_frame_ref: "f-1".into(),
            authorized_action_refs: vec!["a-1".into()],
            deterministic_critical_path: false,
            preauthorized: true,
            issued_at: now,
            expires_at: now + Duration::hours(1),
            policy_version: "v1".into(),
            receipt_ref: "rec-1".into(),
        };
        let result = authorize_temporal_action(
            &calendar,
            &frame,
            Some(&guard),
            &test_scope(),
            "ask-1",
            "a-1",
            now,
        );
        assert!(matches!(result, Err(TemporalContextError::StaleCalendar)));
    }

    #[test]
    fn authorize_rejects_guard_missing() {
        let now = chrono::Utc::now();
        let result = authorize_temporal_action(
            &test_calendar(now + Duration::hours(1)),
            &test_frame(now + Duration::hours(1)),
            None,
            &test_scope(),
            "ask-1",
            "a-1",
            now,
        );
        assert!(matches!(result, Err(TemporalContextError::GuardMissing)));
    }

    #[test]
    fn authorize_rejects_ask_mismatch() {
        let now = chrono::Utc::now();
        let guard = TemporalExecutionGuard {
            guard_id: "g-1".into(),
            scope: test_scope(),
            priority_frame_ref: "f-1".into(),
            authorized_action_refs: vec!["a-1".into()],
            deterministic_critical_path: false,
            preauthorized: true,
            issued_at: now,
            expires_at: now + Duration::hours(1),
            policy_version: "v1".into(),
            receipt_ref: "rec-1".into(),
        };
        let result = authorize_temporal_action(
            &test_calendar(now + Duration::hours(1)),
            &test_frame(now + Duration::hours(1)),
            Some(&guard),
            &test_scope(),
            "wrong-ask",
            "a-1",
            now,
        );
        assert!(matches!(result, Err(TemporalContextError::AskMismatch)));
    }

    #[test]
    fn authorize_rejects_unpreauthorized_guard() {
        let now = chrono::Utc::now();
        let guard = TemporalExecutionGuard {
            guard_id: "g-1".into(),
            scope: test_scope(),
            priority_frame_ref: "f-1".into(),
            authorized_action_refs: vec!["a-1".into()],
            deterministic_critical_path: false,
            preauthorized: false,
            issued_at: now,
            expires_at: now + Duration::hours(1),
            policy_version: "v1".into(),
            receipt_ref: "rec-1".into(),
        };
        let result = authorize_temporal_action(
            &test_calendar(now + Duration::hours(1)),
            &test_frame(now + Duration::hours(1)),
            Some(&guard),
            &test_scope(),
            "ask-1",
            "a-1",
            now,
        );
        assert!(matches!(
            result,
            Err(TemporalContextError::GuardNotPreauthorized)
        ));
    }

    #[test]
    fn authorize_rejects_action_not_listed() {
        let now = chrono::Utc::now();
        let guard = TemporalExecutionGuard {
            guard_id: "g-1".into(),
            scope: test_scope(),
            priority_frame_ref: "f-1".into(),
            authorized_action_refs: vec!["a-1".into()],
            deterministic_critical_path: false,
            preauthorized: true,
            issued_at: now,
            expires_at: now + Duration::hours(1),
            policy_version: "v1".into(),
            receipt_ref: "rec-1".into(),
        };
        let result = authorize_temporal_action(
            &test_calendar(now + Duration::hours(1)),
            &test_frame(now + Duration::hours(1)),
            Some(&guard),
            &test_scope(),
            "ask-1",
            "a-2",
            now,
        );
        assert!(matches!(
            result,
            Err(TemporalContextError::ActionNotAuthorized)
        ));
    }

    #[test]
    fn validate_child_budget_rejects_parent_cap_exceeded() {
        let budget = ChildTimeoutBudget {
            budget_id: "b-1".into(),
            parent_budget_ref: "p-1".into(),
            original_deadline_monotonic_ns: 1000,
            dispatched_at_monotonic_ns: 500,
            remaining_ns: 600,
            elapsed_deducted_ns: 300,
            retry_count: 0,
            cancellation_requested: false,
            cancellation_acknowledged: false,
            cancellation_effective: false,
            possible_effect_requires_reconciliation: false,
        };
        assert!(matches!(
            validate_child_budget(&budget),
            Err(ChildBudgetError::ParentCapExceeded)
        ));
    }

    #[test]
    fn validate_child_budget_passes_valid() {
        let budget = ChildTimeoutBudget {
            budget_id: "b-1".into(),
            parent_budget_ref: "p-1".into(),
            original_deadline_monotonic_ns: 1000,
            dispatched_at_monotonic_ns: 500,
            remaining_ns: 400,
            elapsed_deducted_ns: 100,
            retry_count: 0,
            cancellation_requested: false,
            cancellation_acknowledged: false,
            cancellation_effective: false,
            possible_effect_requires_reconciliation: false,
        };
        assert!(validate_child_budget(&budget).is_ok());
    }

    #[test]
    fn validate_closure_rejects_non_completion_rollup() {
        let posture = ClosureTemporalPosture {
            factual_status: "cancelled".into(),
            operator_disposition: None,
            amendment_ref: None,
            degraded_posture: None,
            rollup_eligible: true,
            temporal_failure: false,
            spec131_closure_ref: "spec131-ref".into(),
            spec137_temporal_refs: vec![],
            receipt_refs: vec![],
        };
        assert!(matches!(
            validate_closure_posture(&posture),
            Err(ClosurePostureError::NonCompletionMasqueradesAsCompletion)
        ));
    }

    #[test]
    fn validate_closure_rejects_empty_spec131_ref() {
        let posture = ClosureTemporalPosture {
            factual_status: "completed".into(),
            operator_disposition: None,
            amendment_ref: None,
            degraded_posture: None,
            rollup_eligible: false,
            temporal_failure: false,
            spec131_closure_ref: "  ".into(),
            spec137_temporal_refs: vec![],
            receipt_refs: vec![],
        };
        assert!(matches!(
            validate_closure_posture(&posture),
            Err(ClosurePostureError::MissingSpec131Authority)
        ));
    }

    #[test]
    fn dependency_activation_requires_all_fields() {
        let contract = DependencyActivationContract {
            dependency_name: "spec136".into(),
            source_commit: "abc123".into(),
            document_hash: "def456".into(),
            schema_ref: "schema".into(),
            ownership_ref: "owner".into(),
            migration_ref: "migrate".into(),
            conformance_ref: "conform".into(),
            approval_receipt_ref: "rec".into(),
            immutable: true,
        };
        assert!(dependency_is_activated(&contract));
    }

    #[test]
    fn dependency_activation_rejects_empty_fields() {
        let contract = DependencyActivationContract {
            dependency_name: "spec136".into(),
            source_commit: "  ".into(),
            document_hash: "def456".into(),
            schema_ref: "schema".into(),
            ownership_ref: "owner".into(),
            migration_ref: "migrate".into(),
            conformance_ref: "conform".into(),
            approval_receipt_ref: "rec".into(),
            immutable: true,
        };
        assert!(!dependency_is_activated(&contract));
    }
}

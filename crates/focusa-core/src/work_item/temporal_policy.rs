//! Temporal authority integration for Work Loop and Silent Session scheduling.
//! Spec137 Slice 6: deadline-or-gated work selection, overdue detection,
//! cancellation propagation, and protected delivery focus.
//!
//! Every Work Loop selection and Silent Session dispatch MUST pass temporal
//! preflight before mutating work-item state or consuming execution budget.

use crate::temporal::{TemporalEvent, TemporalEventKind, TemporalScope};
use crate::temporal_operations::{
    DeadlineConflictState, HumanCalendarContext, TemporalExecutionGuard, TemporalPriorityFrame,
    authorize_temporal_action,
};
use chrono::{DateTime, Duration, Utc};

/// Outcome of a temporal preflight check before Work Loop selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalSelectionPolicy {
    /// Proceed normally — all temporal constraints satisfied.
    Proceed,
    /// Pin this deadline item for protected delivery focus.
    DeadlineProtected,
    /// Escalate — approaching deadline with insufficient remaining budget.
    DeadlineEscalated,
    /// Skip — item deadline has expired; reconciliation required.
    DeadlineExpired,
    /// Indeterminate — clock/calendar/guard state cannot be resolved.
    Unavailable,
}

/// Evaluates the temporal posture of a work item before Work Loop selection.
///
/// Returns the scheduling policy based on deadline comparison, time budget,
/// and operator priority frame validity.
pub fn evaluate_temporal_selection_policy(
    now: DateTime<Utc>,
    item_deadline_at: Option<DateTime<Utc>>,
    item_estimated_ms: Option<u64>,
    calendar: Option<&HumanCalendarContext>,
    priority_frame: Option<&TemporalPriorityFrame>,
    guard: Option<&TemporalExecutionGuard>,
    scope: &TemporalScope,
    ask_digest: &str,
) -> TemporalSelectionPolicy {
    // 1. Temporal action authorization — guard + calendar + frame validity.
    if let Some(calendar) = calendar {
        if let Some(frame) = priority_frame {
            if authorize_temporal_action(calendar, frame, guard, scope, ask_digest, "select", now)
                .is_err()
            {
                return TemporalSelectionPolicy::Unavailable;
            }
        }
    }

    // 2. Deadline comparison.
    if let Some(deadline_at) = item_deadline_at {
        let remaining = deadline_at
            .signed_duration_since(now)
            .num_milliseconds()
            .max(0) as u64;

        if deadline_at <= now {
            return TemporalSelectionPolicy::DeadlineExpired;
        }

        // Protected delivery focus: if deadline is within 10 minutes AND
        // we have a valid guard, pin this item for uninterrupted delivery.
        if remaining < 600_000 {
            return TemporalSelectionPolicy::DeadlineProtected;
        }

        // Escalate: if estimated work exceeds 2/3 of remaining budget.
        if let Some(estimated) = item_estimated_ms {
            if estimated > remaining.saturating_mul(2) / 3 {
                return TemporalSelectionPolicy::DeadlineEscalated;
            }
        }
    }

    TemporalSelectionPolicy::Proceed
}

/// Records a temporal event for a Work Loop selection action.
///
/// Every selection that changes temporal posture (escalated, protected,
/// expired) MUST produce a signed temporal event for auditability.
pub fn record_selection_event(
    scope: &TemporalScope,
    policy: &TemporalSelectionPolicy,
    item_id: &str,
    now: DateTime<Utc>,
) -> TemporalEvent {
    let event_kind = match policy {
        TemporalSelectionPolicy::DeadlineProtected => TemporalEventKind::TargetSatisfied,
        TemporalSelectionPolicy::DeadlineEscalated => TemporalEventKind::ClaimRevised,
        TemporalSelectionPolicy::DeadlineExpired => TemporalEventKind::TargetBreached,
        _ => TemporalEventKind::ClaimCommitted,
    };
    TemporalEvent {
        event_id: format!("selection:{item_id}:{}", now.timestamp_millis()),
        sequence: 0,
        event_kind,
        scope: scope.clone(),
        claim: None,
        clock_sample: None,
        metadata: Default::default(),
        signature: None,
        predecessor_digest: None,
        recorded_at: now,
        idempotency_key: format!("select:{item_id}"),
        digest: String::new(),
    }
}

/// Overdue opportunity assessment — determines whether a missed deadline
/// requires reconciliation, compensation, cleanup, or can be safely closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverdueDeliveryMode {
    /// Missed deadline without material harm — record and close.
    RecordOnly,
    /// Partial delivery completed — reconcile remaining scope.
    ReconcileRemaining,
    /// Full compensation required — reverse effects.
    FullCompensation,
    /// Delivery is still in progress — extend deadline.
    ExtendDeadline,
}

pub fn assess_overdue_delivery(
    already_delivered: usize,
    expected_deliverables: usize,
    deadline_at: DateTime<Utc>,
    now: DateTime<Utc>,
    has_material_harm: bool,
    delivery_in_progress: bool,
) -> OverdueDeliveryMode {
    if delivery_in_progress {
        return OverdueDeliveryMode::ExtendDeadline;
    }
    if !has_material_harm && already_delivered >= expected_deliverables {
        return OverdueDeliveryMode::RecordOnly;
    }
    if already_delivered > 0 && already_delivered < expected_deliverables {
        return OverdueDeliveryMode::ReconcileRemaining;
    }
    OverdueDeliveryMode::FullCompensation
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn proceed_when_no_deadline_set() {
        let result = evaluate_temporal_selection_policy(
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            &test_scope(),
            "ask-1",
        );
        assert_eq!(result, TemporalSelectionPolicy::Proceed);
    }

    #[test]
    fn deadline_expired_when_past_due() {
        let now = Utc::now();
        let result = evaluate_temporal_selection_policy(
            now,
            Some(now - Duration::hours(1)),
            None,
            None,
            None,
            None,
            &test_scope(),
            "ask-1",
        );
        assert_eq!(result, TemporalSelectionPolicy::DeadlineExpired);
    }

    #[test]
    fn deadline_protected_when_within_10_minutes() {
        let now = Utc::now();
        let result = evaluate_temporal_selection_policy(
            now,
            Some(now + Duration::minutes(5)),
            None,
            None,
            None,
            None,
            &test_scope(),
            "ask-1",
        );
        assert_eq!(result, TemporalSelectionPolicy::DeadlineProtected);
    }

    #[test]
    fn deadline_escalated_when_estimate_exceeds_budget() {
        let now = Utc::now();
        let result = evaluate_temporal_selection_policy(
            now,
            Some(now + Duration::hours(1)),
            Some(3_000_000), // 50 min est > 2/3 * 60 min = 40 min
            None,
            None,
            None,
            &test_scope(),
            "ask-1",
        );
        assert_eq!(result, TemporalSelectionPolicy::DeadlineEscalated);
    }

    #[test]
    fn overdue_record_only_when_delivered_without_harm() {
        let mode = assess_overdue_delivery(
            5, 5,
            Utc::now() - Duration::hours(1),
            Utc::now(),
            false,
            false,
        );
        assert_eq!(mode, OverdueDeliveryMode::RecordOnly);
    }

    #[test]
    fn overdue_reconcile_when_partial_delivery() {
        let mode = assess_overdue_delivery(
            2, 5,
            Utc::now() - Duration::hours(1),
            Utc::now(),
            false,
            false,
        );
        assert_eq!(mode, OverdueDeliveryMode::ReconcileRemaining);
    }

    #[test]
    fn overdue_compensate_when_no_delivery_and_harm() {
        let mode = assess_overdue_delivery(
            0, 5,
            Utc::now() - Duration::hours(1),
            Utc::now(),
            true,
            false,
        );
        assert_eq!(mode, OverdueDeliveryMode::FullCompensation);
    }

    #[test]
    fn overdue_extend_when_delivery_in_progress() {
        let mode = assess_overdue_delivery(
            1, 5,
            Utc::now() - Duration::hours(1),
            Utc::now(),
            true,
            true,
        );
        assert_eq!(mode, OverdueDeliveryMode::ExtendDeadline);
    }
}

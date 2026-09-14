//! Spec137 Slice 8: temporal closure validation integrated with Spec116/119 Receipts.
//! No closure receipt may be issued without temporal posture verification.

use crate::temporal_deadline::{DeadlineComparison, compare_deadline};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Temporal closure posture — recorded alongside every Spec119 Receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalClosureReceipt {
    pub receipt_id: String,
    pub closure_timestamp: DateTime<Utc>,
    pub temporal_status: TemporalClosureStatus,
    pub deadline_comparisons: Vec<DeadlineComparisonRecord>,
    pub undelivered_deadline_refs: Vec<String>,
    pub overdue_delivery_count: usize,
    pub temporal_failure_recovery: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClosureStatus {
    /// All deadlines satisfied; normal closure.
    TemporalComplete,
    /// Closure completed but some deadlines were breached.
    TemporalBreached,
    /// Closure completed late — deadlines exceeded but delivery finished.
    TemporalLateComplete,
    /// Closure requested but temporal state is indeterminate.
    TemporalIndeterminate,
    /// Closure blocked — at least one undelivered protected deadline.
    TemporalBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadlineComparisonRecord {
    pub deadline_ref: String,
    pub comparison: String,
    pub observed_earliest: Option<DateTime<Utc>>,
    pub observed_latest: Option<DateTime<Utc>>,
}

/// Evaluates the temporal closure posture for a set of deadlines.
/// Returns the worst-case status and a list of breached/undelivered deadlines.
pub fn evaluate_temporal_closure(
    deadline_records: &[DeadlineComparisonRecord],
    now: DateTime<Utc>,
) -> (TemporalClosureStatus, Vec<String>) {
    let mut undelivered = Vec::new();
    let mut breached = false;
    let mut late = false;
    let mut indeterminate = false;

    for record in deadline_records {
        match record.comparison.as_str() {
            "on_time" => {}
            "late_window" => {
                late = true;
            }
            "breached" => {
                breached = true;
                undelivered.push(record.deadline_ref.clone());
            }
            "indeterminate" | "possibly_crossed" => {
                indeterminate = true;
                undelivered.push(record.deadline_ref.clone());
            }
            _ => {}
        }
    }

    let status = if breached {
        TemporalClosureStatus::TemporalBlocked
    } else if indeterminate && undelivered.len() > 1 {
        TemporalClosureStatus::TemporalIndeterminate
    } else if breached && late {
        TemporalClosureStatus::TemporalBreached
    } else if late {
        TemporalClosureStatus::TemporalLateComplete
    } else {
        TemporalClosureStatus::TemporalComplete
    };

    (status, undelivered)
}

/// Builds a temporal closure receipt compatible with Spec119 Receipt format.
pub fn build_temporal_closure_receipt(
    receipt_id: impl Into<String>,
    deadline_records: Vec<DeadlineComparisonRecord>,
    recovery_plan: Option<String>,
) -> TemporalClosureReceipt {
    let now = Utc::now();
    let (status, undelivered) = evaluate_temporal_closure(&deadline_records, now);
    let recovery = if status != TemporalClosureStatus::TemporalComplete {
        recovery_plan
    } else {
        None
    };
    TemporalClosureReceipt {
        receipt_id: receipt_id.into(),
        closure_timestamp: now,
        temporal_status: status,
        overdue_delivery_count: undelivered.len(),
        undelivered_deadline_refs: undelivered,
        deadline_comparisons: deadline_records,
        temporal_failure_recovery: recovery,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_complete_when_all_on_time() {
        let records = vec![DeadlineComparisonRecord {
            deadline_ref: "d1".into(),
            comparison: "on_time".into(),
            observed_earliest: None,
            observed_latest: None,
        }];
        let (status, undelivered) = evaluate_temporal_closure(&records, Utc::now());
        assert_eq!(status, TemporalClosureStatus::TemporalComplete);
        assert!(undelivered.is_empty());
    }

    #[test]
    fn temporal_blocked_when_breached() {
        let records = vec![DeadlineComparisonRecord {
            deadline_ref: "d1".into(),
            comparison: "breached".into(),
            observed_earliest: Some(Utc::now()),
            observed_latest: None,
        }];
        let (status, undelivered) = evaluate_temporal_closure(&records, Utc::now());
        assert_eq!(status, TemporalClosureStatus::TemporalBlocked);
        assert_eq!(undelivered, vec!["d1"]);
    }

    #[test]
    fn late_complete_when_in_late_window() {
        let records = vec![DeadlineComparisonRecord {
            deadline_ref: "d1".into(),
            comparison: "late_window".into(),
            observed_earliest: None,
            observed_latest: None,
        }];
        let (status, _) = evaluate_temporal_closure(&records, Utc::now());
        assert_eq!(status, TemporalClosureStatus::TemporalLateComplete);
    }

    #[test]
    fn temporal_indeterminate_when_possibly_crossed_multiple() {
        let records = vec![
            DeadlineComparisonRecord {
                deadline_ref: "d1".into(),
                comparison: "possibly_crossed".into(),
                observed_earliest: None,
                observed_latest: None,
            },
            DeadlineComparisonRecord {
                deadline_ref: "d2".into(),
                comparison: "indeterminate".into(),
                observed_earliest: None,
                observed_latest: None,
            },
        ];
        let (status, undelivered) = evaluate_temporal_closure(&records, Utc::now());
        assert_eq!(status, TemporalClosureStatus::TemporalIndeterminate);
        assert_eq!(undelivered.len(), 2);
    }

    #[test]
    fn build_receipt_binds_temporal_status() {
        let records = vec![DeadlineComparisonRecord {
            deadline_ref: "d1".into(),
            comparison: "on_time".into(),
            observed_earliest: None,
            observed_latest: None,
        }];
        let receipt =
            build_temporal_closure_receipt("rec-1", records, Some("retry delivery".into()));
        assert_eq!(receipt.receipt_id, "rec-1");
        assert_eq!(
            receipt.temporal_status,
            TemporalClosureStatus::TemporalComplete
        );
        assert!(receipt.temporal_failure_recovery.is_none());
    }
}

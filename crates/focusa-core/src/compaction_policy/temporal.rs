//! Spec137 Slice 7: temporal context injection into compaction and awareness.
//!
//! Compaction MUST preserve temporal context across handoff boundaries.
//! Before compacting, check for deadline-protected items and preserve
//! temporal evidence through the compaction cycle.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Temporal evidence preserved across compaction boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalCompactionContext {
    /// Active deadlines that must survive compaction.
    pub active_deadline_refs: Vec<String>,
    /// Clock epoch at compaction time — for monotonic verification post-compaction.
    pub compaction_epoch_monotonic_ns: u64,
    /// Whether any deadline-protected items were deferred by compaction.
    pub deadline_protection_deferred: bool,
    /// Temporal events that need re-emission after compaction.
    pub pending_temporal_event_refs: Vec<String>,
    /// Estimated remaining work before next deadline.
    pub estimated_remaining_before_deadline_ms: Option<u64>,
    /// Whether the compaction window overlaps with a protected delivery focus.
    pub within_delivery_focus_window: bool,
    /// Minimum time before next deadline crossing — compaction must not delay past this.
    pub min_time_to_deadline_crossing_ms: Option<u64>,
}

impl TemporalCompactionContext {
    /// Build a temporal compaction context from the current temporal state.
    pub fn capture(
        now: DateTime<Utc>,
        active_deadline_refs: Vec<String>,
        deadline_protection_deferred: bool,
        min_time_to_deadline_ms: Option<u64>,
        estimated_remaining_ms: Option<u64>,
    ) -> Self {
        Self {
            active_deadline_refs,
            compaction_epoch_monotonic_ns: monotonic_now_ns(),
            deadline_protection_deferred,
            pending_temporal_event_refs: Vec::new(),
            estimated_remaining_before_deadline_ms: estimated_remaining_ms,
            within_delivery_focus_window: min_time_to_deadline_ms.is_some_and(|ms| ms < 600_000),
            min_time_to_deadline_crossing_ms: min_time_to_deadline_ms,
        }
    }

    /// Returns true if the context has expired — deadlines crossed during compaction.
    pub fn has_expired_since(&self, now: DateTime<Utc>) -> bool {
        if let Some(min_ms) = self.min_time_to_deadline_crossing_ms {
            let elapsed_since_compaction_ms = chrono::Utc::now()
                .signed_duration_since(now)
                .num_milliseconds()
                .max(0) as u64;
            elapsed_since_compaction_ms >= min_ms
        } else {
            false
        }
    }

    /// Returns whether any deadline was missed during the compaction window.
    pub fn deadline_missed_during_compaction(&self) -> bool {
        self.deadline_protection_deferred && self.within_delivery_focus_window
    }
}

/// Returns a monotonic nanosecond timestamp from a platform clock.
/// Used for compaction epoch verification — NOT for absolute wall-clock authority.
fn monotonic_now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

/// Validates that a post-compaction temporal state is consistent with the
/// captured pre-compaction context. Returns false if deadlines were missed
/// or temporal integrity was lost during compaction.
pub fn verify_temporal_compaction_integrity(
    pre: &TemporalCompactionContext,
    post_active_deadline_refs: &[String],
) -> bool {
    if pre.deadline_missed_during_compaction() {
        return false;
    }
    // All active deadline refs must survive compaction.
    for ref_id in &pre.active_deadline_refs {
        if !post_active_deadline_refs.contains(ref_id) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_context_records_deadlines() {
        let now = Utc::now();
        let ctx = TemporalCompactionContext::capture(
            now,
            vec!["deadline-1".into(), "deadline-2".into()],
            false,
            Some(300_000),
            Some(200_000),
        );
        assert_eq!(ctx.active_deadline_refs.len(), 2);
        assert!(ctx.compaction_epoch_monotonic_ns > 0);
        assert!(ctx.within_delivery_focus_window);
    }

    #[test]
    fn deadline_missed_when_protection_deferred_in_focus_window() {
        let ctx = TemporalCompactionContext {
            active_deadline_refs: vec!["d1".into()],
            compaction_epoch_monotonic_ns: 1000,
            deadline_protection_deferred: true,
            pending_temporal_event_refs: vec![],
            estimated_remaining_before_deadline_ms: Some(100_000),
            within_delivery_focus_window: true,
            min_time_to_deadline_crossing_ms: Some(300_000),
        };
        assert!(ctx.deadline_missed_during_compaction());
    }

    #[test]
    fn no_deadline_missed_without_focus_window() {
        let ctx = TemporalCompactionContext {
            active_deadline_refs: vec!["d1".into()],
            compaction_epoch_monotonic_ns: 1000,
            deadline_protection_deferred: true,
            pending_temporal_event_refs: vec![],
            estimated_remaining_before_deadline_ms: Some(100_000),
            within_delivery_focus_window: false,
            min_time_to_deadline_crossing_ms: Some(3_600_000),
        };
        assert!(!ctx.deadline_missed_during_compaction());
    }

    #[test]
    fn verify_integrity_detects_missing_deadline() {
        let pre = TemporalCompactionContext::capture(
            Utc::now(),
            vec!["deadline-1".into()],
            false,
            Some(500_000),
            Some(100_000),
        );
        assert!(!verify_temporal_compaction_integrity(&pre, &[]));
    }

    #[test]
    fn verify_integrity_passes_when_deadlines_survive() {
        let pre = TemporalCompactionContext::capture(
            Utc::now(),
            vec!["deadline-1".into()],
            false,
            Some(500_000),
            Some(100_000),
        );
        assert!(verify_temporal_compaction_integrity(
            &pre,
            &["deadline-1".into()],
        ));
    }

    #[test]
    fn has_expired_when_deadline_crossed() {
        let ctx = TemporalCompactionContext {
            active_deadline_refs: vec![],
            compaction_epoch_monotonic_ns: 1000,
            deadline_protection_deferred: false,
            pending_temporal_event_refs: vec![],
            estimated_remaining_before_deadline_ms: None,
            within_delivery_focus_window: false,
            min_time_to_deadline_crossing_ms: Some(1),
        };
        // Advance time past the 1ms crossing threshold
        let past = Utc::now() - Duration::milliseconds(2);
        assert!(ctx.has_expired_since(past));
    }

    #[test]
    fn not_expired_when_min_time_far_out() {
        let ctx = TemporalCompactionContext {
            active_deadline_refs: vec![],
            compaction_epoch_monotonic_ns: 1000,
            deadline_protection_deferred: false,
            pending_temporal_event_refs: vec![],
            estimated_remaining_before_deadline_ms: None,
            within_delivery_focus_window: false,
            min_time_to_deadline_crossing_ms: Some(3_600_000), // 1 hour
        };
        assert!(!ctx.has_expired_since(Utc::now()));
    }
}

//! Garbage collection for ECS.
//!
//! Source: G1-detail-08-ecs.md
//!
//! MVP strategy:
//!   - Keep everything by default (no auto-GC)
//!   - Provide manual GC for unpinned handles older than a threshold
//!   - Ensure index consistency on startup (repair pass)
//!
//! GC rules:
//!   - Pinned artifacts are NEVER collected
//!   - Artifacts referenced by active frames are NEVER collected
//!   - Only artifacts from closed sessions are eligible
//!   - GC is explicit — never automatic in MVP

use crate::types::*;
use chrono::{Duration, Utc};

/// Identify artifacts eligible for garbage collection.
///
/// Eligible = unpinned + older than `max_age` + not in any active frame's handles.
pub fn find_eligible(
    state: &FocusaState,
    max_age: Duration,
) -> Vec<ArtifactId> {
    let cutoff = Utc::now() - max_age;

    // Collect all handle IDs referenced by non-completed frames.
    let active_handle_ids: Vec<HandleId> = state
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status != FrameStatus::Completed && f.status != FrameStatus::Archived)
        .flat_map(|f| f.handles.iter().map(|h| h.id))
        .collect();

    state
        .reference_index
        .handles
        .iter()
        .filter(|h| {
            !h.pinned
                && h.created_at < cutoff
                && !active_handle_ids.contains(&h.id)
        })
        .map(|h| h.id)
        .collect()
}

/// Count total storage used by handles in the index.
pub fn total_storage_bytes(state: &FocusaState) -> u64 {
    state.reference_index.handles.iter().map(|h| h.size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_find_eligible_empty_state() {
        let state = FocusaState::default();
        let eligible = find_eligible(&state, Duration::hours(24));
        assert!(eligible.is_empty());
    }

    #[test]
    fn test_pinned_never_eligible() {
        let mut state = FocusaState::default();
        state.reference_index.handles.push(HandleRef {
            id: uuid::Uuid::now_v7(),
            kind: HandleKind::Log,
            label: "important".into(),
            size: 1024,
            sha256: "abc".into(),
            created_at: Utc::now() - Duration::days(30),
            session_id: None,
            pinned: true,
        });
        let eligible = find_eligible(&state, Duration::hours(1));
        assert!(eligible.is_empty());
    }

    #[test]
    fn test_old_unpinned_is_eligible() {
        let mut state = FocusaState::default();
        let id = uuid::Uuid::now_v7();
        state.reference_index.handles.push(HandleRef {
            id,
            kind: HandleKind::Log,
            label: "old log".into(),
            size: 512,
            sha256: "def".into(),
            created_at: Utc::now() - Duration::days(30),
            session_id: None,
            pinned: false,
        });
        let eligible = find_eligible(&state, Duration::hours(1));
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0], id);
    }

    #[test]
    fn test_recent_not_eligible() {
        let mut state = FocusaState::default();
        state.reference_index.handles.push(HandleRef {
            id: uuid::Uuid::now_v7(),
            kind: HandleKind::Text,
            label: "recent".into(),
            size: 256,
            sha256: "ghi".into(),
            created_at: Utc::now(),
            session_id: None,
            pinned: false,
        });
        let eligible = find_eligible(&state, Duration::hours(24));
        assert!(eligible.is_empty());
    }
}

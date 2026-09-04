//! Reference Store / ECS — externalized context store.
//!
//! Source: 07-reference-store.md, G1-detail-08-ecs.md
//!
//! INVARIANT: Artifacts are never implicitly injected.
//! INVARIANT: Artifacts are referenced by handles only.
//! INVARIANT: Artifacts are immutable once written.
//! INVARIANT: Rehydration is explicit and auditable.
//! INVARIANT: Storage is session-scoped by default.

pub mod artifact;
pub mod gc;
pub mod store;

use crate::types::{HandleRef, ReferenceIndex, SessionId};

/// Default number of complete handle records retained in the hot FocusaState projection.
/// Canonical content and metadata remain losslessly addressable in the ECS object store.
pub const DEFAULT_HOT_HANDLE_LIMIT: usize = 2_048;
pub const MAX_HOT_HANDLE_LIMIT: usize = 16_384;

/// Bound the in-memory/snapshot handle projection without deleting durable ECS artifacts.
///
/// Pinned and active-session handles are preferred, then the newest remaining handles.
/// The cap remains strict: excess preferred handles stay available by exact id through
/// their durable metadata instead of making memory growth unbounded.
pub fn retain_hot_handles(
    index: &mut ReferenceIndex,
    active_session_id: Option<SessionId>,
    max_handles: usize,
) -> usize {
    let max_handles = max_handles.clamp(1, MAX_HOT_HANDLE_LIMIT);
    if index.handles.len() <= max_handles {
        return 0;
    }

    index.handles.sort_by(|left, right| {
        handle_priority(left, active_session_id)
            .cmp(&handle_priority(right, active_session_id))
            .then_with(|| left.created_at.cmp(&right.created_at))
            .then_with(|| left.id.cmp(&right.id))
    });
    let removed = index.handles.len() - max_handles;
    index.handles.drain(..removed);
    index.handles.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then(left.id.cmp(&right.id))
    });
    index.cold_handle_count = index.cold_handle_count.saturating_add(removed as u64);
    removed
}

fn handle_priority(handle: &HandleRef, active_session_id: Option<SessionId>) -> u8 {
    u8::from(handle.pinned) * 2
        + u8::from(active_session_id.is_some_and(|id| handle.session_id == Some(id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HandleKind;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn handle(
        label: &str,
        age_minutes: i64,
        session_id: Option<SessionId>,
        pinned: bool,
    ) -> HandleRef {
        HandleRef {
            id: Uuid::now_v7(),
            kind: HandleKind::Text,
            label: label.to_string(),
            size: 1,
            sha256: "a".repeat(64),
            created_at: Utc::now() - Duration::minutes(age_minutes),
            session_id,
            project_root: None,
            continuity_id: None,
            pinned,
            trajectory: None,
        }
    }

    #[test]
    fn hot_index_is_strict_and_prefers_pinned_active_and_recent_handles() {
        let active = Uuid::now_v7();
        let mut index = ReferenceIndex {
            handles: vec![
                handle("old", 50, None, false),
                handle("recent", 1, None, false),
                handle("active", 40, Some(active), false),
                handle("pinned", 60, None, true),
                handle("middle", 20, None, false),
            ],
            cold_handle_count: 7,
        };

        assert_eq!(retain_hot_handles(&mut index, Some(active), 3), 2);
        assert_eq!(index.handles.len(), 3);
        assert_eq!(index.cold_handle_count, 9);
        let labels = index
            .handles
            .iter()
            .map(|handle| handle.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["pinned", "active", "recent"]);

        assert_eq!(retain_hot_handles(&mut index, Some(active), 3), 0);
        assert_eq!(index.cold_handle_count, 9);
    }
}

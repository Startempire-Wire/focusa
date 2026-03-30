//! Focus Frame — a single unit of focused work.
//!
//! Source: 03-focus-stack.md
//!
//! Required properties:
//!   - Title
//!   - Goal
//!   - Bound Beads issue ID
//!   - Focus State
//!   - Completion reason (when closed)

use crate::types::{FrameRecord, FrameStatus};

/// Get the active frame from a list.
pub fn find_active(frames: &[FrameRecord]) -> Option<&FrameRecord> {
    frames.iter().find(|f| f.status == FrameStatus::Active)
}

/// Get a frame by ID.
pub fn find_by_id(frames: &[FrameRecord], id: uuid::Uuid) -> Option<&FrameRecord> {
    frames.iter().find(|f| f.id == id)
}

/// Count active frames (should always be 0 or 1).
pub fn count_active(frames: &[FrameRecord]) -> usize {
    frames
        .iter()
        .filter(|f| f.status == FrameStatus::Active)
        .count()
}

//! Focus Stack — Hierarchical Execution Context.
//!
//! Source: 03-focus-stack.md, G1-detail-05-focus-stack-hec.md
//!
//! Core Invariants:
//!   1. Exactly one active Focus Frame exists at any time
//!   2. Every Focus Frame has a concrete intent
//!   3. Every Focus Frame maps to a Beads issue
//!   4. Frames are entered and exited explicitly
//!   5. Completed frames are archived, not forgotten
//!
//! Invalid Operations (Forbidden):
//!   - Multiple active frames
//!   - Implicit frame switching
//!   - Editing archived frames
//!   - Frames without Beads linkage
//!   - Skipping completion reasons
//!
//! Frame push/pop is handled exclusively by the reducer (see reducer.rs).
//! Actions dispatch through the daemon → reducer pipeline.

use crate::types::*;

/// Rebuild the stack path cache from root to active.
///
/// Includes cycle detection to prevent infinite loops on corrupt data.
pub fn rebuild_stack_path(stack: &mut FocusStackState) {
    stack.stack_path_cache.clear();
    if let Some(active_id) = stack.active_id {
        let mut current = Some(active_id);
        let mut path = Vec::new();
        let max_depth = stack.frames.len();
        while let Some(id) = current {
            path.push(id);
            if path.len() > max_depth {
                tracing::error!(
                    "Cycle detected in focus stack parent pointers — aborting path rebuild"
                );
                break;
            }
            current = stack
                .frames
                .iter()
                .find(|f| f.id == id)
                .and_then(|f| f.parent_id);
        }
        path.reverse();
        stack.stack_path_cache = path;
    }
}

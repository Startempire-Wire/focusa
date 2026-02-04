//! Core Reducer — single-writer state machine.
//!
//! Source: core-reducer.md
//!
//! Contract:
//!   reduce(state: FocusaState, event: FocusaEvent) -> ReductionResult
//!
//! Guarantees:
//!   - Deterministic
//!   - Replayable from event log
//!   - Crash-safe
//!   - Testable in isolation
//!   - Free of side effects
//!
//! Global Invariants (checked pre/post):
//!   1. At most one active Focus Frame exists
//!   2. Every Focus Frame maps to a Beads issue
//!   3. Focus State sections always exist
//!   4. Intuition Engine cannot mutate focus
//!   5. Focus Gate is advisory only
//!   6. Artifacts are immutable once registered
//!   7. Conversation never mutates cognition

use crate::types::{FocusaEvent, FocusaState, ReductionResult};

/// Core reducer: apply an event to state, producing new state + emitted events.
///
/// State version is incremented on every successful reduction.
pub fn reduce(state: FocusaState, event: FocusaEvent) -> Result<ReductionResult, ReducerError> {
    // TODO: Implement per core-reducer.md algorithm
    // Each of the 15 event types has specific handling rules.
    //
    // Pre-check invariants → apply event → post-check invariants → bump version.
    let _ = (&state, &event);
    todo!("Implement reducer — see core-reducer.md for full algorithm")
}

/// Verify all 7 global invariants hold on the given state.
pub fn check_invariants(state: &FocusaState) -> Result<(), ReducerError> {
    // INVARIANT 1: At most one active Focus Frame exists
    let active_count = state
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == crate::types::FrameStatus::Active)
        .count();
    if active_count > 1 {
        return Err(ReducerError::InvariantViolation(
            "Multiple active Focus Frames detected".into(),
        ));
    }

    // INVARIANT 2: Every Focus Frame maps to a Beads issue
    for frame in &state.focus_stack.frames {
        if frame.beads_issue_id.is_empty() {
            return Err(ReducerError::InvariantViolation(format!(
                "Frame {} has no Beads issue linkage",
                frame.id
            )));
        }
    }

    // TODO: Invariants 3-7

    Ok(())
}

/// Errors from the reducer.
#[derive(Debug, thiserror::Error)]
pub enum ReducerError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Invalid event for current state: {0}")]
    InvalidEvent(String),

    #[error("Frame not found: {0}")]
    FrameNotFound(String),

    #[error("Session error: {0}")]
    SessionError(String),
}

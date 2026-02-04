//! Focus State — the system's current state of mind.
//!
//! Source: 06-focus-state.md
//!
//! Update Rules:
//!   - Only changed sections are updated (incremental)
//!   - No full regeneration
//!   - Anchored to frame lifecycle
//!   - Contradictions must be logged
//!   - Prior decisions preserved
//!   - Resolution recorded explicitly

use crate::types::{FocusState, FocusStateDelta};

/// Apply an incremental delta to a Focus State.
///
/// Only non-None fields in the delta replace existing values.
pub fn apply_delta(state: &mut FocusState, delta: &FocusStateDelta) {
    if let Some(ref intent) = delta.intent {
        state.intent = intent.clone();
    }
    if let Some(ref decisions) = delta.decisions {
        // Append new unique decisions
        for d in decisions {
            if !state.decisions.contains(d) {
                state.decisions.push(d.clone());
            }
        }
    }
    if let Some(ref constraints) = delta.constraints {
        for c in constraints {
            if !state.constraints.contains(c) {
                state.constraints.push(c.clone());
            }
        }
    }
    if let Some(ref artifacts) = delta.artifacts {
        state.artifacts.extend(artifacts.iter().cloned());
    }
    if let Some(ref failures) = delta.failures {
        state.failures.extend(failures.iter().cloned());
    }
    if let Some(ref next_steps) = delta.next_steps {
        state.next_steps = next_steps.clone();
    }
    if let Some(ref current_state) = delta.current_state {
        state.current_state = current_state.clone();
    }
}

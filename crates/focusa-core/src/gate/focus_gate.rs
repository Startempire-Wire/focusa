//! Focus Gate algorithm — 5-step pipeline.
//!
//! 1. Normalize signals
//! 2. Candidate matching or creation
//! 3. Pressure update
//! 4. Surfacing (pressure >= threshold)
//! 5. User actions (accept/suppress/pin/resolve/ignore)
//!
//! Base pressure increments per SignalKind:
//!   user_input: +0.6, tool_output: +0.5, assistant_output: +0.2,
//!   warning: +0.7, error: +1.2, repeated_pattern: +0.8, manual_pin: +2.0
//!
//! Modifiers:
//!   Goal alignment: active frame ×1.3, stack path ×1.1, else ×0.8
//!   Recency (<5 min): +0.3
//!   Risk (error/warning): +0.4
//!   Decay per tick: pressure *= 0.98
//!
//! Surface threshold: 2.2 (configurable)

use crate::types::*;

/// Compute base pressure increment for a signal kind.
pub fn base_pressure(kind: SignalKind) -> f32 {
    match kind {
        SignalKind::UserInput => 0.6,
        SignalKind::ToolOutput => 0.5,
        SignalKind::AssistantOutput => 0.2,
        SignalKind::Warning => 0.7,
        SignalKind::Error => 1.2,
        SignalKind::RepeatedPattern => 0.8,
        SignalKind::ManualPin => 2.0,
        SignalKind::ArtifactChanged => 0.4,
        SignalKind::DeadlineTick => 0.5,
    }
}

/// Apply decay to all candidates.
pub fn decay_candidates(gate: &mut FocusGateState, decay_factor: f32) {
    for candidate in &mut gate.candidates {
        if !candidate.pinned {
            candidate.pressure *= decay_factor;
        }
    }
}

/// Get surfaced candidates (pressure >= threshold, not suppressed, not resolved).
pub fn surfaced_candidates(gate: &FocusGateState, threshold: f32) -> Vec<&Candidate> {
    let now = chrono::Utc::now();
    gate.candidates
        .iter()
        .filter(|c| {
            c.pressure >= threshold
                && c.state != CandidateState::Resolved
                && c.suppressed_until.map_or(true, |until| now >= until)
        })
        .collect()
}

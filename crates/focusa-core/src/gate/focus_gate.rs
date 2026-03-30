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

/// Apply decay to all candidates and clean up expired suppressions.
pub fn decay_candidates(gate: &mut FocusGateState, decay_factor: f32) {
    let now = chrono::Utc::now();
    for candidate in &mut gate.candidates {
        if !candidate.pinned {
            candidate.pressure *= decay_factor;
        }
        // Reset expired suppressions so state data stays honest.
        if candidate.state == CandidateState::Suppressed
            && candidate.suppressed_until.is_some_and(|until| now >= until)
        {
            candidate.state = CandidateState::Latent;
            candidate.suppressed_until = None;
        }
    }
}

/// Get surfaced candidates (pressure >= threshold, not resolved, not actively suppressed).
///
/// Time-based suppression: suppressed candidates become eligible again
/// once `suppressed_until` has passed.
pub fn surfaced_candidates(gate: &FocusGateState, threshold: f32) -> Vec<&Candidate> {
    let now = chrono::Utc::now();
    gate.candidates
        .iter()
        .filter(|c| {
            c.pressure >= threshold
                && c.state != CandidateState::Resolved
                && (c.state != CandidateState::Suppressed
                    || c.suppressed_until.is_some_and(|until| now >= until))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn make_candidate(pressure: f32, state: CandidateState) -> Candidate {
        let now = Utc::now();
        Candidate {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            kind: CandidateKind::SuggestFixError,
            label: "test".into(),
            origin_signal_ids: vec![],
            related_frame_id: None,
            state,
            pressure,
            last_seen_at: now,
            times_seen: 1,
            suppressed_until: None,
            resolution: None,
            pinned: false,
        }
    }

    #[test]
    fn test_base_pressure_values() {
        assert!(base_pressure(SignalKind::Error) > base_pressure(SignalKind::AssistantOutput));
        assert_eq!(base_pressure(SignalKind::ManualPin), 2.0);
    }

    #[test]
    fn test_decay_reduces_pressure() {
        let mut gate = FocusGateState::default();
        let c = make_candidate(2.0, CandidateState::Surfaced);
        gate.candidates.push(c);

        decay_candidates(&mut gate, 0.98);
        assert!((gate.candidates[0].pressure - 1.96).abs() < 0.01);
    }

    #[test]
    fn test_decay_skips_pinned() {
        let mut gate = FocusGateState::default();
        let mut c = make_candidate(2.0, CandidateState::Surfaced);
        c.pinned = true;
        gate.candidates.push(c);

        decay_candidates(&mut gate, 0.5);
        assert_eq!(gate.candidates[0].pressure, 2.0); // Unchanged.
    }

    #[test]
    fn test_decay_clears_expired_suppression() {
        let mut gate = FocusGateState::default();
        let mut c = make_candidate(0.0, CandidateState::Suppressed);
        c.suppressed_until = Some(Utc::now() - Duration::seconds(10)); // Already expired.
        gate.candidates.push(c);

        decay_candidates(&mut gate, 0.98);
        assert_eq!(gate.candidates[0].state, CandidateState::Latent);
        assert!(gate.candidates[0].suppressed_until.is_none());
    }

    #[test]
    fn test_surfaced_candidates_threshold() {
        let mut gate = FocusGateState::default();
        gate.candidates
            .push(make_candidate(1.0, CandidateState::Surfaced)); // Below threshold
        gate.candidates
            .push(make_candidate(3.0, CandidateState::Surfaced)); // Above threshold

        let result = surfaced_candidates(&gate, 2.2);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pressure, 3.0);
    }

    #[test]
    fn test_surfaced_excludes_resolved() {
        let mut gate = FocusGateState::default();
        gate.candidates
            .push(make_candidate(5.0, CandidateState::Resolved));

        let result = surfaced_candidates(&gate, 2.2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_surfaced_excludes_actively_suppressed() {
        let mut gate = FocusGateState::default();
        let mut c = make_candidate(5.0, CandidateState::Suppressed);
        c.suppressed_until = Some(Utc::now() + Duration::hours(1)); // Still active.
        gate.candidates.push(c);

        let result = surfaced_candidates(&gate, 2.2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_surfaced_includes_expired_suppression() {
        let mut gate = FocusGateState::default();
        let mut c = make_candidate(5.0, CandidateState::Suppressed);
        c.suppressed_until = Some(Utc::now() - Duration::seconds(10)); // Expired.
        gate.candidates.push(c);

        let result = surfaced_candidates(&gate, 2.2);
        assert_eq!(result.len(), 1);
    }
}

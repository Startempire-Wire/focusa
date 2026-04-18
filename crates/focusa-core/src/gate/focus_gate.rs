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
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Default signal window for aggregation (5 minutes).
const SIGNAL_WINDOW_SECS: i64 = 300;

/// Maximum signals retained in state (prevent unbounded growth).
const MAX_SIGNALS: usize = 1000;

/// Maximum candidates retained (spec: default 200).
const MAX_CANDIDATES: usize = 200;

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
        // Per G1-detail-06 UPDATE §Time as First-Class Signal:
        // Time signals have low base pressure but accumulate over time.
        SignalKind::InactivityTick => 0.3,
        SignalKind::LongRunningFrame => 0.4,
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

/// Compute fingerprint for a signal (Step 1 of G1-detail-06).
///
/// hash(kind + normalized summary + frame_context + key tags)
fn signal_fingerprint(signal: &Signal) -> u64 {
    let mut hasher = DefaultHasher::new();
    // kind
    std::mem::discriminant(&signal.kind).hash(&mut hasher);
    // normalized summary: lowercase, trim, first 100 chars
    let norm: String = signal
        .summary
        .to_lowercase()
        .trim()
        .chars()
        .take(100)
        .collect();
    norm.hash(&mut hasher);
    // frame_context
    signal.frame_context.hash(&mut hasher);
    // key tags (sorted for determinism)
    let mut sorted_tags = signal.tags.clone();
    sorted_tags.sort();
    sorted_tags.hash(&mut hasher);
    hasher.finish()
}

/// Compute pressure modifier for goal alignment (G1-detail-06 §Step 3).
///
/// - related_frame == active: ×1.3
/// - related_frame in stack_path: ×1.1  
/// - else: ×0.8
fn goal_alignment_modifier(
    related_frame: Option<FrameId>,
    active_id: Option<FrameId>,
    stack_path: &[FrameId],
) -> f32 {
    match related_frame {
        Some(fid) if Some(fid) == active_id => 1.3,
        Some(fid) if stack_path.contains(&fid) => 1.1,
        _ => 0.8,
    }
}

/// Run the full 5-step Focus Gate pipeline (G1-detail-06).
///
/// Steps:
///   1. Normalize signals — fingerprint for dedup
///   2. Candidate matching or creation — upsert by fingerprint
///   3. Pressure update — base + goal alignment + recency + risk modifiers
///   4. Surfacing — pressure >= threshold
///   5. User actions — handled externally (API/CLI)
///
/// Returns the number of newly surfaced candidates.
pub fn run_gate_pipeline(
    gate: &mut FocusGateState,
    active_id: Option<FrameId>,
    stack_path: &[FrameId],
    threshold: f32,
) -> usize {
    let now = Utc::now();
    let cutoff = now - chrono::Duration::seconds(SIGNAL_WINDOW_SECS);
    let mut newly_surfaced = 0usize;

    // Step 1 + 2 + 3: Process recent signals → match/create candidates → pressure update.
    // Collect signals within window; work on indices to avoid borrow issues.
    let recent_indices: Vec<usize> = gate
        .signals
        .iter()
        .enumerate()
        .filter(|(_, s)| s.ts > cutoff)
        .map(|(i, _)| i)
        .collect();

    for &idx in &recent_indices {
        let signal = &gate.signals[idx];
        let _fp = signal_fingerprint(signal);
        let kind = signal.kind;
        let summary = signal.summary.clone();
        let signal_id = signal.id;
        let related_frame = signal.frame_context;
        let signal_ts = signal.ts;

        // Step 3: Compute pressure increment with modifiers.
        let base = base_pressure(kind);
        let alignment = goal_alignment_modifier(related_frame, active_id, stack_path);
        let recency_bonus = if (now - signal_ts).num_seconds() < 300 {
            0.3
        } else {
            0.0
        };
        let risk_bonus = match kind {
            SignalKind::Error | SignalKind::Warning => 0.4,
            _ => 0.0,
        };
        let delta_p = base * alignment + recency_bonus + risk_bonus;

        // Step 2: Match existing candidate by fingerprint or create new.
        // Use fingerprint stored as label suffix for matching (simple approach —
        // a proper fingerprint index could be added later).
        let existing = gate.candidates.iter_mut().find(|c| {
            c.kind == crate::intuition::aggregation::suggest_candidate_kind_from_signal(kind)
                && c.related_frame_id == related_frame
                && c.state != CandidateState::Resolved
                && normalized_match(&c.label, &summary)
        });

        match existing {
            Some(candidate) => {
                candidate.times_seen += 1;
                candidate.last_seen_at = now;
                candidate.pressure += delta_p;
                candidate.updated_at = now;
                if !candidate.origin_signal_ids.contains(&signal_id) {
                    candidate.origin_signal_ids.push(signal_id);
                    // Cap origin signal IDs to prevent unbounded growth.
                    if candidate.origin_signal_ids.len() > 20 {
                        candidate.origin_signal_ids.remove(0);
                    }
                }
                // Re-surface if was latent and now above threshold.
                if candidate.state == CandidateState::Latent && candidate.pressure >= threshold {
                    candidate.state = CandidateState::Surfaced;
                    newly_surfaced += 1;
                }
            }
            None => {
                // Create new candidate.
                let candidate_kind =
                    crate::intuition::aggregation::suggest_candidate_kind_from_signal(kind);
                let initial_state = if delta_p >= threshold {
                    newly_surfaced += 1;
                    CandidateState::Surfaced
                } else {
                    CandidateState::Latent
                };
                gate.candidates.push(Candidate {
                    id: Uuid::now_v7(),
                    created_at: now,
                    updated_at: now,
                    kind: candidate_kind,
                    label: summary,
                    origin_signal_ids: vec![signal_id],
                    related_frame_id: related_frame,
                    state: initial_state,
                    pressure: delta_p,
                    last_seen_at: now,
                    times_seen: 1,
                    suppressed_until: None,
                    resolution: None,
                    pinned: false,
                });
            }
        }
    }

    // Step 4: Re-check all candidates for surfacing (some may have accumulated
    // pressure across multiple signals in this pass).
    for candidate in &mut gate.candidates {
        if candidate.state == CandidateState::Latent && candidate.pressure >= threshold {
            candidate.state = CandidateState::Surfaced;
            newly_surfaced += 1;
        }
    }

    // Enforce caps: signals and candidates.
    cap_signals(gate);
    cap_candidates(gate);

    newly_surfaced
}

/// Normalize and compare two summaries for candidate matching.
fn normalized_match(a: &str, b: &str) -> bool {
    let na: String = a.to_lowercase().chars().take(80).collect();
    let nb: String = b.to_lowercase().chars().take(80).collect();
    na == nb
}

/// Cap signals Vec to MAX_SIGNALS, removing oldest first.
pub fn cap_signals(gate: &mut FocusGateState) {
    if gate.signals.len() > MAX_SIGNALS {
        let remove = gate.signals.len() - MAX_SIGNALS;
        gate.signals.drain(..remove);
    }
}

/// Cap candidates to MAX_CANDIDATES, removing lowest-pressure resolved/latent first.
pub fn cap_candidates(gate: &mut FocusGateState) {
    if gate.candidates.len() <= MAX_CANDIDATES {
        return;
    }
    // Sort: resolved first, then latent, then by lowest pressure.
    gate.candidates.sort_by(|a, b| {
        let state_ord = |s: &CandidateState| -> u8 {
            match s {
                CandidateState::Resolved => 0,
                CandidateState::Latent => 1,
                CandidateState::Suppressed => 2,
                CandidateState::Surfaced => 3,
            }
        };
        state_ord(&a.state).cmp(&state_ord(&b.state)).then(
            a.pressure
                .partial_cmp(&b.pressure)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    gate.candidates.truncate(MAX_CANDIDATES);
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

    // ─── Gate Pipeline Tests ─────────────────────────────────────────

    fn make_signal(kind: SignalKind, summary: &str, frame: Option<FrameId>) -> Signal {
        Signal {
            id: Uuid::now_v7(),
            ts: Utc::now(),
            origin: SignalOrigin::Adapter,
            kind,
            frame_context: frame,
            summary: summary.into(),
            payload_ref: None,
            tags: vec![],
        }
    }

    #[test]
    fn test_pipeline_creates_candidate_from_signals() {
        let mut gate = FocusGateState::default();
        for _ in 0..3 {
            gate.signals
                .push(make_signal(SignalKind::Error, "build failed", None));
        }
        let surfaced = run_gate_pipeline(&mut gate, None, &[], 2.2);
        assert!(
            !gate.candidates.is_empty(),
            "Should have created candidates"
        );
        assert!(surfaced > 0, "Error signals should surface");
        assert_eq!(gate.candidates[0].kind, CandidateKind::SuggestFixError);
        assert!(gate.candidates[0].pressure > 0.0);
    }

    #[test]
    fn test_pipeline_merges_duplicate_signals() {
        let mut gate = FocusGateState::default();
        for _ in 0..5 {
            gate.signals
                .push(make_signal(SignalKind::Warning, "disk space low", None));
        }
        run_gate_pipeline(&mut gate, None, &[], 2.2);
        assert_eq!(gate.candidates.len(), 1, "Duplicate signals should merge");
        assert!(gate.candidates[0].times_seen >= 2);
    }

    #[test]
    fn test_pipeline_goal_alignment_boosts_active_frame() {
        let frame_id = Uuid::now_v7();
        let mut gate = FocusGateState::default();
        gate.signals
            .push(make_signal(SignalKind::Error, "test error", Some(frame_id)));
        run_gate_pipeline(&mut gate, Some(frame_id), &[frame_id], 0.0);
        let pressure_active = gate.candidates[0].pressure;

        let mut gate2 = FocusGateState::default();
        gate2
            .signals
            .push(make_signal(SignalKind::Error, "test error", None));
        run_gate_pipeline(&mut gate2, Some(frame_id), &[frame_id], 0.0);
        let pressure_none = gate2.candidates[0].pressure;

        assert!(
            pressure_active > pressure_none,
            "Active frame should get ×1.3 vs ×0.8: {} vs {}",
            pressure_active,
            pressure_none
        );
    }

    #[test]
    fn test_pipeline_caps_signals() {
        let mut gate = FocusGateState::default();
        for i in 0..1500 {
            gate.signals.push(make_signal(
                SignalKind::AssistantOutput,
                &format!("output {}", i),
                None,
            ));
        }
        run_gate_pipeline(&mut gate, None, &[], 2.2);
        assert!(
            gate.signals.len() <= 1000,
            "Signals should be capped at MAX_SIGNALS"
        );
    }

    #[test]
    fn test_pipeline_caps_candidates() {
        let mut gate = FocusGateState::default();
        for i in 0..300 {
            gate.signals.push(make_signal(
                SignalKind::Error,
                &format!("unique error {}", i),
                None,
            ));
        }
        run_gate_pipeline(&mut gate, None, &[], 0.0);
        assert!(
            gate.candidates.len() <= 200,
            "Candidates should be capped at MAX_CANDIDATES"
        );
    }

    #[test]
    fn test_pipeline_low_pressure_stays_latent() {
        let mut gate = FocusGateState::default();
        gate.signals.push(make_signal(
            SignalKind::AssistantOutput,
            "normal response",
            None,
        ));
        let surfaced = run_gate_pipeline(&mut gate, None, &[], 2.2);
        assert_eq!(surfaced, 0);
        assert_eq!(gate.candidates.len(), 1);
        assert_eq!(gate.candidates[0].state, CandidateState::Latent);
    }
}

//! Signal aggregation — group signals by type, frame, and time window.
//!
//! Source: 05-intuition-engine.md
//!
//! Aggregation produces:
//!   - Cumulative pressure per signal group
//!   - Summarized description
//!
//! Emission to Focus Gate is idempotent — updates existing candidates
//! where possible, creates new candidates only when necessary.

use crate::gate::focus_gate::base_pressure;
use crate::types::*;
use chrono::{Duration, Utc};

/// A group of related signals within a time window.
#[derive(Debug, Clone)]
pub struct SignalGroup {
    /// Representative signal kind for this group.
    pub kind: SignalKind,
    /// Frame these signals relate to (None = global).
    pub frame_id: Option<FrameId>,
    /// Number of signals in this group.
    pub count: usize,
    /// Cumulative pressure from all signals in group.
    pub cumulative_pressure: f32,
    /// Summary description.
    pub summary: String,
}

/// Aggregate signals within a time window, grouped by (kind, frame_id).
///
/// Returns groups that can be used to create or update candidates.
/// Only considers signals within the last `window_secs` seconds.
pub fn aggregate_signals(signals: &[Signal], window_secs: i64) -> Vec<SignalGroup> {
    let cutoff = Utc::now() - Duration::seconds(window_secs);

    let recent: Vec<&Signal> = signals.iter().filter(|s| s.ts > cutoff).collect();

    if recent.is_empty() {
        return vec![];
    }

    // Group by (kind, frame_context).
    let mut groups: Vec<SignalGroup> = Vec::new();

    for signal in &recent {
        let key_match = groups
            .iter_mut()
            .find(|g| g.kind == signal.kind && g.frame_id == signal.frame_context);

        match key_match {
            Some(group) => {
                group.count += 1;
                group.cumulative_pressure += base_pressure(signal.kind);
                // Update summary with latest signal info.
                group.summary = format!("{} (×{})", signal.summary, group.count);
            }
            None => {
                groups.push(SignalGroup {
                    kind: signal.kind,
                    frame_id: signal.frame_context,
                    count: 1,
                    cumulative_pressure: base_pressure(signal.kind),
                    summary: signal.summary.clone(),
                });
            }
        }
    }

    groups
}

/// Map a signal group to a candidate kind for the Focus Gate.
pub fn suggest_candidate_kind(group: &SignalGroup) -> CandidateKind {
    match group.kind {
        SignalKind::Error => CandidateKind::SuggestFixError,
        SignalKind::Warning => CandidateKind::SuggestFixError,
        SignalKind::RepeatedPattern => CandidateKind::SuggestPinMemory,
        SignalKind::ArtifactChanged => CandidateKind::SuggestCheckArtifact,
        SignalKind::DeadlineTick => CandidateKind::SuggestPushFrame,
        _ => CandidateKind::SuggestPushFrame,
    }
}

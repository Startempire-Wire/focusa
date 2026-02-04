//! Candidate management — create, pin, suppress, resolve.

use crate::types::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Create a new candidate from a signal.
pub fn create_candidate(
    kind: CandidateKind,
    label: String,
    signal_ids: Vec<SignalId>,
    related_frame_id: Option<FrameId>,
    initial_pressure: f32,
) -> Candidate {
    let now = Utc::now();
    Candidate {
        id: Uuid::now_v7(),
        created_at: now,
        updated_at: now,
        kind,
        label,
        origin_signal_ids: signal_ids,
        related_frame_id,
        state: CandidateState::Latent,
        pressure: initial_pressure,
        last_seen_at: now,
        times_seen: 1,
        suppressed_until: None,
        resolution: None,
        pinned: false,
    }
}

/// Pin a candidate (immune to decay, always eligible for surfacing).
pub fn pin(candidate: &mut Candidate) {
    candidate.pinned = true;
    candidate.updated_at = Utc::now();
}

/// Unpin a candidate.
pub fn unpin(candidate: &mut Candidate) {
    candidate.pinned = false;
    candidate.updated_at = Utc::now();
}

/// Suppress a candidate until a given time.
pub fn suppress(candidate: &mut Candidate, until: Option<DateTime<Utc>>) {
    candidate.state = CandidateState::Suppressed;
    candidate.suppressed_until = until;
    candidate.pressure = 0.0;
    candidate.updated_at = Utc::now();
}

/// Resolve a candidate.
pub fn resolve(candidate: &mut Candidate, resolution: String) {
    candidate.state = CandidateState::Resolved;
    candidate.resolution = Some(resolution);
    candidate.updated_at = Utc::now();
}

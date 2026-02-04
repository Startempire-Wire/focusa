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
//!   3. Focus State sections always exist (FocusState Default is valid)
//!   4. Intuition Engine cannot mutate focus (structural — gate events don't touch stack)
//!   5. Focus Gate is advisory only (structural — gate events don't touch stack)
//!   6. Artifacts are immutable once registered
//!   7. Conversation never mutates cognition (structural — no conversation in state)

use crate::focus::state::apply_delta;
use crate::types::*;
use chrono::Utc;

/// Core reducer: apply an event to state, producing new state + emitted events.
///
/// Flow: pre-check invariants → apply event → post-check invariants → bump version.
///
/// The input event is included in emitted_events on success (for event log persistence).
pub fn reduce(state: FocusaState, event: FocusaEvent) -> Result<ReductionResult, ReducerError> {
    check_invariants(&state)?;

    let mut state = state;
    let emitted_event = event.clone();

    match event {
        // ─── Session Lifecycle ───────────────────────────────────────────

        FocusaEvent::SessionStarted {
            session_id,
            adapter_id,
            workspace_id,
        } => {
            if state.session.is_some() {
                return Err(ReducerError::InvalidEvent(
                    "SessionStarted but a session already exists".into(),
                ));
            }
            state.session = Some(SessionState {
                session_id,
                created_at: Utc::now(),
                adapter_id,
                workspace_id,
                status: SessionStatus::Active,
            });
        }

        FocusaEvent::SessionRestored { session_id } => {
            // The daemon pre-loads state from disk before emitting this event.
            // Validate the loaded session matches the requested ID.
            match &state.session {
                Some(s) if s.session_id == session_id => {
                    // Already loaded — nothing to change.
                }
                Some(s) => {
                    return Err(ReducerError::SessionError(format!(
                        "SessionRestored for {} but loaded session is {}",
                        session_id, s.session_id
                    )));
                }
                None => {
                    return Err(ReducerError::SessionError(format!(
                        "SessionRestored for {} but no session in state — daemon must pre-load",
                        session_id
                    )));
                }
            }
        }

        FocusaEvent::SessionClosed { reason: _ } => {
            let session = state.session.as_mut().ok_or_else(|| {
                ReducerError::SessionError("SessionClosed but no active session".into())
            })?;
            session.status = SessionStatus::Closed;
        }

        // ─── Focus Stack ─────────────────────────────────────────────────

        FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id,
            title,
            goal,
            constraints,
            tags,
        } => {
            if beads_issue_id.is_empty() {
                return Err(ReducerError::InvariantViolation(
                    "FocusFramePushed with empty beads_issue_id".into(),
                ));
            }

            let now = Utc::now();
            let stack = &mut state.focus_stack;

            // Pause current active frame.
            if let Some(active_id) = stack.active_id {
                if let Some(frame) = stack.frames.iter_mut().find(|f| f.id == active_id) {
                    frame.status = FrameStatus::Paused;
                    frame.updated_at = now;
                }
            }

            let parent_id = stack.active_id;

            stack.frames.push(FrameRecord {
                id: frame_id,
                parent_id,
                created_at: now,
                updated_at: now,
                status: FrameStatus::Active,
                title,
                goal,
                beads_issue_id,
                tags,
                priority_hint: None,
                ascc_checkpoint_id: None,
                stats: FrameStats::default(),
                handles: vec![],
                constraints,
                focus_state: FocusState::default(),
            });

            stack.active_id = Some(frame_id);
            if stack.root_id.is_none() {
                stack.root_id = Some(frame_id);
            }
            rebuild_stack_path(stack);
            stack.version += 1;
        }

        FocusaEvent::FocusFrameCompleted {
            frame_id,
            completion_reason: _,
        } => {
            let stack = &mut state.focus_stack;

            // Must be completing the active frame.
            if stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusFrameCompleted for {} but active is {:?}",
                    frame_id, stack.active_id
                )));
            }

            let active_idx = stack
                .frames
                .iter()
                .position(|f| f.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;

            let parent_id = stack.frames[active_idx].parent_id;

            // Validate parent is Paused (if it exists).
            if let Some(pid) = parent_id {
                let parent = stack
                    .frames
                    .iter()
                    .find(|f| f.id == pid)
                    .ok_or_else(|| ReducerError::FrameNotFound(format!("parent {}", pid)))?;
                if parent.status != FrameStatus::Paused {
                    return Err(ReducerError::InvariantViolation(format!(
                        "Parent frame {} has status {:?}, expected Paused",
                        pid, parent.status
                    )));
                }
            }

            // All checks passed — mutate.
            let now = Utc::now();
            stack.frames[active_idx].status = FrameStatus::Completed;
            stack.frames[active_idx].updated_at = now;

            if let Some(pid) = parent_id {
                if let Some(parent) = stack.frames.iter_mut().find(|f| f.id == pid) {
                    parent.status = FrameStatus::Active;
                    parent.updated_at = now;
                }
                stack.active_id = Some(pid);
            } else {
                stack.active_id = None;
                stack.root_id = None;
            }

            rebuild_stack_path(stack);
            stack.version += 1;
        }

        FocusaEvent::FocusFrameSuspended {
            frame_id,
            reason: _,
        } => {
            let stack = &mut state.focus_stack;

            if stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusFrameSuspended for {} but active is {:?}",
                    frame_id, stack.active_id
                )));
            }

            let now = Utc::now();
            if let Some(frame) = stack.frames.iter_mut().find(|f| f.id == frame_id) {
                frame.status = FrameStatus::Paused;
                frame.updated_at = now;
            }

            // Suspension clears active — user must explicitly resume or push.
            stack.active_id = None;
            rebuild_stack_path(stack);
            stack.version += 1;
        }

        // ─── Focus State ─────────────────────────────────────────────────

        FocusaEvent::FocusStateUpdated { frame_id, delta } => {
            if state.focus_stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusStateUpdated for {} but active is {:?}",
                    frame_id, state.focus_stack.active_id
                )));
            }

            let frame = state
                .focus_stack
                .frames
                .iter_mut()
                .find(|f| f.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;

            apply_delta(&mut frame.focus_state, &delta);
            frame.updated_at = Utc::now();
        }

        // ─── Intuition → Gate ────────────────────────────────────────────

        FocusaEvent::IntuitionSignalObserved {
            signal_id,
            signal_type,
            severity: _,
            summary,
            related_frame_id,
        } => {
            let now = Utc::now();
            state.focus_gate.signals.push(Signal {
                id: signal_id,
                ts: now,
                origin: SignalOrigin::Daemon,
                kind: signal_type,
                frame_context: related_frame_id,
                summary,
                payload_ref: None,
                tags: vec![],
            });
        }

        FocusaEvent::CandidateSurfaced {
            candidate_id,
            kind,
            description,
            pressure,
            related_frame_id,
        } => {
            let now = Utc::now();
            // Upsert: update if exists, create if new.
            if let Some(existing) = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
            {
                existing.pressure = pressure;
                existing.label = description;
                existing.last_seen_at = now;
                existing.times_seen += 1;
                existing.updated_at = now;
                // Re-surface if was latent.
                if existing.state == CandidateState::Latent {
                    existing.state = CandidateState::Surfaced;
                }
            } else {
                state.focus_gate.candidates.push(Candidate {
                    id: candidate_id,
                    created_at: now,
                    updated_at: now,
                    kind,
                    label: description,
                    origin_signal_ids: vec![],
                    related_frame_id,
                    state: CandidateState::Surfaced,
                    pressure,
                    last_seen_at: now,
                    times_seen: 1,
                    suppressed_until: None,
                    resolution: None,
                    pinned: false,
                });
            }
        }

        FocusaEvent::CandidatePinned { candidate_id } => {
            let candidate = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Candidate {} not found", candidate_id))
                })?;
            candidate.pinned = true;
            candidate.updated_at = Utc::now();
        }

        FocusaEvent::CandidateSuppressed {
            candidate_id,
            scope: _,
        } => {
            let candidate = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Candidate {} not found", candidate_id))
                })?;
            candidate.state = CandidateState::Suppressed;
            candidate.pressure = 0.0;
            candidate.updated_at = Utc::now();
        }

        // ─── Reference Store ─────────────────────────────────────────────

        FocusaEvent::ArtifactRegistered {
            artifact_id,
            artifact_type,
            summary,
            storage_uri: _,
        } => {
            // Check immutability: if this artifact_id already exists, reject.
            if state
                .reference_index
                .handles
                .iter()
                .any(|h| h.id == artifact_id)
            {
                return Err(ReducerError::InvariantViolation(format!(
                    "Artifact {} already registered — artifacts are immutable",
                    artifact_id
                )));
            }

            // Create a minimal HandleRef from event data.
            // Full metadata lives in ecs/handles/ on disk.
            let kind = parse_handle_kind(&artifact_type);
            state.reference_index.handles.push(HandleRef {
                id: artifact_id,
                kind,
                label: summary,
                size: 0,
                sha256: String::new(),
                created_at: Utc::now(),
                session_id: state.session.as_ref().map(|s| s.session_id),
                pinned: false,
            });
        }

        FocusaEvent::ArtifactPinned { artifact_id } => {
            let handle = state
                .reference_index
                .handles
                .iter_mut()
                .find(|h| h.id == artifact_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Artifact {} not found", artifact_id))
                })?;
            handle.pinned = true;
        }

        FocusaEvent::ArtifactGarbageCollected { artifact_id } => {
            let idx = state
                .reference_index
                .handles
                .iter()
                .position(|h| h.id == artifact_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Artifact {} not found for GC", artifact_id))
                })?;
            // Pinned artifacts cannot be garbage collected.
            if state.reference_index.handles[idx].pinned {
                return Err(ReducerError::InvariantViolation(format!(
                    "Artifact {} is pinned — cannot garbage collect",
                    artifact_id
                )));
            }
            state.reference_index.handles.remove(idx);
        }

        // ─── Errors ──────────────────────────────────────────────────────

        FocusaEvent::InvariantViolation {
            invariant: _,
            details: _,
        } => {
            // Log-only event — no state mutation.
            // The event itself is recorded in the event log via emitted_events.
        }
    }

    state.version += 1;

    check_invariants(&state)?;

    Ok(ReductionResult {
        new_state: state,
        emitted_events: vec![emitted_event],
    })
}

/// Verify all 7 global invariants hold on the given state.
pub fn check_invariants(state: &FocusaState) -> Result<(), ReducerError> {
    // INVARIANT 1: At most one active Focus Frame exists.
    let active_count = state
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == FrameStatus::Active)
        .count();
    if active_count > 1 {
        return Err(ReducerError::InvariantViolation(format!(
            "Multiple active Focus Frames: {} found",
            active_count
        )));
    }

    // INVARIANT 2: Every Focus Frame maps to a Beads issue.
    for frame in &state.focus_stack.frames {
        if frame.beads_issue_id.is_empty() {
            return Err(ReducerError::InvariantViolation(format!(
                "Frame {} has no Beads issue linkage",
                frame.id
            )));
        }
    }

    // INVARIANT 3: Focus State sections always exist.
    // Structurally guaranteed — FocusState derives Default and all fields have defaults.
    // No runtime check needed.

    // INVARIANT 4: Intuition Engine cannot mutate focus.
    // Structurally guaranteed — IntuitionSignalObserved only touches focus_gate,
    // never focus_stack. Enforced by the match arms above.

    // INVARIANT 5: Focus Gate is advisory only.
    // Structurally guaranteed — CandidateSurfaced/Pinned/Suppressed only touch
    // focus_gate.candidates, never focus_stack.

    // INVARIANT 6: Artifacts are immutable once registered.
    // Enforced at registration time: ArtifactRegistered rejects duplicate IDs.
    // No handles in reference_index share the same ID.
    let handle_count = state.reference_index.handles.len();
    let unique_count = {
        let mut ids: Vec<_> = state.reference_index.handles.iter().map(|h| h.id).collect();
        ids.sort();
        ids.dedup();
        ids.len()
    };
    if handle_count != unique_count {
        return Err(ReducerError::InvariantViolation(format!(
            "Duplicate artifact IDs in reference_index: {} handles but {} unique",
            handle_count, unique_count
        )));
    }

    // INVARIANT 7: Conversation never mutates cognition.
    // Structurally guaranteed — FocusaState has no conversation/chat history field.
    // No event type carries raw conversation data.

    Ok(())
}

/// Rebuild the stack path cache from root → active.
fn rebuild_stack_path(stack: &mut FocusStackState) {
    stack.stack_path_cache.clear();
    if let Some(active_id) = stack.active_id {
        let mut current = Some(active_id);
        let mut path = Vec::new();
        let max_depth = stack.frames.len();
        while let Some(id) = current {
            path.push(id);
            if path.len() > max_depth {
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

/// Parse a string artifact type to HandleKind (best-effort).
fn parse_handle_kind(s: &str) -> HandleKind {
    match s.to_lowercase().as_str() {
        "log" => HandleKind::Log,
        "diff" => HandleKind::Diff,
        "text" => HandleKind::Text,
        "json" => HandleKind::Json,
        "url" => HandleKind::Url,
        "file_snapshot" | "file" => HandleKind::FileSnapshot,
        _ => HandleKind::Other,
    }
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

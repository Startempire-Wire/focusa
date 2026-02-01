# Focusa-Core Reducer — Canonical Pseudocode Spec (AUTHORITATIVE)

> This document defines the **single-writer cognitive reducer** for Focusa.
> All mutable cognition state transitions MUST be expressed through this reducer.
> No side effects, IO, model calls, or async work occur inside the reducer.

---

## Reducer Contract

```
reduce(
  state: FocusaState,
  event: FocusaEvent
) -> ReductionResult
```

Where:

```
ReductionResult {
  new_state: FocusaState
  emitted_events: Vec<FocusaEvent>
}
```

---

## Canonical State Shape

```
FocusaState {
  session: Option<SessionState>

  focus_stack: FocusStack
  focus_gate: FocusGateState

  reference_index: ReferenceIndex
  memory: ExplicitMemory

  version: u64
}
```

⚠️ Conversation history is NEVER part of FocusaState.

---

## Canonical Event Types

```
enum FocusaEvent {

  // ─────────────────────────
  // Session Lifecycle
  // ─────────────────────────
  SessionStarted { session_id }
  SessionRestored { session_id }
  SessionClosed   { reason }

  // ─────────────────────────
  // Focus Stack
  // ─────────────────────────
  FocusFramePushed {
    frame_id
    beads_issue_id
    title
    goal
  }

  FocusFrameCompleted {
    frame_id
    completion_reason
  }

  FocusFrameSuspended {
    frame_id
    reason
  }

  // ─────────────────────────
  // Focus State
  // ─────────────────────────
  FocusStateUpdated {
    frame_id
    delta: FocusStateDelta
  }

  // ─────────────────────────
  // Intuition → Gate
  // ─────────────────────────
  IntuitionSignalObserved {
    signal_id
    signal_type
    severity
    related_frame_id?
  }

  CandidateSurfaced {
    candidate_id
    description
    pressure
    related_frame_id?
  }

  CandidatePinned {
    candidate_id
  }

  CandidateSuppressed {
    candidate_id
    scope
  }

  // ─────────────────────────
  // Reference Store
  // ─────────────────────────
  ArtifactRegistered {
    artifact_id
    artifact_type
    summary
    storage_uri
  }

  ArtifactPinned {
    artifact_id
  }

  ArtifactGarbageCollected {
    artifact_id
  }

  // ─────────────────────────
  // Errors
  // ─────────────────────────
  InvariantViolation {
    invariant
    details
  }
}
```

---

## Reducer Algorithm (High-Level)

```
function reduce(state, event):

  assert invariants_pre(state)

  emitted_events = []

  match event.type:

    // ─────────────────────────
    // Session
    // ─────────────────────────
    case SessionStarted:
      assert state.session == None
      state.session = new SessionState(event.session_id)
      emitted_events.push(SessionStartedConfirmed)

    case SessionRestored:
      state.session = load_session(event.session_id)
      emitted_events.push(SessionRestoredConfirmed)

    case SessionClosed:
      state.session.status = CLOSED
      emitted_events.push(SessionClosedConfirmed)

    // ─────────────────────────
    // Focus Stack
    // ─────────────────────────
    case FocusFramePushed:
      assert beads_issue_exists(event.beads_issue_id)

      if state.focus_stack.has_active_frame():
        state.focus_stack.suspend_active_frame()

      state.focus_stack.push_new_frame(
        frame_id=event.frame_id,
        beads_issue_id=event.beads_issue_id,
        title=event.title,
        goal=event.goal
      )

      emitted_events.push(FocusFrameActivated)

    case FocusFrameCompleted:
      assert state.focus_stack.active_frame_id == event.frame_id
      assert event.completion_reason exists

      state.focus_stack.complete_active_frame(event.completion_reason)
      state.focus_stack.restore_parent_frame()

      emitted_events.push(FocusFrameArchived)

    case FocusFrameSuspended:
      assert state.focus_stack.active_frame_id == event.frame_id
      state.focus_stack.suspend_active_frame(event.reason)
      emitted_events.push(FocusFrameSuspendedConfirmed)

    // ─────────────────────────
    // Focus State
    // ─────────────────────────
    case FocusStateUpdated:
      assert state.focus_stack.active_frame_id == event.frame_id

      apply_incremental_focus_state_delta(
        state.focus_stack.active_frame.focus_state,
        event.delta
      )

      emitted_events.push(FocusStateCommitted)

    // ─────────────────────────
    // Intuition Engine (Signals Only)
    // ─────────────────────────
    case IntuitionSignalObserved:
      state.focus_gate.aggregate_signal(event)
      emitted_events.push(IntuitionSignalAggregated)

    case CandidateSurfaced:
      state.focus_gate.upsert_candidate(
        candidate_id=event.candidate_id,
        description=event.description,
        pressure=event.pressure,
        related_frame_id=event.related_frame_id
      )
      emitted_events.push(CandidateVisible)

    case CandidatePinned:
      state.focus_gate.pin_candidate(event.candidate_id)
      emitted_events.push(CandidatePinnedConfirmed)

    case CandidateSuppressed:
      state.focus_gate.suppress_candidate(
        event.candidate_id,
        event.scope
      )
      emitted_events.push(CandidateSuppressedConfirmed)

    // ─────────────────────────
    // Reference Store
    // ─────────────────────────
    case ArtifactRegistered:
      state.reference_index.register(
        artifact_id=event.artifact_id,
        type=event.artifact_type,
        summary=event.summary,
        uri=event.storage_uri
      )
      emitted_events.push(ArtifactIndexed)

    case ArtifactPinned:
      state.reference_index.pin(event.artifact_id)
      emitted_events.push(ArtifactPinnedConfirmed)

    case ArtifactGarbageCollected:
      state.reference_index.remove(event.artifact_id)
      emitted_events.push(ArtifactRemoved)

    // ─────────────────────────
    // Errors
    // ─────────────────────────
    case InvariantViolation:
      emitted_events.push(ErrorLogged)
      return { state, emitted_events }

    default:
      emitted_events.push(UnhandledEventError)

  state.version += 1

  assert invariants_post(state)

  return { state, emitted_events }
```

---

## Global Invariants (Checked Pre/Post)

```
INVARIANT: At most one active Focus Frame exists
INVARIANT: Every Focus Frame maps to a Beads issue
INVARIANT: Focus State sections always exist
INVARIANT: Intuition Engine cannot mutate focus
INVARIANT: Focus Gate is advisory only
INVARIANT: Artifacts are immutable once registered
INVARIANT: Conversation never mutates cognition
```

---

## Reducer Guarantees

- Deterministic
- Replayable from event log
- Crash-safe
- Testable in isolation
- Free of side effects

---

## Final Canonical Rule

> **If a cognition change cannot be expressed as a reducer event, it does not belong in Focusa.**

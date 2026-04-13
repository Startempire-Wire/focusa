# Tool and Action Contracts — Implementation Notes

## Purpose
This document details the implementation of SPEC 55 requirements.

It translates the normative contract language into the current Pi-extension/Focusa bridge behavior.
Where implementation is weaker than the desired contract, this document names the gap instead of pretending full coverage.

## Implementation Anchors

Current behavior is primarily implemented in:
- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/src/session.ts`
- `apps/pi-extension/src/turns.ts`

## Input Schema ✅
- Focusa bridge tools use typed schemas via tool registration.
- `focusa_decide`, `focusa_constraint`, and `focusa_failure` apply semantic validators before write.
- `pushDelta()` validates slot payload size/content before any Focus State update.

## Output Schema ⚠️ Partial
Current outputs are uneven, but the Focusa write path is now more explicit.

Implemented patterns:
- interactive tool rejection messages with explicit reason
- reason-coded `pushDelta()` result for Focus State writes
- command submission acknowledgement checks such as `r?.accepted`
- UI notifications for degraded or fallback outcomes

Current `pushDelta()` failure reasons:
- `offline`
- `no_active_frame`
- `validation_rejected`
- `write_failed`

Gap:
- not every path emits a single typed result envelope with `accepted|completed|partial|unknown`
- some tool `details` payloads still expose user-facing text more richly than machine-readable result metadata

## Side Effects ✅
| Tool/action | Side effects |
|------|---------------|
| `focusa_scratch` | Appends working notes to `/tmp/pi-scratch/turn-XXXX/notes.txt` |
| `focusa_decide` / `focusa_constraint` / `focusa_failure` | Appends cognitive entries to Focus State via `pushDelta()` |
| `focusa_intent` / `focusa_current_focus` | Replaces a singleton Focus State slot |
| `focusa_next_step` / `focusa_open_question` / `focusa_recent_result` / `focusa_note` | Appends bounded entries to Focus State |
| compaction submit | Sends remote command requests to Focusa |
| local fallback compact | Mutates Pi session state without canonical Focusa authority |

## Failure Modes ✅

### Validation failure
Implemented in semantic validators and `validateSlot()`.
Invalid entries are rejected before any Focus State write.

### Dependency failure
If Focusa is unavailable, `pushDelta()` returns `offline`.
If Focusa is healthy but Pi has no active frame, the bridge attempts shared frame recovery before returning `no_active_frame`.
Session health checks also mark Focusa offline and disable write tools.

### Permission / execution failure
Filesystem and subprocess tools inherit Pi/runtime permissions.
Remote command submission depends on Focusa daemon availability and remote acceptance.

### Timeout / ambiguous completion
Compaction submission can fail to confirm acceptance.
Current implementation falls back to local compact when configured, but that fallback is explicitly marked non-canonical.

### Partial success
The main partial-success pattern is local success without confirmed remote persistence:
- scratchpad written but Focusa write absent
- local compact completed while canonical Focusa compact did not complete

### Rollback failure
No universal rollback exists for append-only Focus State writes.
Recovery is typically forward-only: write a corrective entry, reset frame, or create a fresh frame.

## Idempotency ✅ with Explicit Classes

| Tool/action | Class | Current implementation note |
|---|---|---|
| `focusa_scratch` | Append-only non-idempotent | Repeating creates another note |
| `focusa_decide` / `focusa_constraint` / `focusa_failure` | Append-only non-idempotent | No automatic dedupe in bridge |
| `focusa_intent` / `focusa_current_focus` | Conditionally idempotent | Singleton slot semantics |
| `focusa_next_step` / `focusa_open_question` / `focusa_recent_result` / `focusa_note` | Append-only non-idempotent | Replay duplicates entries |
| `pushDelta()` | Conditional | Safe only when caller understands append vs replace slot behavior; now returns reason-coded failure states |
| remote compact submit | Conditional | Uses `idempotency_key`, but current keys include timestamps and therefore scope one submission attempt, not global semantic dedupe |
| local fallback compact | Non-idempotent in session terms | Replays create more compaction events and altered summaries |

## Retry Policy ✅ with Caveats

### Safe automatic retry
- read-only status/health queries

### Built-in write recovery
- missing active frame on Focus State write now triggers one shared `ensurePiFrame()` recovery attempt before the write is declared failed
- concurrent recovery paths converge through one in-flight frame promise instead of racing duplicate frame creation
- transient fetches where no side effect occurred

### Retry only with guards
- `edit`-style mutation: retry only if old-text precondition still matches
- Focusa command submit: retry only with deliberate idempotency handling and post-state verification
- singleton slot writes: retry only after checking current slot value

### No blind retry
- append-only Focus State entries
- scratchpad note creation
- external process execution
- local fallback compact after ambiguous prior completion

## Verification Hooks ✅
Implemented hooks include:
- semantic validation before tool write
- `pushDelta()` slot validation before Focus State update
- Focusa command acceptance checks (`r?.accepted`)
- health checks and reconnect logic in session lifecycle
- explicit UI labeling of non-canonical fallback compaction

Gap:
- command acceptance is not yet the same as end-to-end completion proof
- some write paths still expose boolean success instead of richer verification metadata

## Degraded Fallback ✅
Implemented degraded behaviors:
- Focusa offline → write tools disabled / bridge returns failure
- critical unrecoverable `focusa_decide` / `focusa_constraint` / `focusa_failure` writes auto-mirror to scratchpad with reason metadata
- compaction submit unavailable → optional local compact fallback
- fallback compact instructions explicitly mark the result non-canonical
- offline state is surfaced in UI instead of pretending success

## Reproducible Validation Checklist

Use this checklist to validate current behavior:
1. Force Focusa offline and verify write tools surface `offline` semantics rather than generic unavailability.
2. Clear `activeFrameId` while Focusa is healthy and verify a write triggers shared frame recovery before succeeding.
3. Submit an invalid decision/constraint/failure payload and verify validation rejection occurs before remote write.
4. Force `/focus/update` failure and verify the tool reports `write_failed`.
5. Force failed critical cognitive writes and verify a `focusa_write_fallback` scratch entry appears in `/tmp/pi-scratch/turn-XXXX/notes.txt`.
6. Verify telemetry includes `focusa_write_attempt`, `focusa_write_recovery_attempt`, `focusa_write_recovery_result`, `focusa_write_failed`, `focusa_write_succeeded`, and `focusa_write_fallback`.

## Required Clarification for Future Work
The following would strengthen SPEC 55 compliance further:
- unify all tool results under a richer typed result envelope, not only `pushDelta()`
- distinguish `accepted`, `completed`, `partial`, and `unknown`
- add dedupe keys for append-only cognitive writes where replay is plausible
- add explicit post-command verification for remote compaction completion

## Success Condition
SPEC 55 is now documented with concrete failure-mode and idempotency semantics tied to the current bridge implementation, including where behavior is intentionally non-canonical or only partially verified.

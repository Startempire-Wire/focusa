# Spec88 Current Workpoint — Compaction Handoff

**Date:** 2026-04-28  
**Purpose:** Compact/resume anchor for the current Pi conversation at high context usage.

## Active work

Spec88: Ontology-backed Workpoint Continuity and Pi Compaction Integration.

## Completed in this conversation

- Created `docs/88-ontology-backed-workpoint-continuity.md`.
- Created `docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md`.
- Created `docs/evidence/SPEC88_WORKPOINT_CONTRACT_MATRIX_2026-04-28.md`.
- Created/closed spec bead `focusa-ql1o`.
- Created implementation epic `focusa-a2w2`.
- Created 11 child implementation beads `focusa-a2w2.1` through `focusa-a2w2.11`.
- Closed `focusa-a2w2.1` Phase 0 contract matrix.

## Current bead state

- Epic: `focusa-a2w2` — in progress.
- Closed: `focusa-a2w2.1` — Phase 0 matrix.
- Next ready: `focusa-a2w2.2` — Phase 1 core Workpoint types and reducer events.

## Next implementation target

Implement `focusa-a2w2.2`:

- add Workpoint core structs/enums in `crates/focusa-core/src/types.rs` or a new `workpoint` module;
- add reducer events for checkpoint/promote/reject/supersede/resume/drift/degraded fallback;
- enforce bounded vectors, active pointer transition, canonical/degraded semantics;
- add unit tests for defaults, bounds, supersession, and reducer acceptance/rejection.

## Key design invariant

Focusa preserves continuation as typed workpoint state, not transcript tail.

## Pi compaction invariant

Before compaction: checkpoint typed workpoint.  
After compaction: resume from WorkpointResumePacket.  
After resume: drift detection verifies the agent stayed on ActiveMissionSet + ActionIntent.

## Repo caution

Work in `/home/wirebot/focusa` as user `wirebot`.
Do not bundle unrelated untracked file:

- `docs/evidence/SPEC_INTENT_VS_ACTUAL_CODE_RUNTIME_GAP_AUDIT_2026-04-23.md`

## Validation reminders

- Use `bd` from repo root.
- Preserve existing dirty work not created by this session.
- Run targeted Rust/TS tests for changed code.
- `bd sync` after bead changes.

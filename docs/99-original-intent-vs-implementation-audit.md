# Spec99 - Original Intent vs Implementation Audit

Status: initial full audit record
Date: 2026-06-05
Related: `docs/98-project-root-crdt-reconciliation-foundation-spec.md`

---

## 1. Executive verdict

The original Focusa intent was **not** global singleton state and was **not** one-session-only.

The original docs support:

- high-level multiplexing across projects, sessions, devices, harnesses, and threads
- local-first CRDT-backed sync/reconciliation substrate
- timestamped async concurrency
- proposal/resolution for canonical decisions
- explicit session/instance/attachment/thread identity
- no silent cross-session or cross-device mutation

The implementation partially contains those ideas, but the canonical runtime shape still centers many subsystems around singleton `current` / `active` / `last` fields. Later patches add project-root/continuity guards around those fields instead of rebuilding the state root around verified project authority and CRDT reconciliation.

Trust concern is valid: the implementation foundation diverged from original intent.

---

## 2. Audit scope

Compared original intent docs against current implementation in:

- `docs/02-runtime-daemon.md`
- `docs/G1-detail-03-runtime-daemon.md`
- `docs/40-instance-session-attachment-spec.md`
- `docs/41-proposal-resolution-engine.md`
- `docs/43-multi-device-sync.md`
- `docs/96-trajectory-projection-and-daemon-stability-spec.md`
- `docs/98-project-root-crdt-reconciliation-foundation-spec.md`
- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/reducer.rs`
- `crates/focusa-core/src/sync/crdt.rs`
- `crates/focusa-api/src/routes/*`
- `apps/pi-extension/src/*`

This is an architecture audit, not a line-by-line security audit.

---

## 3. Original intent evidence

### 3.1 Sessions are isolated, not singleton-only

`docs/02-runtime-daemon.md` states:

- exactly one daemon instance owns mutable state **per session**
- all state belongs to exactly one session
- cross-session access is forbidden by default

This supports session isolation, not a global singleton product model.

### 3.2 Multiple sessions/projects were explicitly intended

`docs/G1-detail-03-runtime-daemon.md` states the session identity layer exists to ensure clean isolation across:

- multiple harness runs
- terminal restarts
- concurrent projects

It also defines `sessions: HashMap<SessionId, SessionMeta>` and `workspace_id` as grouping metadata that does not merge state automatically.

### 3.3 Multiplexing was first-class

`docs/40-instance-session-attachment-spec.md` defines Instances, Sessions, Attachments, and Thread attachment semantics.

Key intent:

- instances can have many sessions over time
- a session can attach to multiple threads if it declares a primary attachment
- a thread can be attached by many instances simultaneously
- one engineer can work across many projects
- many instances can work on the same thread concurrently
- attachments grant proposal authority, not direct mutation authority

### 3.4 Concurrency and conflict resolution were first-class

`docs/41-proposal-resolution-engine.md` defines PRE as timestamped async concurrency across multiple instances/sessions without locks.

Key intent:

- observations are append-only and always concurrent
- decisions become proposals
- proposals are grouped by thread/target/time window
- one canonical outcome is resolved while alternatives remain preserved

### 3.5 CRDT was real, not "CRDT-ish"

`docs/43-multi-device-sync.md` defines local-first bidirectional sync, deterministic behavior, idempotency, event cursors, and no silent merges.

`crates/focusa-core/src/sync/crdt.rs` implements:

- `VectorClock`
- causal comparison
- `CrdtEvent`
- `CrdtLog`
- `merge_remote`
- Lamport fallback ordering
- deterministic conflict resolver

The intent class is real CRDT-backed reconciliation substrate.

---

## 4. Implementation divergence summary

| Area | Original intent | Implementation reality | Severity |
| --- | --- | --- | --- |
| Root authority | Identifiable source of truth / thread/project/session identity | Many canonical states remain under singleton global state tree | Critical |
| Focus Stack | Active frame within session/thread/workspace | `FocusStackState.active_id` is one field in global `FocusaState` | Critical |
| Focus State | Scoped to frame/session/thread | `/focus/update` can fall back to daemon active frame | Critical |
| Workpoint | Continuity packet under project/workstream | `WorkpointState.active_workpoint_id` singleton, scope guards patched around it | High |
| Trajectory | Project/workstream trajectory with advisory similarity | `TrajectoryState.active_trajectory_id` singleton; prior-project fallback exists | High |
| Work-loop | Continuous work under project/workstream/loop | `WorkLoopState.current_task` and API `active_writer` are singleton | High |
| Sync/CRDT | Multi-session/device update reconciliation | Remote receive imports all remote events as observations by default | High |
| Pi extension | Multiplex-aware session binding | `S.*` singleton caches for project/workpoint/trajectory/focus | High |
| API status | Daemon overview plus scoped project views | `/status` exposes singleton active frame/session summary | Medium |
| Tests | Prove bleed impossible | Tests exist for some scope guards, not full project-root CRDT foundation | High |

---

## 5. Detailed findings

### F1 - `FocusaState` remains a singleton state root

Implementation evidence: `crates/focusa-core/src/types.rs` defines one `FocusaState` with:

- `session: Option<SessionState>`
- `focus_stack: FocusStackState`
- `workpoint: WorkpointState`
- `trajectory: TrajectoryState`
- `work_loop: WorkLoopState`
- `instances`, `attachments`, `threads`

Intent mismatch:

- docs intend instances/sessions/attachments/threads as concurrency substrate
- implementation stores those entities but keeps the canonical cognitive state as a single root instead of `ProjectRoot -> Workstream -> scoped state`

Severity: Critical.

Repair direction:

- introduce project-root keyed state registry
- materialize scoped state under verified root/workstream
- keep daemon-global state only for health, peer registry, and unowned telemetry

### F2 - Focus Stack active pointer is still singleton

Implementation evidence:

- `crates/focusa-core/src/types.rs:1088-1096` defines `FocusStackState { root_id, active_id, frames, stack_path_cache }`
- invariant still says exactly one active Focus Frame exists at any time
- `crates/focusa-core/src/reducer.rs:843-850` patches push behavior by finding same `project_root + continuity_id` active frame
- reducer still sets `stack.active_id = Some(frame_id)` after pushing

Intent mismatch:

- original one-active-frame rule makes sense inside one session/thread/workstream
- implementation leaves `active_id` at global stack level, then patches same-continuity behavior around it

Severity: Critical.

Repair direction:

- move active frame pointer under `WorkstreamState` or `ThreadState`
- unscoped active frame reads return `scope_required`

### F3 - Focus State write path can still fall back to daemon active frame

Implementation evidence:

- `crates/focusa-api/src/routes/focus.rs:825-838` uses `focusa.focus_stack.active_id` when `body.frame_id` is absent
- comments acknowledge preserving Pi session-scoped frame writes without relying on global active-frame alignment, but fallback still exists
- unsafe project root guard exists, but only after active frame fallback selection

Intent mismatch:

- docs require session/thread attachment identity and no cross-session leakage
- fallback to daemon active frame is exactly the global-current pattern that can bleed

Severity: Critical.

Repair direction:

- require frame id or scoped root/workstream identity for Focus State mutation
- remove daemon active frame fallback for canonical writes

### F4 - Workpoint active pointer is singleton with guards, not foundationally scoped

Implementation evidence:

- `crates/focusa-core/src/types.rs:703-711` defines `WorkpointState.active_workpoint_id`
- `crates/focusa-api/src/routes/workpoint.rs:747` defines `active_workpoint_for_scope`
- routes use project/continuity checks around singleton active pointer

Intent mismatch:

- Workpoint identity should be under project-root/workstream authority
- active Workpoint should be `active_workpoint_by_workstream`, not one global pointer with filters

Severity: High.

Repair direction:

- move Workpoint records/index under project/workstream registry
- delete or de-authorize singleton `active_workpoint_id`

### F5 - Trajectory active pointer is singleton and prior-project fallback exists

Implementation evidence:

- `crates/focusa-core/src/types.rs:902-909` defines `TrajectoryState.active_trajectory_id`
- `crates/focusa-api/src/routes/trajectory.rs:328` consults `state.trajectory.active_trajectory_id`
- `crates/focusa-api/src/routes/trajectory.rs:924-935` supports `allow_prior_project_trajectory`
- payload exposes `fallback_prior_project_trajectory`

Intent mismatch:

- similarity/prior trajectory can be advisory, but not authority
- existence of prior-project fallback increases bleed/confusion risk

Severity: High.

Repair direction:

- active trajectory must be keyed by project root + continuity/workstream
- prior project trajectory should be cross-scope advisory only, never in canonical trajectory packet path

### F6 - Work-loop is daemon-global

Implementation evidence:

- `crates/focusa-core/src/types.rs:439-466` defines `WorkLoopState.current_task`
- `crates/focusa-api/src/routes/work_loop.rs` uses global `current_task` throughout status, checkpoint, select-next, replay, and secondary-loop evidence
- `crates/focusa-api/src/routes/work_loop.rs:188` and related functions expose/use API-level `active_writer`

Intent mismatch:

- continuous work should attach to a project root/workstream/loop id
- daemon-global current task cannot represent many concurrent projects or sessions

Severity: High.

Repair direction:

- introduce scoped `WorkLoopId` or project/workstream keyed loop state
- writer ownership scoped per loop

### F7 - Sync imports remote events as observations by default, limiting CRDT update reconciliation

Implementation evidence:

- `crates/focusa-api/src/routes/sync_receive.rs` says Policy #2: all imported remote events are tagged observations
- `EventLogEntry.is_observation = true`
- `crates/focusa-core/src/reducer.rs:226-241` returns without mutating canonical state when `is_observation` is true
- `sync_transfer.rs` has a narrow exception for ownership transfer events

Intent mismatch:

- CRDT class implies multi-device/multi-session update reconciliation with integrity
- observation-only import is safe as a default quarantine, but insufficient as the full reconciliation foundation

Severity: High.

Repair direction:

- keep observation quarantine for unverified/foreign roots
- for same verified project root, route compatible update operations through CRDT reconciliation and PRE/resolution

### F8 - CRDT implementation exists but is not the canonical state root

Implementation evidence:

- `crates/focusa-core/src/sync/crdt.rs` implements vector clocks and CRDT log merge
- canonical state structs are not shaped around `CrdtLog<ProjectEvent>` or project-root timelines
- core reducer still materializes directly into singleton `FocusaState`

Intent mismatch:

- CRDT substrate exists as a module, but canonical cognitive surfaces do not derive from project-root CRDT timelines

Severity: High.

Repair direction:

- make project timeline/event log the canonical source for materialized scoped state
- store causal metadata in canonical-capable events

### F9 - Pi extension keeps singleton session-local caches that can bleed on project switches

Implementation evidence: `apps/pi-extension/src/state.ts` defines singleton mutable fields:

- `S.sessionCwd`
- `S.continuityId`
- `S.activeWorkpointPacket`
- `S.activeWorkpointSummary`
- `S.lastTrajectoryClarity`
- `S.lastProjectIdentity`
- `S.lastProjectVerify`
- `S.focusStateCache`

`apps/pi-extension/src/compaction.ts` and `apps/pi-extension/src/tools.ts` repeatedly use `S.sessionCwd || process.cwd()` and local fallback packets.

Intent mismatch:

- Pi is one runtime instance that may switch projects/roots
- cache should be keyed by verified project root/workstream, not stored as one current packet

Severity: High.

Repair direction:

- replace singleton caches with `scopeCache[ProjectRootKey][WorkstreamKey]`
- make compaction/resume block unless packet scope matches current verified root and continuity

### F10 - Project identity work is good but retrofitted

Implementation evidence:

- `FocusaSessionIdentity` and `ProjectIdentityRecord` exist in `types.rs`
- project identity tools and docs now enforce project root + continuity in many surfaces
- `docs/96` explicitly says scoped queries must not fall back to global active frame

Intent mismatch:

- ProjectIdentity is currently a guard/adapter layer around older singleton state
- should be the root key of canonical state, not a late validation envelope

Severity: Medium-high.

Repair direction:

- promote ProjectIdentity/ProjectRootKey to root of state model

---

## 6. What was implemented correctly or partially correctly

Important: the implementation is not worthless. Several pieces are aligned but misplaced.

### Correct / useful pieces

- `ProjectIdentityRecord` and `FocusaSessionIdentity` exist.
- `FrameRecord` has `project_root` and `continuity_id` fields.
- reducer push logic now pauses only same-root/same-continuity active frames.
- Workpoint routes have scope rejection and canonical envelope logic.
- Trajectory routes mark similarity as advisory and must-not-merge in tests.
- Sync has idempotent event import and peer cursor model.
- CRDT module has vector-clock and Lamport foundations.
- PRE exists and supports proposal scoring/resolution concepts.
- Thread ownership exists and reducer can enforce owner machine checks.

### Why those are insufficient

They are not organized as the foundation.

They sit beside or around singleton canonical state instead of owning the canonical state root.

---

## 7. Root cause

The original docs planned multiplexing and reconciliation, but the implementation appears to have taken an MVP shortcut:

```text
one daemon state -> one active stack -> one active frame -> one current workpoint/trajectory/task
```

Then later specs/tools added:

```text
project_root + continuity guards around singleton state
```

That path creates permanent risk because any missed route/tool/cache can still read or write the singleton current state.

The correct foundation is:

```text
verified project root -> workstreams -> sessions/attachments -> CRDT timeline -> materialized scoped state
```

---

## 8. Required architectural correction

### New source-of-truth shape

```text
ProjectRegistry
  ProjectRootKey
    identity/fingerprint/signals
    project timeline (CRDT log)
    sessions
    workstreams
      focus stack/state
      workpoints
      trajectory
      work-loop
```

### Required invariants

1. No canonical read/write without verified project root authority.
2. No daemon-global active/current/last pointer may be canonical.
3. Sessions attach to roots/workstreams; sessions do not define project identity alone.
4. Same-root concurrent updates are reconciled, not discarded as read-only observations.
5. Conflicting cognitive decisions go through deterministic resolution/PRE.
6. All alternatives remain inspectable timeline facts.
7. Cross-project aggregates are labeled advisory views only.

---

## 9. Implementation decomposition

### Phase A - Inventory and hard fail unscoped canonical paths

- enumerate every API/CLI/Pi route that returns active/current/last state
- add `scope_required` fail-closed behavior where safe
- mark singleton state fields legacy/non-authoritative

### Phase B - ProjectRootKey + WorkstreamKey

- define canonical key types in core
- add keys to event envelope
- add project registry skeleton
- add migration mapping from old records to project/workstream buckets

### Phase C - Workpoint + Trajectory first

- move active Workpoint and Trajectory under workstream state
- delete singleton active authority from route payloads
- update Pi Workpoint/Trajectory packets

### Phase D - Focus Stack + Focus State

- move active frame under workstream/thread state
- require frame/workstream identity on Focus State writes
- remove daemon active-frame fallback

### Phase E - Work-loop

- add loop id or workstream keyed loop state
- scope writer ownership and current task

### Phase F - CRDT reconciliation path

- carry vector/Lamport metadata on project events
- integrate `CrdtLog` with project timeline persistence
- support same-root compatible update merge
- route conflicting same-root decisions into PRE

### Phase G - Pi extension scope cache

- replace singleton `S.last*` and `S.active*` caches with keyed cache
- compaction packets include project root key, continuity, timeline head
- ambiguous cwd blocks canonical binding

### Phase H - Regression proof

- two-project bleed test
- same-root two-session timeline test
- same-root compatible update merge test
- same-root conflicting decision resolution test
- multi-device sync test
- ambiguous `/root` cwd test
- compaction cross-project injection test

---

## 10. Immediate files likely requiring change

Core:

- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/reducer.rs`
- `crates/focusa-core/src/runtime/daemon.rs`
- `crates/focusa-core/src/runtime/persistence_sqlite.rs`
- `crates/focusa-core/src/sync/crdt.rs`
- `crates/focusa-core/src/pre/*`

API:

- `crates/focusa-api/src/routes/focus.rs`
- `crates/focusa-api/src/routes/session.rs`
- `crates/focusa-api/src/routes/workpoint.rs`
- `crates/focusa-api/src/routes/trajectory.rs`
- `crates/focusa-api/src/routes/work_loop.rs`
- `crates/focusa-api/src/routes/sync.rs`
- `crates/focusa-api/src/routes/sync_receive.rs`
- `crates/focusa-api/src/routes/sync_transfer.rs`
- `crates/focusa-api/src/routes/events_sqlite.rs`

Pi:

- `apps/pi-extension/src/state.ts`
- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/src/compaction.ts`
- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/project-root.ts`

Docs/tests:

- `docs/98-project-root-crdt-reconciliation-foundation-spec.md`
- this audit
- static route-contract tests
- reducer tests
- sync tests
- Pi compaction/resume tests

---

## 11. Severity-ranked gap list

### Critical

1. Singleton `FocusaState` root for canonical cognition.
2. Singleton `FocusStackState.active_id` used by routes.
3. Focus State writes can fall back to daemon active frame.

### High

4. Singleton Workpoint active pointer.
5. Singleton Trajectory active pointer and prior-project fallback path.
6. Singleton Work-loop current task/writer.
7. CRDT module not integrated as project-root canonical timeline.
8. Sync receive observation-only path for all remote updates.
9. Pi singleton caches and fallback packets.

### Medium

10. ProjectIdentity is a guard, not the root state key.
11. API `/status` still presents singleton current state.
12. Tests prove patches, not impossible bleed.

---

## 12. Trust restoration criteria

Trust should not be restored by declarations. Trust requires proof:

- route inventory shows no unscoped canonical current/active/last path
- reducer rejects canonical mutation without ProjectRootKey/WorkstreamKey
- two projects can run concurrently with no bleed
- two sessions can update same root and produce one diffable timeline
- same-root conflicts become resolution records, not silent overwrites
- Pi compaction cannot inject stale cross-project context
- sync reconciles same-root updates with causal metadata

---

## 13. Bottom line

Original intent was multiplexed, local-first, CRDT-backed reconciliation with integrity.

Implementation contains pieces of that intent but does not use them as the foundation. The current architecture is a patched singleton model with scope guards.

Spec98 is the repair foundation. This audit is the evidence record for why that repair is necessary.

---

## 14. Second-pass findings: non-obvious implementation divergences

This section records the deeper pass requested after the initial audit. These findings are not just more confirmation of singleton state; they identify disconnected or under-integrated intended subsystems.

### S1 - CRDT module is effectively orphaned from production sync

Evidence:

- `crates/focusa-core/src/sync/crdt.rs` defines `VectorClock`, `CrdtEvent`, `CrdtLog`, `merge_remote`, and `ConflictResolver`.
- Search across `crates/` and `apps/` found `CrdtLog`, `VectorClock`, and `ConflictResolver` used only in the CRDT module and its tests, plus re-export in `sync/mod.rs`.
- Production sync routes use SQLite event cursors and observation import, not `CrdtLog` causal merge.

Implication:

The CRDT implementation exists as a tested island, not as the production reconciliation engine. This is a larger divergence than singleton active pointers: the named CRDT foundation is not wired into canonical runtime sync.

### S2 - SQLite event schema lacks causal/project-root columns

Evidence:

- `crates/focusa-core/src/runtime/persistence_sqlite.rs` creates `events(event_id, ts, origin, correlation_id, payload_json, machine_id, instance_id, session_id, thread_id, is_observation)`.
- There are indexes for timestamp, machine, session, and thread.
- There are no first-class columns for `project_root`, `project_fingerprint`, `continuity_id`, `vector_clock`, `lamport_ts`, `workspace_id`, or `repo_signature`.

Implication:

Even if payload JSON contains some identity fields, persistence is not shaped for project-root source-of-truth timelines or CRDT causal queries. It cannot efficiently or structurally enforce the intended root-keyed reconciliation model.

### S3 - PRE implementation is threshold scoring, not the documented resolution-window model

Evidence:

- `docs/41-proposal-resolution-engine.md` specifies windows keyed by `thread_id + target + window_start`, gathers competing proposals, scores, selects one winner/no-winner, emits resolution events, and records citations.
- `crates/focusa-core/src/pre/mod.rs` implements a simple proposal list with score, deadline, threshold acceptance, rejection, and garbage collection.
- `crates/focusa-api/src/routes/proposals.rs` submits proposals and can apply a focus-change proposal, but `apply_focus_change_proposal` calls `reduce_with_meta(..., Some(machine_id), None, false)`, passing no `thread_id` ownership boundary.

Implication:

The real PRE is not yet the intended conflict-resolution substrate for concurrent same-root/session decisions. It is closer to a scored queue than deterministic windowed reconciliation.

### S4 - Thread ownership enforcement is bypassed by many first-party route calls

Evidence:

- `reduce_with_meta` can enforce owner machine when `thread_id` is supplied.
- Many canonical routes call `reduce_with_meta(current, event, None, None, false)`:
  - `routes/turn.rs`
  - `routes/focus.rs`
  - `routes/session.rs`
  - `routes/trajectory.rs`
  - `routes/workpoint.rs`
- Passing `thread_id=None` disables thread ownership enforcement.

Implication:

Thread ownership exists in reducer code but is optional metadata. Canonical API routes frequently omit it, so the intended ownership model is not actually a hard authority boundary.

### S5 - CLI operator/agent status silently composes unscoped surfaces

Evidence:

- `crates/focusa-cli/src/main.rs` operator status calls:
  - `/v1/status`
  - `/v1/project/identity`
  - `/v1/trajectory/view?mode=summary`
  - `/v1/workpoint/resume` with only `{mode:"operator_summary"}`
- agent status calls `/v1/workpoint/current` and `/v1/work-loop/status?summary_only=true` without project root or continuity.
- `crates/focusa-cli/src/commands/continue_work.rs` reads `/v1/workpoint/current` and work-loop status without scope before resuming work-loop.

Implication:

The CLI itself reinforces daemon-global "current" ergonomics. Even if API routes are guarded, the user-facing command center still trains operators/agents to trust unscoped current state.

### S6 - Session/project identity exists but is not the event-store partition key

Evidence:

- `ProjectIdentityRecord` and `FocusaSessionIdentity` exist in `types.rs`.
- `FrameRecord` has `project_root` and `continuity_id`.
- But `FocusaState` remains one root object and SQLite events do not index project root/continuity as first-class columns.

Implication:

ProjectIdentity is currently a validation/envelope layer, not the database/state partition foundation. This explains why cross-session bleed can recur despite many scope guards.

### S7 - Sync transfer has only a narrow mutating exception

Evidence:

- `sync_receive.rs` imports all normal remote events as `is_observation=true`.
- `sync_transfer.rs` has a special mutating path only for `ThreadOwnershipTransferred`.

Implication:

The system has one narrow canonical remote update path but no general same-root CRDT reconciliation path for ordinary cognitive/project changes.

### S8 - The implementation has the right words but wrong attachment point

Evidence:

- Terms present: project root, continuity, thread, proposal, CRDT, vector clock, ownership, observation, session identity.
- Attachment point wrong: they are attached to routes, payloads, tests, or side modules while singleton `FocusaState` remains the central authority.

Implication:

This is why earlier patches looked plausible: the vocabulary is there. The failure is not missing concepts; it is that the concepts are not the foundation.

---

## 15. Earliest-doc non-CRDT divergence pass

This pass excludes CRDT/multi-device reconciliation and asks: what veered in early implementation relative to the first Focusa docs?

### E1 - Conversation leaked into canonical state shape

Early docs say conversation history is never part of `FocusaState` and conversation never mutates cognition.

Implementation stores `ActiveTurn { raw_user_input, assembled_prompt }` inside `FocusaState`, writes it during `TurnStarted`, and later mutates assembled prompt in `routes/turn.rs` and `routes/proxy.rs`.

This is not a CRDT problem. It is a boundary error between runtime correlation and cognition state.

### E2 - Direct mutation bypassed the reducer contract

Early docs say all mutable cognition transitions must go through the single-writer reducer and be replayable.

Implementation directly mutates state in API paths, including active-turn prompt fields and semantic memory contradiction resolution in `routes/proxy.rs`.

This weakens replayability and violates the reducer-as-authority foundation.

### E3 - Beads validation became string validation

Early docs make Beads the task authority and forbid frames without Beads linkage.

Implementation checks only that `beads_issue_id` is non-empty in focus/command paths; proposals can default to `proposal-focus-change`; tests and helpers use synthetic `issue-{title}` ids.

This means "Beads authority" became "string present".

### E4 — Focus Gate lost its central gate role

Early docs made Focus Gate the salience/candidate gate and said no subsystem may bypass Focus Gate or Focus Stack.

Implementation lets many subsystems develop independent authority surfaces: Workpoint, Trajectory, Attention Recall, Work-loop, and direct API focus updates. Some are useful, but they are not reconciled with the original Focus Gate model.

This is a spec evolution gap: either Focus Gate must be restored as the canonical front door or explicitly superseded.

### E6 — HLT history was reconstructive, not first-class

Original intent: HLT (High-Level Trajectory) as the north-star of the project should be saved precisely with a historical list always easily available.

Previous implementation: HLT lived inside `TrajectoryProjectionRecord` snapshots and trajectory projections, with no dedicated append-only ledger. HLT history was effectively reconstructive from compressed summaries.

Fixed (2026-06-08): Implemented HLT Ledger per Spec98/99:
- Append-only JSONL ledger: `{data_dir}/hlt-ledger/{project_root_hash}/hlt.jsonl`
- Scope-bounded by `(project_root, continuity_id)`, no singleton
- CRDT-grade with Lamport timestamps
- API: `GET /v1/hlt/history` + tool `focusa_hlt_history`
- `define_goal` route atomically appends entry on successful HLT change

### E5 — Expression Engine accumulated non-expression responsibilities

Early docs define Expression Engine as deterministic output shaping only, with no reasoning/planning, no memory mutation, no implicit summarization, and explicit degradation.

Implementation includes richer dynamic assembly and nearby proxy behavior that resolves memory contradictions, injects external context, and writes assembled prompt back into state.

This blurred expression, retrieval, memory maintenance, and runtime turn storage.

### E6 - Reference Store lost strict source/scope ergonomics

Early docs require lossless, immutable, scoped, explicit rehydration.

Implementation enforces handle-id immutability in reducer, but the store route discovers newly written handles by label, and handle access is still broadly global in later ontology/prompt routes.

This is a storage/ergonomics leak independent of CRDT.

### E7 - Runtime, telemetry, and cognition planes were not kept separate

Early daemon docs describe staged runtime flow. Core reducer docs keep cognition state pure and event-replayable.

Implementation interleaves runtime active turn, assembled prompt, CLT interaction append, memory cleanup, and telemetry into the same operational state flow.

The non-CRDT foundational correction is to split:

1. canonical cognition state,
2. runtime session/turn correlation,
3. telemetry/history/event log.

---

## 16. Core-system coverage pass

This pass covers all core systems, not just the first visible non-CRDT veers.

| Core system | Original intent | Current veer | Correction theme |
|---|---|---|---|
| Runtime daemon | Single authoritative execution context; not planner/orchestrator | Work-loop/autonomy/trajectory make daemon behave like governance + orchestration host | Split cognition authority from orchestration authority |
| Reducer/events | Replayable, pure, complete state transitions | Some routes still direct-write state and mark freshness without replay-equivalent events | Ban canonical `mark_external_mutation` without event |
| Focus State | Single source of truth for meaning | Workpoint/Trajectory/Attention Recall now carry action authority and continuity | Decide supersession or integrate as typed Focus State extensions |
| Focus Stack | One active frame, Beads-bound | Real Beads existence not always validated; Workpoint may carry active task better than stack | Restore Beads validation and define stack/workpoint relation |
| Focus Gate | Advisory candidates only | Attention Recall and work-loop gates govern action authority | Separate advisory salience from action governance |
| Intuition Engine | Async signals only, no actions/mutations | Later background/control systems can resemble decisions rather than signals | Route decisions through PRE or explicit governance plane |
| ASCC / Workpoint / CLT / Trajectory Ladder | Separate per-frame meaning, resume continuity, lineage, and goal-route context | Workpoint checkpoints are now more reliable compaction artifacts, while Trajectory Ladder carries route/goal context | Align contracts without merging systems |
| CLT | Append-only lineage, handles only, not authority | Lineage tooling can become reasoning/next-action input | Keep CLT advisory until promoted through canonical state |
| Memory | Opt-in, minimal, confirmed writes | Proxy runs contradiction cleanup; later extraction pressure expanded | Require source/confidence/confirmation posture for memory writes/cleanup |
| Reference Store/ECS | Scoped, lossless handles; explicit rehydration | Store returns by label; global handle scans exist | Exact created handle + scoped identity + auditable rehydration |
| Expression Engine | Deterministic rendering only | Proxy pipeline mixes enrichment, eval/regeneration, prompt writes | Split render/retrieval/eval/runtime telemetry stages |
| Proxy adapters | Transparent, semantics-preserving, fail-open | Provider shims and inline evaluation can alter request/response behavior | Opt-in shims with visible stage telemetry |
| Telemetry | Observation, not cognition | Telemetry/resource buffers mutate same state/version | Separate telemetry version/freshness from cognition version |
| Training/contribution | Post-cognition read-only export; no raw conversation | Contribution queue/policy lives in FocusaState and mutates directly | Classify as export governance, not cognition |
| PRE/proposals | Decisional changes resolved as proposals | Many direct canonical routes bypass PRE windows | Define PRE-required vs single-writer-local exemptions |
| Ontology/read indexes | Advisory intelligence/read model | Active-object/next-action projections may look canonical | Mark ontology projections advisory unless promoted |
| Workpoint | Later continuity layer | Became canonical compaction/resume authority without updating early docs | Formalize as continuity authority layer |
| Trajectory | Later project goal projection | Can compete with Focus State for goal authority | Bind to project+continuity and define supersession rules |
| Work-loop | Later continuous execution | Supersedes "not automation engine/scheduler" boundary | Declare bounded orchestration plane or externalize it |
| API/CLI | Status/stack/memory/events control | Many route classes with mixed mutation semantics | Add route taxonomy and tests |

### 16.1 The central pattern

The recurring problem is not one bad subsystem. It is **authority-plane collapse**:

- canonical cognition,
- governance decisions,
- runtime correlation,
- telemetry,
- read indexes,
- export queues,
- orchestration state,
- advisory projections

all accumulated inside one state object and one route surface.

### 16.2 The correction principle

Every core system must answer four questions:

1. Is this canonical truth, advisory projection, runtime cache, or telemetry/export?
2. If it mutates state, what replayable event proves it?
3. What identity scope gates it: project, continuity, session, thread, frame, or workpoint?
4. Can it ever affect action authority, and if so through which governance path?

### 16.3 Priority order for implementation

1. **Plane taxonomy**: classify fields/routes/events before moving code.
2. **ActiveTurn separation**: remove raw turn/prompt buffers from cognition authority.
3. **Replay enforcement**: make direct mutations impossible for canonical state.
4. **Beads/Focus Stack validation**: restore task authority invariants.
5. **ASCC/Workpoint/CLT/Trajectory Ladder alignment**: one handoff route across separate systems, with no subsystem merge.
6. **Memory/ECS/Expression hardening**: opt-in memory, scoped handles, deterministic expression.
7. **Governance front door**: decide Focus Gate/PRE/Attention Recall/Work-loop roles.
8. **Route taxonomy tests**: every route declares mutation class and authority scope.

---

## 17. Cascading-effects audit lens

Operator guidance: decisions must be evaluated for their cascading effects across the whole project and for how an agent benefits or is handicapped.

### 17.1 Why this changes the audit

The audit cannot simply say "move X out of `FocusaState`" or "make Y advisory." Those changes can improve reducer purity while harming agent continuity if the agent loses clear handoff context.

The correct lens is:

> Purity without operational continuity is a handicap. Continuity without authority boundaries is drift.

### 17.2 Required decision impact notes

Each implementation bead should add impact notes for:

- Agent benefit
- Agent handicap risk
- Project-wide downstream dependencies
- Mitigation / handoff contract
- Tests proving the agent still has enough context

### 17.3 High-risk cascades to watch

1. **ActiveTurn separation**
   - Benefit: removes conversation bleed from canonical cognition.
   - Risk: agent loses turn correlation and prompt diagnostics.
   - Mitigation: runtime-only turn cache + explicit telemetry/handle rehydration.

2. **Trajectory Ladder advisory status**
   - Benefit: prevents route model from overriding operator/project scope.
   - Risk: agent ignores HLT/MLG/STG and becomes locally reactive.
   - Mitigation: Workpoint resume includes trajectory as route context with advisory flag.

3. **Workpoint as continuation contract**
   - Benefit: resumed agent gets exact next action and evidence hooks.
   - Risk: Workpoint silently supersedes Focus State/ASCC meaning.
   - Mitigation: handoff envelope states Workpoint authority and ASCC/Trajectory/CLT corroboration roles.

4. **Beads validation restoration**
   - Benefit: task authority becomes real again.
   - Risk: proposals/demos/tests become brittle if they need synthetic tasks.
   - Mitigation: explicit noncanonical proposal/demo frame class.

5. **Telemetry/export separation**
   - Benefit: observation pruning cannot mutate cognition.
   - Risk: agents miss operational warnings if telemetry is too detached.
   - Mitigation: telemetry remains visible but labeled noncanonical.

6. **Ontology advisory flagging**
   - Benefit: read-index intelligence does not become false truth.
   - Risk: active-object resolution becomes underpowered.
   - Mitigation: promote ontology findings through Workpoint/evidence/PRE when needed.

### 17.4 Agent usefulness invariant

For every correction, a resumed agent must still be able to answer:

1. What project am I in?
2. What is the current mission?
3. What is the current trajectory route?
4. What is the concrete next action?
5. What evidence proves the current state?
6. What context is advisory only?
7. What authority boundaries prevent drift?
8. What recovery path applies if state is stale/degraded?

If a correction makes any answer harder, the correction needs a mitigation before implementation.

### 17.5 Propagation status

The cascade/agent-impact requirement has been propagated to all `focusa-877z.1-.13` implementation beads. Future code changes should start from `focusa-877z.8` so every route/field/event taxonomy decision is reviewed for project-wide effects before local refactors land.

---

## 18. All-surfaces audit requirement

Operator guidance: foundational changes affect **all surfaces** and must be right before implementation: daemon, API, CLI, Pi plugin, and Focusa tools.

### 18.1 Audit expansion

The authority-plane audit must not stop at core Rust state. For every proposed correction, audit must identify expected impact on:

- daemon/core reducer and runtime caches,
- persistence/events and replay,
- API route taxonomy and response envelopes,
- CLI human and JSON output,
- Pi plugin resume/context injection behavior,
- Focusa Pi tool schemas/result envelopes,
- Workpoint and Trajectory tool behavior,
- docs/skills/examples,
- tests and live proof harnesses,
- migration/backcompat for existing packets and snapshots.

### 18.2 Agent benefit / handicap across surfaces

A correction benefits agents only if the same authority semantics appear consistently everywhere the agent can touch Focusa:

- API says canonical/advisory/degraded.
- CLI says the same thing for humans/scripts.
- Pi plugin injects the same distinction into prompts.
- Focusa tools expose the same envelope to agents.
- Docs/skills teach the same route.

If any surface says something different, the agent is handicapped by split-brain semantics.

### 18.3 Priority consequence

`focusa-877z.8` becomes the mandatory first implementation worksheet. Its output must be reviewed before any local refactor under `.1-.13` lands, because daemon-only purity can break API/CLI/Pi/tool continuity.

### 18.4 Menubar/UI surface addition

The Mac menubar app must be included in the all-surfaces audit. It is an authority-display and safe-control surface, not merely observability.

Required audit coverage:

- canonical/advisory/degraded/stale display parity with API/CLI/Pi tools,
- Workpoint resume and handoff UX,
- Trajectory Ladder HLT/MLG/STG display with advisory labeling,
- Focus State/ASCC structured meaning without raw chat,
- CLT lineage as history only,
- Work-loop writer/conflict controls,
- daemon/API/CLI/Pi/Beads/tool health,
- resource/LowMem and token pressure telemetry,
- evidence handle viewing and explicit rehydration,
- contribution/privacy controls,
- recovery commands and copyable diagnostic packets,
- docs/bead/checkpoint navigation.

If the menubar presents authority differently from API/CLI/Pi tools, agents and humans get split-brain guidance. Foundational changes must keep menubar semantics in lockstep with the route/tool envelope contract.

---

## 19. Pi tools and UIAI browser integration audit

Operator guidance: Pi agent tools are very important, and UIAI Engine browser integration must be included in foundational-change impact analysis.

### 19.1 Server-observed baseline

KH exposes the UIAI compatibility surface at `localhost:7456`, forwarded through authenticated SSH to the OVH sticky worker pool; browser compute and Chromium memory execute on OVH. Health reports browser service standby/idle-off with diagnostics and bounded async eval enabled, max page pool `2`, queue depth `0`, and an `agent_pressure` packet recommendation. UIAI docs and tool metadata advertise Focusa scope echo, stable `uiai-*` evidence refs, `/api/agent/research-packet`, and diagnostics-first browser debugging.

Focusa already integrates this through:

- `focusa_browser_diagnostics_intake`,
- `focusa_tool_doctor` UIAI health/pressure reporting,
- UIAI `focusa_scope` echo into diagnostics/session errors,
- ResearchDiagnosticsPacket schema `uiai.focusa_research_diagnostics_packet.v1`,
- visual workflow evidence routes,
- Pi extension UIAI/browser tool contracts.

### 19.2 Audit impact

All authority-plane corrections must include Pi/UIAI effects. A change is incomplete if it preserves daemon purity but breaks:

- Pi tool result envelopes,
- Focusa tool choreography,
- UIAI diagnostics intake,
- Workpoint evidence handoff,
- ResearchDiagnosticsPacket parity,
- browser health/pressure routing,
- visual evidence handle stability,
- Pi TUI/RPC/JSON/MCP/CLI parity.

### 19.3 UIAI/Pi split-brain hazard

The high-risk failure mode is split-brain evidence semantics:

- UIAI says a diagnostic packet is proof,
- Pi renders it as next action,
- Focusa treats it as advisory or unscoped,
- Menubar/CLI shows another state.

The fix is a single cross-surface envelope: `canonical`, `advisory`, `degraded`, `stale`, `scope_status`, `evidence_refs`, `preferred_focusa_tool`, `next_tools`, and `recovery_hint` must mean the same thing across UIAI, API, CLI, Pi plugin, Focusa tools, and menubar.

### 19.4 Required additions to `focusa-877z.8`

The authority-plane taxonomy worksheet must add Pi/UIAI columns:

- affected `focusa_*` tools,
- affected `uiai_*` Pi tools,
- Focusa tool-contract registry impact,
- UIAI packet/schema impact,
- browser diagnostics/evidence impact,
- `focusa_scope` handling,
- Pi render/compaction impact,
- TUI/RPC/JSON/MCP/CLI parity,
- UIAI reliability proof needed.

### 19.5 Deeper Pi/UIAI risk register

| Risk | Why it matters | Required mitigation |
|---|---|---|
| UIAI guided packet workflow hardcodes `/home/wpuiai/uiai-engine` Focusa scope | Cross-project packet use can create scope bleed if agents treat UIAI scope as Focusa authority | Always obtain project/root/continuity from Focusa Workpoint/ProjectIdentity for cross-project use; mark hardcoded UIAI scope as local demo/default |
| ResearchDiagnosticsPacket lacks full Focusa tool-result authority flags | Agents may treat packet proposal as captured evidence | Render packet as proposal-only until Focusa capture/intake/link succeeds |
| `focusa_browser_diagnostics_intake` is Pi-only but critical | Headless/API/CLI users get weaker browser evidence path | Document exact lower-level fallback or add daemon/API composite intake |
| Visual evidence store returns handle by label polling | Concurrent/duplicate visual labels can return wrong handle | Return exact created handle with write correlation/idempotency and scope fields |
| UIAI browser pressure is operational telemetry | Browser queue/pressure should narrow workflows but not mutate cognition truth | Surface through tool doctor/Menubar/CLI as noncanonical operational blocker/warning |
| Compact UIAI rendering can hide durable-capture status | Tool-output flood improves, but agents may miss that Focusa capture is pending | Compact line must state packet composed vs Focusa-captured vs scope-missing/degraded |
| Cross-repo proof required | Focusa tool-envelope changes can break UIAI packet/Pi/MCP/CLI parity | Run/cite Focusa and UIAI proof gates for any foundational contract change |

### 19.6 Required worksheet additions for `focusa-877z.8/.14`

Add these columns to the authority-plane worksheet:

- `packet_status_semantics`: proposal-only, captured, linked, degraded, stale, or rejected.
- `scope_source`: Focusa Workpoint/ProjectIdentity, UIAI local default, operator input, or missing.
- `cross_project_risk`: whether UIAI project scope can bleed into Focusa project scope.
- `headless_parity`: Pi-only, API, CLI, MCP, RPC/JSON parity status.
- `visual_evidence_handle_source`: exact-created-handle vs label lookup.
- `compact_render_required_text`: what one-line Pi output must say to prevent false authority.
- `proof_commands`: Focusa and UIAI commands required before claiming the change.

---

## 20. Formalized defaults, configurability, and missing artifacts

Operator guidance: use opinionated defaults with configurability. Reduce friction by making the correct path automatic, not optional ceremony.

### 20.1 Final default rule

Defaults protect agents from drift; configurability supports expert workflows without weakening the baseline.

Default classifications:

- Missing scope → degraded/advisory.
- UIAI packet → proposal-only until Focusa capture/link succeeds.
- Workpoint → continuation authority when canonical and scope-matched.
- Trajectory Ladder → project/workstream north star: durable HLT with adaptive MLG/STG/waypoints.
- CLT → history/lineage only.
- Telemetry/resource/UIAI pressure → noncanonical operational warning.
- Raw blobs/logs/screenshots/page bodies → handles only.
- Synthetic Beads → noncanonical proposal/demo only.
- Canonical mutation → reducer/PRE/Workpoint path only.

### 20.2 Configurability rule

Configurability is profile-based:

- `safe_default`
- `builder`
- `audit_strict`
- `lowmem`
- `browser_debug`
- `headless_ci`
- `demo_noncanonical`

Advanced overrides are allowed, but must record reason, affected surfaces, benefit, risk, rollback, and proof.

### 20.3 Trajectory Ladder / HLT persistence reconciliation

Operator-confirmed policy:

- Trajectory Ladder is the north star.
- HLT persists per `project_root + continuity_id` and auto-loads on Pi restart/session continuation.
- The manual project-root `hlt-ledger.md` pattern becomes unnecessary for normal continuation.
- HLT update prompt threshold is **7 resumed sessions**.
- Earlier HLT prompts require explicit operator steering, project identity change, or durable supersession evidence.
- MLG, STG, and waypoints are inferred more frequently from recent file changes, git activity, Beads, Workpoint, ontology, Focus State/current focus, evidence refs, predictions, and metacog signals.
- Inferred lower-ladder values remain advisory/proposal until accepted through Workpoint/Trajectory/evidence paths.

### 20.4 Implementation-ready worksheet and remaining generated artifacts

The worksheet seed at `docs/worksheets/focusa-877z.8-authority-taxonomy.yaml` is now `implementation_ready_seed` and covers the core authority surfaces. Planning exit status is `planning_complete_with_guardrails`: code movement can start from worksheet IDs only after explicit implementation authorization, and the project still needs generated/wired artifacts:

1. shared envelope schema/stubs,
2. migration/backcompat implementation,
3. policy profile registry implementation,
4. proof bundle map runner,
5. Menubar state contract implementation,
6. headless fallback for Pi-only diagnostics intake,
7. exact-handle evidence write semantics,
8. UIAI packet capture-status rendering,
9. cross-project UIAI scope guard,
10. generated docs/lint path,
11. side-effect classification tests.

Critical planning caveats:

- TL authority must be rendered as split authority, not blanket advisory: HLT north-star, lower ladder advisory, Workpoint action authority.
- Prior-project TL fallback needs explicit provenance and scope status in shared envelopes before compaction/resume can rely on it.
- The 7-resumed-session HLT prompt policy remains a planned requirement until lifecycle counter wiring exists.
- Compaction hot paths need bounded calls, cached packets, or combined routes before adding more packet refreshes.
- `implementation_ready_seed` means planning-ready, not code-complete.

### 20.5 Expected side effects

Expected good effects:

- safer compaction resume,
- less transcript-tail dependence,
- less cross-project/session bleed,
- clearer Pi/UIAI/menubar/CLI status,
- better browser evidence handoff,
- better privacy and artifact handling,
- higher proof quality.

Expected friction risks:

- stricter route/tool fields,
- old packets demoted or warned,
- more tests before release,
- more explicit degraded/advisory states,
- some demo/synthetic workflows need noncanonical labels,
- packet-as-proof shortcuts stop working.

Mitigation:

- generate tables/tests from the worksheet,
- scaffold route/tool defaults,
- render compact traffic-light status,
- bundle proof commands,
- keep advanced overrides explicit and reversible.

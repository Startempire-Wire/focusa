# Spec98 — Project-Root CRDT Reconciliation Foundation

Status: draft record spec
Date: 2026-06-05
Scope: Focusa core, API, CLI, Pi tools, sync, Workpoint, Trajectory, Focus Stack, Focus State, work-loop

---

## 1. Problem statement

Focusa was intended to support high-integrity multiplexing: many projects, many sessions, many devices, many harnesses, and concurrent work with deterministic reconciliation.

The observed project/session bleed indicates a foundational implementation error: singleton daemon `current` / `active` / `last` pointers were allowed to become authority, then later tooling patched around those pointers with scope checks, fallback guards, and recovery logic.

This spec records the required foundation repair: canonical Focusa authority must be rooted in an identifiable project source of truth and reconciled through CRDT-grade event/update semantics, not daemon-global singleton state.

---

## 2. Existing spec basis

This spec supersedes any implementation interpretation that treats global singleton active state as canonical authority.

It aligns with existing Focusa documents:

- `docs/40-instance-session-attachment-spec.md`
  - defines multiplexing engineers across multiple IDEs, terminals, tmux panes, harnesses
  - defines Instances, Sessions, Attachments
  - states one engineer may have many projects
  - states many instances may attach to the same Thread
- `docs/41-proposal-resolution-engine.md`
  - defines timestamped async concurrency across multiple Instances and Sessions without locks
  - preserves alternatives for audit, explanation, and forking
  - resolves decisional proposals into canonical outcomes
- `docs/43-multi-device-sync.md`
  - defines local-first bidirectional sync
  - requires deterministic inspectable behavior
  - requires no silent merges
  - requires explicit matching via `workspace_id` and `repo_signature`
- `crates/focusa-core/src/sync/crdt.rs`
  - defines CRDT-based multi-device sync
  - includes vector clocks, Lamport ordering, CRDT events, CRDT log merge, and deterministic conflict resolution

The intent is not “CRDT-ish”. The intended class is CRDT-backed multi-session, multi-device reconciliation with integrity.

---

## 3. Correct foundation

### 3.1 Canonical source of truth

The canonical source of truth is the verified project root.

A project root is the stable container for a project and must be verified by multiple independent signals when possible:

- explicit `.focusa-project.json`
- git remote URL
- repo root fingerprint
- default branch / workspace manifest
- Beads prefix and `.beads/` location
- package/workspace markers
- deployment/live-root evidence when applicable
- operator confirmation when automatic confidence is insufficient

A session may observe or attach to a project root. A session does not define project identity by itself.

### 3.2 Root authority key

Canonical state is addressed under a root authority key:

```text
ProjectRootKey = verified_project_root + project_fingerprint
```

Where logical work is distinct inside one root, add continuity:

```text
WorkstreamKey = ProjectRootKey + continuity_id
```

Where runtime attachment matters, add session metadata:

```text
AttachmentKey = WorkstreamKey + instance_id + session_id + attachment_id
```

Authority order:

1. `ProjectRootKey` — project source of truth
2. `WorkstreamKey` — logical concurrent workstream under the root
3. `AttachmentKey` — runtime session/instance binding

`session_id` is temporal metadata. It is never the root authority by itself.

### 3.3 Multiplexing axiom

Multiplexing is always anchored to an identifiable source of truth.

No verified source of truth means no canonical Focusa read/write. Ambiguous root identity returns `project_root_required` or `scope_required`, not fallback state.

---

## 4. CRDT reconciliation contract

### 4.1 CRDT means update reconciliation

CRDT-backed Focusa state is not read-only observation storage.

CRDT as a class means:

- multi-session updates
- multi-device updates
- idempotent event ingestion
- causal ordering
- deterministic convergence
- conflict visibility
- integrity-preserving reconciliation

### 4.2 Event/update substrate

All canonical-capable changes enter as scoped operations/events under a verified `ProjectRootKey`.

Every event/update must include:

- `event_id`
- timestamp
- machine id
- instance id when applicable
- session id when applicable
- attachment id when applicable
- verified project root key
- continuity id when applicable
- target object
- operation kind
- causal metadata: vector clock and/or Lamport timestamp

### 4.3 Observations vs decisions

Observation events are mergeable and append-only.

Decisional updates are also real updates, not discarded or treated as read-only. When concurrent decisions target the same canonical object, they enter deterministic resolution:

- compatible operations merge
- conflicting operations become resolution candidates
- resolution preserves all alternatives
- accepted resolution materializes canonical state
- rejected/superseded alternatives remain inspectable timeline facts

### 4.4 No silent merge

No silent merge of cognitive state is allowed.

Convergence must be one of:

1. automatic deterministic merge for operations proven compatible
2. deterministic winner with explicit conflict record and citations
3. pending proposal/resolution window
4. operator-required decision

### 4.5 Spec 131 temporal reconciliation boundary

CRDT/Lamport/vector metadata establishes causal reconciliation order; it does not prove physical clock accuracy, elapsed duration, deadline satisfaction, or temporal authority. Spec 131 owns ClockSamplePairs, per-boot monotonic segments, civil/fixed deadline semantics, uncertainty, TemporalExecutionGuards, temporal claims, and completion/disposition truth.

Rules:

- physical timestamps remain observations with source/profile/uncertainty; last-write-wins physical time is not authority;
- compatible append-only temporal observations may converge, but conflicting deadline revisions, civil-time resolutions, clock corrections, guard revocations, closure facts, and operator dispositions require explicit deterministic conflict records or governed resolution;
- CRDT merge cannot bridge boot epochs into synthetic monotonic time, clear an external deadline, extend a lease/guard, activate an optional capability, or manufacture verified completion;
- canonical temporal mutations remain reducer/CAS/lease/fencing-owned; CRDT proof applies only to declared replicated/portable surfaces;
- merged records preserve source schema/policy/clock/calendar/estimator versions, causal refs, and all rejected/superseded alternatives for replay and settlement.

---

## 5. Forbidden foundation patterns

The following are not canonical authority:

- daemon-global active project
- daemon-global active session
- daemon-global active frame
- daemon-global active Workpoint
- daemon-global active Trajectory
- daemon-global current task
- daemon-global `lastProjectIdentity`
- daemon-global `lastTrajectoryClarity`
- daemon-global `activeWorkpointPacket`
- fallback from scoped query to global active state
- fallback from ambiguous cwd such as `/root` to a prior active project
- trajectory similarity as session merge authority
- session id as project identity

If any route/tool lacks verified root authority, it must return a blocked envelope rather than canonical content.

---

## 6. Required state model

### 6.1 Project registry

```rust
ProjectRegistry {
  projects: HashMap<ProjectRootKey, ProjectRecord>,
}

ProjectRecord {
  key: ProjectRootKey,
  root: PathBuf,
  fingerprint: String,
  identity_signals: Vec<ProjectIdentitySignal>,
  sessions: HashMap<SessionId, SessionRecord>,
  workstreams: HashMap<ContinuityId, WorkstreamState>,
  timeline: ProjectTimeline,
}
```

### 6.2 Workstream state

```rust
WorkstreamState {
  key: WorkstreamKey,
  focus_stack: FocusStackState,
  focus_state: FocusState,
  workpoints: WorkpointIndex,
  active_workpoint_id: Option<WorkpointId>,
  trajectories: TrajectoryIndex,
  active_trajectory_id: Option<TrajectoryId>,
  work_loop: WorkLoopState,
  timeline_head: TimelineHead,
}
```

Active/current pointers may exist only inside `WorkstreamState` or a narrower scoped state.

### 6.3 Project timeline

```rust
ProjectTimeline {
  events: CrdtLog<ProjectEvent>,
  materialized_heads: HashMap<WorkstreamKey, TimelineHead>,
  conflict_index: ConflictIndex,
}
```

All session changes under the same project root are timestamped, causally ordered, diffable, and inspectable.

---

## 7. Cascade by subsystem

### 7.1 focusa-core

- Introduce `ProjectRootKey`, `WorkstreamKey`, and `AttachmentKey`.
- Move singleton canonical state under project/workstream maps.
- Require root/workstream identity on canonical mutations.
- Materialize state by replaying/reconciling scoped CRDT operations.
- Reject unscoped canonical actions in the reducer.
- Preserve global daemon state only for process health, peer registry, and unowned telemetry.

### 7.2 API

- Every canonical route must accept or derive verified project root authority.
- Unscoped canonical routes return `project_root_required` / `scope_required`.
- `/status` may expose daemon health and known projects but must not imply one current project.
- Workpoint, Trajectory, Focus State, Focus Stack, evidence, Work-loop, prediction, metacog, and ontology routes must distinguish:
  - canonical scoped result
  - cross-project advisory result
  - blocked unscoped request

### 7.3 CLI

- CLI commands must operate against an explicit project root or selected local CLI profile.
- CLI profile selection is convenience only, not daemon authority.
- `focusa status` must separate daemon status from scoped project status.
- CLI must show candidate roots when ambiguous and block canonical writes until one root is selected/verified.

### 7.4 Pi tools

- Pi tools must carry `FocusaSessionIdentity` with verified project root and continuity.
- Pi extension caches must be keyed by `ProjectRootKey` / `WorkstreamKey`.
- Compaction/resume packets must include root key, continuity, and timeline head.
- Pi must not adopt daemon-global active Workpoint/Trajectory/Focus State.
- Ambiguous cwd must produce a vital-info prompt or blocked envelope.

### 7.5 Workpoint

- Workpoint checkpoints and resumes are under `WorkstreamKey`.
- Active Workpoint is scoped inside a workstream.
- Same high-level trajectory or similar mission never merges Workpoints across workstreams.

### 7.6 Trajectory

- Active Trajectory is scoped inside a workstream.
- Similarity groups are advisory clustering only.
- Trajectory views must identify project root, continuity, timeline head, and confidence.
- **HLT Ledger:** HLT changes are persisted to append-only JSONL ledger scoped by `(project_root, continuity_id)` with CRDT-grade Lamport timestamps. File: `{data_dir}/hlt-ledger/{project_root_hash}/hlt.jsonl`. This ensures the north-star is never lost and history is always recoverable.
- HLT changes via `trajectory_define_goal` atomically append to the ledger.

### 7.7 Focus Stack / Focus State

- Active frame is scoped inside a workstream or thread.
- Focus State slots are scoped.
- Cross-project operator views are labeled aggregations, not canonical current state.

### 7.8 Work-loop

- Current task and writer ownership are scoped by project/workstream or explicit loop id.
- Daemon may supervise multiple loops.
- No daemon-global current task is canonical.

### 7.9 Sync

- Sync must carry project root authority and causal metadata.
- Compatible operations reconcile.
- Conflicting decisions enter resolution, not read-only observation limbo.
- Remote updates from the same verified root can become canonical through reconciliation/resolution.
- Remote updates from unverified roots remain quarantined until matched.

---

## 8. Migration plan

### Phase 0 — Record and freeze unsafe assumptions

- Add this spec.
- Mark singleton active/current fields as legacy compatibility only.
- Add warnings on unscoped canonical routes.

### Phase 1 — Define identity keys

- Implement `ProjectRootKey`, `WorkstreamKey`, `AttachmentKey`.
- Add project registry and root verification quorum.
- Store keys in all new events.

### Phase 2 — Scope Workpoint and Trajectory

- Move active Workpoint and active Trajectory under `WorkstreamState`.
- Block unscoped resume/view/checkpoint.
- Update Pi packets to include timeline head and root key.

### Phase 3 — Scope Focus Stack and Focus State

- Move `active_id` under scoped workstream/thread state.
- Remove global active frame fallback from API and Pi injection.

### Phase 4 — Scope work-loop

- Move current task, writer, checkpoints, and loop status under project/workstream or explicit loop id.

### Phase 5 — Reconcile sync with CRDT foundation

- Upgrade sync payloads to include root authority and causal metadata.
- Replace observation-only handling for same-root compatible updates with reconciliation/resolution.
- Preserve observation quarantine for unverified or foreign roots.

### Phase 6 — Remove singleton fallbacks

- Delete or quarantine daemon-global active/current/last authority fields.
- Add hard tests proving unscoped canonical reads fail.

---

## 9. Required tests

### 9.1 Two-project bleed test

- Open project A and project B.
- Set active frame/workpoint/trajectory in both.
- Query project A and verify no project B state appears.
- Query without root and verify blocked `project_root_required`.

### 9.2 Same-root multi-session timeline test

- Open two sessions against the same verified root.
- Emit changes in both sessions.
- Verify one project timeline includes both sessions ordered by causal timestamp.
- Verify diffs can be grouped by session.

### 9.3 Concurrent compatible updates test

- Two sessions update compatible append-only targets.
- Verify CRDT merge converges without conflict.

### 9.4 Concurrent conflicting decisions test

- Two sessions mutate the same focus/trajectory/workpoint target incompatibly.
- Verify conflict record or PRE resolution window.
- Verify no silent overwrite.

### 9.5 Ambiguous cwd test

- Start Pi from `/root` or another broad directory.
- Verify no prior project is adopted as canonical.
- Verify candidate roots are shown and canonical writes block.

### 9.6 Compaction/resume test

- Compact while multiple projects and sessions are active.
- Resume one workstream.
- Verify only matching root+continuity+timeline head is injected.

### 9.7 Multi-device same-root sync test

- Two daemons sync same verified project root.
- Verify causal event import, deterministic reconciliation, and explicit conflict handling.

---

## 10. Acceptance criteria

This foundation is accepted when:

- no canonical Focusa route can return current/active state without verified root authority
- multiple sessions can attach to the same root and produce diffable timeline changes
- multiple projects can remain active without bleed
- CRDT event/update reconciliation is used for same-root multi-session/multi-device updates
- conflicting cognitive updates are resolved explicitly, never silently merged
- Pi compaction/resume cannot inject stale cross-project state
- singleton active/current/last fields are removed or proven non-authoritative

---

## 11. Summary

Focusa multiplexing must be based on the identifiable source of truth: the verified project root.

Sessions are runtime timelines under that root. Continuities are logical workstreams under that root. CRDT-backed operations reconcile concurrent updates with integrity. Canonical cognitive state is materialized from scoped, reconciled operations and explicit resolutions.

Global singleton state is incompatible with this foundation.

---

## 12. Non-CRDT early-spec divergence pass

This section records early implementation veers that are independent of the CRDT/reconciliation issue. They come from the earliest Focusa docs and core reducer contract.

### 12.1 Conversation state entered `FocusaState`

Early intent:

- `docs/core-reducer.md` says conversation history is never part of `FocusaState`.
- Global invariant: conversation never mutates cognition.
- Reducer guarantee: if a cognition change cannot be expressed as a reducer event, it does not belong in Focusa.

Implementation veer:

- `ActiveTurn` in `crates/focusa-core/src/types.rs` stores `raw_user_input` and `assembled_prompt` inside `FocusaState`.
- `TurnStarted` in `crates/focusa-core/src/reducer.rs` writes `raw_user_input` into `state.active_turn`.
- `routes/turn.rs` and `routes/proxy.rs` directly mutate `active_turn.assembled_prompt` during assembly/streaming.

Correction:

- Runtime turn buffers must be outside canonical cognition state, or represented as event-log/runtime telemetry only.
- Prompt/turn content must not become canonical Focusa state authority.

### 12.2 Reducer single-writer rule was weakened by direct API mutations

Early intent:

- `docs/core-reducer.md` defines a single-writer cognitive reducer.
- All mutable cognition transitions must be expressed through reducer events.
- Reducer is deterministic, replayable, crash-safe, and free of side effects.

Implementation veer:

- `routes/turn.rs` directly writes `focusa.active_turn.*` and calls `state.mark_external_mutation()`.
- `routes/proxy.rs` directly runs `focusa_core::memory::semantic::resolve_contradictions(&mut focusa.memory)` outside reducer application.
- `routes/proxy.rs` also directly writes assembled prompt state before emitting a telemetry event.

Correction:

- Any memory maintenance, active-turn mutation, prompt assembly state, or contradiction resolution must be represented as reducer event, runtime-only noncanonical cache, or explicit telemetry—not direct canonical state mutation.

### 12.3 Beads authority degraded to non-empty string checks

Early intent:

- `docs/00-glossary.md` says Beads is authoritative task and long-term intent memory.
- `docs/03-focus-stack.md` says frame push validates Beads issue and frames without Beads linkage are forbidden.
- `docs/bootstrap-prompt-rust.md` says if work is not tracked in Beads, it does not exist.

Implementation veer:

- `routes/focus.rs` rejects empty `beads_issue_id`, but does not validate the issue exists in Beads.
- `routes/commands.rs` similarly validates non-empty Beads id only.
- `routes/proposals.rs` defaults focus-change proposal Beads id to the literal string `proposal-focus-change`.
- Trajectory tests/synthetic helpers create `beads_issue_id: format!("issue-{title}")`, showing synthetic IDs became acceptable in implementation/test culture.

Correction:

- Focus frame creation must validate Beads existence or explicitly mark the frame as noncanonical/proposal/demo.
- Synthetic Beads ids must not satisfy canonical Focus Stack invariants.

### 12.4 Focus Gate became peripheral instead of the required cognitive gate

Early intent:

- `docs/01-architecture-overview.md` says no subsystem may bypass Focus Gate or Focus Stack.
- `docs/04-focus-gate.md` says Focus Gate is advisory, never mutates focus, and only surfaces candidates.
- `docs/03-focus-stack.md` says Focus Gate may surface candidates related to inactive frames and never auto-resumes frames.

Implementation veer:

- Many API routes directly mutate Focus Stack/Focus State via reducer without requiring a Focus Gate candidate/proposal path.
- Workpoint/Trajectory/Work-loop routes developed their own authority, drift, and next-action logic outside the original Focus Gate candidate model.
- The current Attention Recall / Workpoint authority system is useful, but it is a parallel control plane rather than the original Focus Gate path.

Correction:

- Decide explicitly: either retire Focus Gate as the central gate, or restore it as the canonical candidate/proposal front door.
- No route should silently invent a separate cognitive authority path without a documented replacement for Focus Gate’s original role.

### 12.5 Expression Engine drifted from deterministic expression into dynamic cognition assembly

Early intent:

- `docs/08-expression-engine.md` says Expression Engine governs what is said now, not what is known.
- It forbids reasoning/planning, implicit summarization, dynamic prompt shaping, content inference, and memory mutation.
- Truncation must be explicit, logged, and reversible.

Implementation veer:

- `crates/focusa-core/src/expression/engine.rs` adds constitution context, thread thesis, parent context, handle rehydration, and degradation cascades beyond the original simple slot contract.
- `routes/proxy.rs` performs memory contradiction resolution and pre-turn external context injection adjacent to prompt assembly.
- Prompt assembly writes assembled prompt back into state through API paths.

Correction:

- Split deterministic expression from retrieval, memory maintenance, contradiction resolution, and adaptive prompt planning.
- Expression output may reference prepared inputs, but must not be the subsystem that changes what Focusa knows.

### 12.6 Reference Store contract is partially implemented but not fully enforced at route ergonomics

Early intent:

- `docs/07-reference-store.md` says artifacts are lossless, immutable, session-scoped by default, rehydrated only intentionally, and never automatically injected.

Implementation veer:

- Reducer enforces handle-id immutability, but route-level storage polls by label and returns the first matching handle, creating ambiguity for duplicate labels.
- Event/persistence schema does not make session/project/root scoping first-class for artifact handles.
- Many later ontology and prompt routes inspect reference handles globally, making cross-session/project artifact posture depend on route filters rather than storage shape.

Correction:

- Artifact identity must include project/session/root scope.
- Store routes should return the exact handle created by the write operation, not discover by label.
- Rehydration and prompt inclusion must remain explicit, scoped, and auditable.

### 12.7 Early implementation conflated runtime correlation with cognition state

Early intent:

- `docs/02-runtime-daemon.md` separates session validation, gate, stack, state update, expression, model invocation, and persistence/events.
- `docs/core-reducer.md` keeps conversation outside `FocusaState`.

Implementation veer:

- `active_turn`, assembled prompts, token/stream chunks, memory contradiction cleanup, and CLT interaction append are interwoven with the same state object used for cognition.

Correction:

- Separate three planes:
  1. canonical cognition state,
  2. runtime turn/session correlation,
  3. telemetry/history/event log.
- Only plane 1 participates in Focus State authority.

---

## 13. Core-system deep audit matrix

This section broadens §12 beyond the first obvious breaches and covers the core Focusa systems as a whole. The point is not that every later feature is wrong; the point is that every later feature must be placed into the original authority model or explicitly supersede it.

### 13.1 Canonical state shape vs accumulated subsystem state

Earliest contract:

- `docs/core-reducer.md` defines `FocusaState` as session, Focus Stack, Focus Gate, Reference Index, Memory, and version.
- `docs/02-runtime-daemon.md` says the daemon is the single authoritative execution context, but not an agent/planner/orchestrator.
- `docs/01-architecture-overview.md` defines planes: cognitive control, context fidelity, memory, background cognition, and interfaces.

Implementation veer:

- `crates/focusa-core/src/types.rs` expanded `FocusaState` to include CLT, UXP/UFI, autonomy, constitution, telemetry, RFM, PRE, ontology, Workpoint, Trajectory, contribution, Work-loop, instances, attachments, threads, active turn, and anticipated context.
- Some additions are legitimate architectural growth, but they are stored in the same canonical state object without a consistently documented distinction between authority state, read indexes, telemetry, runtime correlation, and export queues.

Correction:

- Reclassify every `FocusaState` field into one of: canonical cognition, canonical governance, append-only observation, read index/cache, runtime-only correlation, export/contribution queue, or telemetry.
- Only canonical cognition/governance fields should affect Focus State authority or replay-derived truth.

### 13.2 Event replay invariant vs state replacement after reducer dispatch

Earliest contract:

- `docs/G1-detail-15-events-observability.md` declares Focusa state must be reconstructible by replaying events in order.
- `docs/core-reducer.md` requires complete reducer-owned transitions.

Implementation veer:

- Many API routes call reducer and persist emitted events, then replace the whole in-memory state with `*state.focusa.write().await = new_state`.
- That pattern is acceptable only if every mutation is fully represented by persisted events and reducer output.
- Other routes still call `state.mark_external_mutation()` after direct writes, making version/freshness advance without replay-equivalent events.

Correction:

- Introduce an explicit classification for write paths:
  - reducer-backed canonical mutation,
  - observation append,
  - runtime cache mutation,
  - telemetry-only mutation.
- Ban `mark_external_mutation()` for canonical meaning unless a replayable event exists.

### 13.3 Session and thread isolation vs global singleton state

Earliest contract:

- `docs/02-runtime-daemon.md` says all state belongs to exactly one session and cross-session access is forbidden by default.
- `docs/17-context-lineage-tree.md` says Focus State references exactly one CLT node and CLT is not authority.
- `docs/40-instance-session-attachment-spec.md` and `docs/43-multi-device-sync.md` later make thread ownership explicit.

Implementation veer:

- `FocusaState` contains global vectors for instances, attachments, threads, Workpoints, Trajectories, telemetry, predictions, contribution queue, and reference handles.
- Many API views and route scans operate over global arrays and then filter by project/session/continuity as route logic rather than storage/key invariants.

Correction:

- Storage shape should encode session/thread/project ownership, not rely only on late route filtering.
- Cross-session/project reads should be explicit advisory projections, never default authority.

### 13.4 Focus Stack/Focus State vs Workpoint/Trajectory authority

Earliest contract:

- `docs/06-focus-state.md` says Focus State is the single source of truth for meaning.
- `docs/03-focus-stack.md` and `docs/core-reducer.md` make Focus Frame lifecycle the active attention authority.

Implementation veer:

- Workpoint and Trajectory now correctly preserve compaction/resume and project goal state, but they can appear more authoritative than Focus Stack/Focus State in practice.
- WorkpointResumePacket and Trajectory projections include action-authority, scope, next action, and do-not-drift semantics that the earliest Focus State spec did not model.

Correction:

- Define whether Workpoint/Trajectory are extensions of Focus State, sibling governance planes, or superseding authority surfaces.
- If they supersede early Focus Stack/Focus State responsibilities, update the early docs and reducer invariants to say so explicitly.

### 13.5 Focus Gate and Intuition Engine vs Attention Recall / Work-loop control

Earliest contract:

- `docs/04-focus-gate.md` and `docs/05-intuition-engine.md` say Gate/Intuition only surface explainable candidates/signals and never act, inject prompt content, or mutate focus.

Implementation veer:

- Attention Recall verdicts, Work-loop writer state, scope conflict gates, and tool-output flood controls now influence whether action authority exists.
- These controls are valuable but are not simply Focus Gate candidates; they actively govern continuation.

Correction:

- Treat Attention Recall / Work-loop control as a separate governance plane, or route them through PRE/Gate as proposals with explicit acceptance.
- Avoid naming/mental-model collisions where advisory Gate signals are mistaken for action authority.

### 13.6 ASCC and Focus State vs Workpoint checkpoint packets

Earliest contract:

- `docs/G1-07-ascc.md` says ASCC is per-frame, anchor-turn based, and replaces linear chat history in prompts.
- `docs/06-focus-state.md` says Focus State survives compaction intact.

Implementation veer:

- Workpoint checkpoints have become the reliable compaction/resume artifact for current operations, while ASCC/Focus State frame checkpoints are not always the canonical continuation source.
- This is a practical improvement, but it means the original ASCC-first compaction model is incomplete.

Correction:

- Do not merge ASCC, Workpoint, CLT, or Trajectory Ladder into one subsystem.
- Define a handoff contract across separate systems: ASCC carries per-frame structured meaning, Workpoint carries resume/action continuity, CLT carries append-only lineage, and Trajectory Ladder carries advisory goal/route context.
- Post-compaction resume must state which system provides authority and which systems provide corroborating context.

### 13.7 CLT lineage vs conversation/raw content handling

Earliest contract:

- `docs/17-context-lineage-tree.md` says CLT nodes are append-only, immutable, store handles not raw text, and are not memory/focus/authority.

Implementation veer:

- Reducer appends CLT interaction records during `TurnCompleted`, while turn payloads and active-turn buffers can still contain assistant/user text in adjacent state flows.
- CLT is also used by later metacognition/lineage tooling as a reasoning surface, which risks being treated as more authoritative than intended.

Correction:

- CLT should record structural lineage and handles only.
- Any learning, decision, or next-action inference from CLT must be advisory until promoted through Focus State/Workpoint/PRE.

### 13.8 Memory model vs automatic maintenance and extraction

Earliest contract:

- `docs/G1-09-memory.md` says memory is opt-in, minimal, bounded, and worker extraction may only suggest memory via Focus Gate.
- Silent preference learning and speculative inference are non-goals.

Implementation veer:

- Semantic contradiction resolution runs directly in proxy pre-turn flow.
- Later memory optimization/RPC specs and ontology routes broaden memory access and extraction pressure beyond the original explicit-user-or-confirmed-candidate model.

Correction:

- Memory writes and cleanup must be explicit events with source, confidence, TTL, and user/operator confirmation posture.
- Automatic cleanup can be runtime maintenance, but it must not silently change canonical meaning.

### 13.9 Reference Store / ECS vs ontology and prompt global handle scans

Earliest contract:

- `docs/07-reference-store.md` says artifacts are never implicitly injected, handles contain no content, storage is session-scoped by default, and rehydration is explicit/auditable.
- `docs/17-context-lineage-tree.md` says raw content lives in Reference Store and CLT stores handles.

Implementation veer:

- ECS storage route returns a handle found by label after write.
- Ontology/prompt tooling scans global `reference_index.handles` slices, sometimes relying on route-level caps/filters.

Correction:

- Return exact handles from writes.
- Add project/session/thread scope to handle identity and indexes.
- Make any prompt inclusion/rehydration an auditable event tied to scope.

### 13.10 Expression Engine vs proxy/provider adaptation

Earliest contract:

- `docs/08-expression-engine.md` says deterministic expression only; no reasoning/planning/memory mutation.
- `docs/09-proxy-adapter.md` says adapters preserve original semantics and compatibility mutations are opt-in.

Implementation veer:

- Proxy flow performs pre-turn enrichment, contradiction cleanup, prompt assembly, provider forwarding, inline evaluation/regeneration, and active-turn state writes in one path.
- Strict proxy transparency and deterministic expression are therefore difficult to reason about.

Correction:

- Split proxy pipeline into named stages with explicit side-effect class:
  - transparent request normalization,
  - deterministic expression render,
  - optional retrieval/enrichment,
  - optional evaluation/regeneration,
  - runtime telemetry.
- Provider compatibility shims must remain opt-in and observable.

### 13.11 Telemetry and resource pruning vs canonical state freshness

Earliest contract:

- `docs/29-telemetry-spec.md` and event docs treat telemetry as observation, not cognition.
- Runtime docs forbid background tasks from blocking hot path.

Implementation veer:

- Telemetry routes and resource-mode pruning mutate `FocusaState` and increment version/freshness.
- Low-memory pruning removes oldest telemetry/context entries from the same state object that also contains canonical cognition.

Correction:

- Telemetry/resource buffers should not share canonical version semantics with cognition.
- Pruning observation buffers must not look like cognition mutation or affect Focus State replay authority.

### 13.12 Training/contribution vs cognition and privacy boundaries

Earliest contract:

- `docs/20-training-dataset-schema.md` says training data must represent cognition, not conversation.
- `docs/22-data-contribution.md` says ODCL is read-only with respect to cognition and never uploads raw conversations/prompts/private intent.

Implementation veer:

- Contribution enable/pause/approve/submit mutate `FocusaState.contribution` directly.
- Contribution queue lives alongside canonical cognition, making export policy state and cognition state easy to conflate.

Correction:

- Contribution policy/queue should be a separate local export subsystem or explicitly classified as governance state, not cognition.
- Eligibility must cite Focus State/CLT/ECS handles without importing raw prompt/turn content.

### 13.13 PRE/proposals vs direct route mutation

Earliest/later contract:

- `docs/41-proposal-resolution-engine.md` says decisional changes such as focus change, thesis update, autonomy adjustment, and constitution update are proposals resolved before canonical application.

Implementation veer:

- Direct focus/session/workpoint/trajectory route dispatch still applies reducer events immediately in many cases.
- PRE exists in state but is not uniformly used as the decisional front door.

Correction:

- Define which decisions require PRE windows and which are single-writer local commands exempt from PRE.
- Multi-instance or background-origin decisions should pass through PRE before reducer application.

### 13.14 Ontology/read indexes vs cognition source of truth

Earliest contract:

- Early docs distinguish Focus State, Memory, Reference Store, and Expression; they do not make ontology the source of truth.
- Later ontology specs legitimately add object/action/link intelligence.

Implementation veer:

- Ontology routes and read indexes now influence active-object resolution, next-action intelligence, cache metadata, and prompt context.
- Without a clear classification, ontology-derived suggestions can be mistaken for canonical task meaning.

Correction:

- Ontology should be classified as a read model/advisory intelligence layer unless an ontology event is promoted through reducer/PRE into canonical Focus State or Workpoint.

### 13.15 Work-loop/autonomy vs original non-orchestrator boundary

Earliest contract:

- `docs/02-runtime-daemon.md` says the daemon is not an agent, planner, or orchestrator.
- `docs/00-glossary.md` says Focusa is not an automation engine or scheduler.

Implementation veer:

- Continuous Work-loop, autonomy scoring, tool action contracts, and SilentSession workflows add orchestration-like behavior.
- This may be intentional evolution, but it supersedes the original non-orchestrator boundary.

Correction:

- Make the evolution explicit: either Focusa remains cognitive governance with external agents doing work, or Focusa includes a bounded orchestration plane with strict writer ownership and operator controls.
- Work-loop authority must be separated from cognition authority.

### 13.16 API/CLI route taxonomy is now required

Earliest contract:

- Runtime interfaces expose local API/CLI for status, stack, memory, events.
- State mutation rules were simple: reducer event or not allowed.

Implementation veer:

- Current API includes Focus, Session, Turn, Proxy, ECS, Memory, Telemetry, Ontology, Workpoint, Trajectory, Work-loop, Training, Predictions, Project, and Reflection routes.
- Routes differ in whether they are canonical, advisory, runtime-only, telemetry, export, read-model, or proposal-producing.

Correction:

- Add a route taxonomy table to the public docs and enforce it in code review/tests:
  - canonical mutation,
  - proposal mutation,
  - observation append,
  - runtime cache mutation,
  - telemetry/export mutation,
  - read-only projection.

---

## 14. Cascading-effects and agent-impact requirement

Architecture corrections must not be evaluated only as local purity fixes. Every correction must be scored for its project-wide cascade and for whether it helps or handicaps the agent operating inside Focusa.

### 14.1 Required cascade questions

For every authority-plane, route, state, or event decision, answer:

1. **Agent continuity:** Does this make post-compaction/session-resume easier or harder?
2. **Action authority:** Does the agent know what it is allowed to do next, or does the decision create ambiguity?
3. **Trajectory Ladder:** Does the decision preserve HLT/MLG/STG route context without making it false authority?
4. **Workpoint:** Does the decision preserve the concrete next-action contract and evidence hooks?
5. **Focus State / ASCC:** Does the decision preserve structured meaning without forcing raw conversation back into cognition?
6. **CLT:** Does the decision preserve lineage/history without making lineage decide current action?
7. **Prompt assembly:** Does the decision reduce prompt bloat and ambiguity, or does it remove useful context agents need?
8. **Tool routing:** Does the decision make the right next tool obvious, or does it force agents to rediscover state?
9. **Error recovery:** Does the decision provide bounded recovery paths for degraded/stale/noncanonical state?
10. **Testing:** Can the cascade be proven with route/state/event tests, not just prose?

### 14.2 Benefit / handicap matrix

Each correction should include a matrix like:

| Decision | Agent benefit | Agent handicap risk | Mitigation |
|---|---|---|---|
| Move `ActiveTurn` out of canonical cognition | Cleaner replay and less conversation bleed | Agent may lose immediate turn correlation if runtime cache is inaccessible | Runtime turn cache with explicit rehydrate handles and fallback telemetry |
| Make Trajectory Ladder advisory | Prevents route goals from overriding operator scope | Agent may underuse HLT/MLG/STG planning context | Workpoint packet includes trajectory refs as corroborating route context |
| Enforce real Beads IDs | Restores task authority | Demo/proposal flows may become harder | Explicit noncanonical proposal/demo frame class |
| Separate telemetry versioning | Prevents observation pruning from looking like cognition mutation | Agent may miss token/resource warnings | Telemetry remains queryable with clear noncanonical status |
| Keep ASCC, Workpoint, CLT separate | Avoids authority collapse | Agent may face too many packets | One handoff envelope names authority source and supporting refs |

### 14.3 Project-wide cascade map

The most important cascade is across these contracts:

```
Operator intent
  -> Beads task authority
  -> Focus Frame / Focus State meaning
  -> ASCC structured checkpoint
  -> Workpoint concrete continuation
  -> Trajectory Ladder route context
  -> CLT lineage/history refs
  -> Expression Engine prompt rendering
  -> Agent tool/action choice
  -> Evidence capture
  -> Reducer/event replay
```

A change at any point must document its downstream effects on the rest of the chain.

### 14.4 Agent-centered acceptance rule

A correction is incomplete if it makes the architecture cleaner but leaves a resumed agent with less clarity about:

- current mission,
- valid authority boundary,
- next action,
- relevant evidence,
- route/trajectory context,
- what not to treat as authority,
- degraded-state recovery.

The implementation target is not minimal state. The target is **bounded, explicit, replayable state that preserves agent usefulness without authority confusion**.

### 14.5 Propagation to implementation beads

The cascade requirement applies to every child bead under `focusa-877z`:

| Bead | Correction area | Required cascade focus |
|---|---|---|
| `focusa-877z.1` | ActiveTurn/runtime buffers | Preserve turn correlation for agents while removing prompt/conversation buffers from canonical cognition. |
| `focusa-877z.2` | Reducer/direct mutation | Replace direct canonical writes without hiding operational diagnostics agents need for recovery. |
| `focusa-877z.3` | Beads validation | Restore task authority while keeping proposal/demo/noncanonical flows usable and explicit. |
| `focusa-877z.4` | Gate / Workpoint / Trajectory / Attention Recall authority | Clarify action authority without overloading advisory salience. |
| `focusa-877z.5` | Expression Engine split | Keep deterministic expression while retaining useful prepared context for prompt assembly. |
| `focusa-877z.6` | Reference Store scope and exact handles | Prevent cross-session leakage while preserving easy evidence rehydration. |
| `focusa-877z.7` | Runtime / telemetry / cognition planes | Separate planes without making agents rediscover context after resume. |
| `focusa-877z.8` | Field/route/event authority taxonomy | Provide the shared worksheet for all benefit/handicap and downstream-dependency analysis. |
| `focusa-877z.9` | ASCC / Workpoint / CLT / Trajectory Ladder alignment | Preserve separate roles while minimizing resumed-agent cognitive load. |
| `focusa-877z.10` | Telemetry/resource/export versioning | Keep operational warnings visible but noncanonical. |
| `focusa-877z.11` | Proxy pipeline stages | Preserve adapter transparency while making enrichment/eval side effects observable. |
| `focusa-877z.12` | Ontology advisory promotion | Keep active-object intelligence useful without making read indexes false authority. |
| `focusa-877z.13` | Work-loop/autonomy orchestration plane | Separate orchestration writer ownership from cognition authority. |

Each bead must update its implementation plan with:

- decision under review,
- agent benefit,
- agent handicap risk,
- downstream systems affected,
- mitigation/handoff contract,
- verification tests.

---

## 15. All-surfaces foundational-change gate

Foundational authority-plane corrections affect every Focusa surface. No core correction may proceed as a daemon-only refactor. Each decision must be evaluated across daemon, API, CLI, Pi plugin, Focusa Pi tools, docs, tests, and migration.

### 15.1 Required surface impact matrix

Every implementation plan must include this matrix before code movement:

| Surface | Required questions |
|---|---|
| Daemon/core | What canonical state, reducer event, runtime cache, telemetry buffer, or read index changes? Does replay still reconstruct truth? |
| Persistence/events | What new event shape, migration, snapshot rule, or export/import behavior is required? |
| API | Which routes change response shape, status taxonomy, canonical/advisory flags, error classes, or retry behavior? |
| CLI | Which commands need new flags, output fields, degraded-state guidance, or migration warnings? |
| Pi plugin | How does the plugin discover authority, resume context, trajectory context, and degraded/noncanonical warnings? |
| Focusa Pi tools | Which `focusa_*` tools need schema/result-envelope updates, evidence linking, or retry guidance? |
| Workpoint/Trajectory tools | How are Workpoint authority and Trajectory Ladder route context presented without merging them? |
| Browser/UIAI tools | Are diagnostics/evidence/result envelopes still scoped and linkable after the change? |
| Docs/skills | Which skill docs, public docs, and examples need updated operator/agent guidance? |
| Tests/proof | Which unit, route, CLI, Pi-tool contract, and end-to-end resume tests prove the change? |
| Migration/backcompat | What old packets/routes/snapshots remain readable and how are stale/mixed-version states handled? |

### 15.2 Surface-specific acceptance rules

#### Daemon/core

- Reducer-backed canonical state remains replayable.
- Runtime-only buffers never become authority by accident.
- Resource pruning cannot mutate cognition truth.
- Work-loop/orchestration writer ownership stays separate from Focus State authority.

#### API

- Every route declares mutation class: canonical, proposal, observation, runtime cache, telemetry/export, read-only projection.
- Every response that can affect agent action carries canonical/advisory/degraded/stale flags.
- Scope fields are explicit: project_root, continuity_id, session_id, thread_id, frame_id, workpoint_id, trajectory_id where applicable.

#### CLI

- CLI output must make the same authority/degraded/advisory distinctions visible to humans.
- JSON CLI output must remain stable enough for Pi plugin/tools.
- Commands that mutate noncanonical telemetry/export state must not look like cognition updates.

#### Pi plugin and Focusa tools

- Post-compaction resume must remain simple: one Workpoint continuation contract with supporting ASCC, CLT, and Trajectory Ladder refs.
- Tool result envelopes must state canonical/degraded/status/failure_class/next_tools/evidence_refs.
- Tools must not force agents to infer authority from transcript tail, folder name, or route-specific quirks.

#### Docs and skills

- Skill docs must teach the new surface contract concisely.
- Public docs must explain roles without collapsing systems: Focus State/ASCC, Workpoint, CLT, Trajectory Ladder, Ontology, Telemetry, Work-loop.

### 15.3 “Perfect before code” rule

Before code movement on `focusa-877z.1-.13`, `focusa-877z.8` must produce an authority-plane and surface-impact worksheet that covers:

1. state fields,
2. reducer events,
3. API routes,
4. CLI commands,
5. Pi plugin calls,
6. Focusa Pi tool result envelopes,
7. docs/skills,
8. tests/proof,
9. migration/backcompat,
10. agent benefit/handicap.

No daemon-only fix is complete until the API, CLI, Pi plugin, and tools surface semantics are accounted for.

### 15.4 Menubar/UI surface gate

The Mac menubar app is a first-class Focusa surface. It is not a passive afterthought and not a new authority plane. It visualizes, explains, and safely controls existing authority planes.

#### Required menubar impact questions

Every foundational change must answer:

1. What should the menubar display as canonical vs advisory vs degraded/stale?
2. Which controls are safe from the UI, and which require confirmation?
3. Which state is read-only observation versus action-authority state?
4. How does the menubar help a human understand what the agent will do next?
5. What warning appears for scope mismatch, stale Workpoint, degraded daemon, or writer conflict?
6. Does the menubar use the same response envelope semantics as API/CLI/Pi tools?

#### Required menubar panels / affordances

| Area | Required display/control |
|---|---|
| Authority status | canonical/advisory/degraded/stale flags with source surface and timestamp |
| Active work | project_root, continuity_id, Focus Frame, Workpoint, Beads id, current mission |
| Resume | Workpoint next action, do-not-drift, evidence refs, copy handoff packet |
| Trajectory Ladder | HLT/MLG/STG, active gap, waypoint, explicitly marked advisory unless promoted |
| Focus State / ASCC | structured meaning per active frame, no raw conversation dump |
| CLT | lineage/head and compaction path as history only, not action authority |
| Work-loop | writer owner, paused/running/blocked, budget, safe pause/resume controls |
| Tool health | daemon, API, CLI, Pi plugin, Focusa tool contract, Beads direct/daemon status |
| Resource mode | normal/lowmem, token pressure, pruning warnings as noncanonical telemetry |
| Conflicts | scope conflict, stale packet, noncanonical packet, project mismatch, writer conflict |
| Evidence | latest verification refs and stable handles, explicit rehydrate/open actions |
| Contribution/privacy | opt-in status, queue count, raw prompt/conversation never-uploaded indicator |
| Recovery | suggested next tool/command for degraded state, copyable diagnostic packet |
| Docs/tasks | open bead, open docs/spec, open latest checkpoint, copy issue id |

#### Menubar non-negotiables

- The menubar must not invent authority from UI selection.
- UI controls that mutate canonical state must call the same API/CLI route taxonomy as every other surface.
- Advisory surfaces such as Trajectory Ladder, CLT, telemetry, and ontology must be visually labeled advisory unless promoted.
- It must be impossible for the UI to make stale/degraded state look canonical.
- Menubar copy/handoff actions must preserve `project_root`, `continuity_id`, `workpoint_id`, `trajectory_id`, and evidence refs.

---

## 16. Pi agent tools and UIAI browser integration gate

Pi agent tools and UIAI Engine browser workflows are critical Focusa surfaces. Foundational changes must preserve their contracts, because agents experience Focusa mostly through Pi tool result envelopes, guided commands, browser evidence packets, diagnostics intake, and post-compaction Workpoint recovery.

### 16.1 Server-observed UIAI browser baseline

Local server inspection found:

- UIAI Engine target: `http://localhost:7456` / `http://127.0.0.1:7456`.
- Browser health endpoint reports `service=uiai-browser`, `status=standby`, `browser_state=idle-off`, `diagnostics_enabled=true`, `eval_async_enabled=true`, `max_pages=2`, queue depth `0`, stored error count visible, and `agent_pressure` metadata.
- UIAI agent card says agents should pass `focusa_scope` to `browser_open`, compose packets with `/api/agent/research-packet`, and ingest diagnostics through `focusa_browser_diagnostics_intake`.
- UIAI docs define `uiai.focusa_research_diagnostics_packet.v1` as the packet schema and expose evidence refs for diagnostics, search, browser read/snapshot, error, screenshot, and share artifacts.
- Focusa already has `focusa_browser_diagnostics_intake`, `focusa_tool_doctor` UIAI browser health awareness, visual workflow evidence routes, and Pi extension tool contracts for browser diagnostics intake.

### 16.2 UIAI impact questions for every foundational change

Every correction must answer:

1. Does `focusa_scope` still carry `project_root`, `continuity_id`, `workpoint_id`, and `evidence_ref` without becoming authority inside UIAI?
2. Do UIAI diagnostics/session errors/search/read/screenshot/share outputs still produce stable evidence handles instead of blobs?
3. Can `focusa_browser_diagnostics_intake` still infer scope and link bounded Workpoint evidence?
4. Does `focusa_tool_doctor` still report UIAI browser status/pressure and recommend narrowed workflows under pressure?
5. Do ResearchDiagnosticsPackets remain usable from Pi TUI, Pi RPC/JSON, MCP, HTTP, and CLI?
6. Do browser failures still route diagnostics-first before code patches or visual guesses?
7. Do visual workflow evidence routes avoid label ambiguity and cross-session leakage after Reference Store changes?
8. Does UIAI evidence stay advisory/proof until captured into Focusa Workpoint/evidence, not canonical truth by itself?
9. Do LowMem/resource-mode changes preserve bounded diagnostics and packet summaries?
10. Do tests prove packet parity, diagnostics redaction, scope echo, and Focusa evidence handoff?

### 16.3 Pi agent tool impact questions

Every correction must also answer:

1. Which `focusa_*` Pi tools change schema, result envelope, retry posture, or next-tools recommendations?
2. Which UIAI Pi tools (`pi_uiai_agent_card`, `uiai_health`, `uiai_browser_*`, packet builders) need updated descriptions or routing hints?
3. Does the Pi extension still render compact-by-default and expandable JSON without tool-output flood?
4. Does compaction/resume still inject a Workpoint continuation packet with supporting Trajectory Ladder and UIAI evidence refs?
5. Can Pi RPC/JSON mode use the same packet/tool schemas without relying on TUI-only widgets?
6. Are tool contracts in `apps/pi-extension/src/tool-contracts.ts` and docs in `docs/focusa-tools/` updated together?
7. Do static/live tool-contract checks catch drift after daemon/API changes?
8. Does the Pi plugin remain thin UX glue and avoid parallel memory or hidden cognitive writes?

### 16.4 Required UIAI/Pi proof matrix

| Surface | Required proof |
|---|---|
| UIAI health | `uiai_health` / `/api/health/browser` returns status, pressure, diagnostics availability, and packet recommendation; uiai_health is infrastructure-only telemetry, must not seed project_root or continuity_id, and project truth comes from ProjectIdentity plus Workpoint/Trajectory scope. |
| UIAI tool graph | `/api/tools/graph` still advertises Focusa scope echo, evidence refs, packet schema, and preferred Focusa tools. |
| UIAI diagnostics | `browser_diagnostics` returns bounded redacted console/exception/network evidence and echoes `focusa_scope` when present; focusa_scope is echo/provenance metadata, not authority. |
| UIAI packet | `/api/agent/research-packet` and Pi packet builder produce `uiai.focusa_research_diagnostics_packet.v1`. |
| Focusa intake | `focusa_browser_diagnostics_intake` converts diagnostics/failure envelopes into Workpoint evidence, active-object hints, prediction, and optional metacog. |
| Pi tool contracts | Static Pi registry and live daemon contract surfaces agree on tool names, families, next tools, and failure taxonomy. |
| Visual evidence | `/v1/visual-workflow/evidence/store` returns exact scoped handle semantics, not label-ambiguous global references. |
| Browser reliability | UIAI stress/soak reports include `focusa_evidence` packets and remain bounded/redacted. |
| Compaction resume | Workpoint resume can carry UIAI evidence refs and next browser/source action without transcript tail. |

### 16.5 UIAI-specific cascade risks

| Decision | Agent benefit | Handicap risk | Mitigation |
|---|---|---|---|
| Move runtime buffers out of canonical state | Cleaner cognition replay | Browser session/diagnostics correlation may be lost | Preserve UIAI session IDs and evidence refs in Workpoint evidence, not raw turn buffers |
| Scope Reference Store handles | Prevents cross-project/browser evidence leakage | Agents may fail to rehydrate old UIAI artifacts | Provide explicit project/session/workpoint scope and migration for old `uiai-*` refs |
| Make ontology advisory | Avoids false active-object truth | Browser URL/stack to source-file hints may feel weaker | Promote verified UIAI diagnostics through evidence + active-object resolution |
| Separate telemetry versioning | UIAI pressure/errors stay noncanonical | Agents may ignore browser pressure | `focusa_tool_doctor` and Pi tools must surface UIAI pressure as operational warning |
| Split proxy/expression stages | Cleaner side effects | Browser/research packets may not enter prompts automatically | Workpoint packet carries evidence refs and next action; expression renders bounded refs only |

### 16.6 Non-negotiables for UIAI/Pi

- UIAI is browser/search/diagnostics/proof execution, not Focusa cognition authority.
- Focusa owns ProjectIdentity, Workpoint, Trajectory, evidence, prediction, metacog, and resume authority.
- Pi owns operator/agent UX and tool selection, not parallel memory.
- UIAI outputs are evidence proposals until captured/linked in Focusa.
- Browser diagnostics are required before patching when page/action/API failure is suspected.
- Raw screenshots, HAR-like data, cookies, auth headers, full page bodies, and raw SERP blobs must not enter Focusa by default.

### 16.7 Deeper Pi/UIAI findings from server inspection

A second pass found several concrete cascade risks that must be handled before foundational code movement.

#### 16.7.1 UIAI guided Pi workflow currently hardcodes Focusa scope

UIAI Engine local Pi extension (`/home/wpuiai/uiai-engine/.pi/extensions/uiai-engine.ts`) builds guided packet workflows with hardcoded scope:

- `project_root: "/home/wpuiai/uiai-engine"`
- `continuity_id: "focusa-cont-uiai-engine-..."`
- `evidence_ref: "pi-uiai-<mode>-packet"`

Impact:

- This is correct only when operating inside the UIAI Engine project.
- If copied mentally into Focusa workflows, it creates cross-project scope bleed.
- Focusa foundational changes must require Pi/UIAI packet workflows to receive canonical Focusa scope from `focusa_workpoint_resume` / `focusa_project_identity`, or mark packet `scope_status=missing|partial|mismatch_candidate`.

Required correction:

- UIAI guided Pi workflows must treat hardcoded scope as project-local demo/default only.
- Cross-project use must pass operator/project-bound `focusa_scope` from Focusa, not infer from UIAI folder.
- Focusa tools must reject or degrade packets whose `focusa_scope.project_root` conflicts with active Workpoint/project identity.

#### 16.7.2 ResearchDiagnosticsPacket is a proposal envelope, not a Focusa tool envelope

UIAI packet schema carries `schema`, `mode`, `scope_status`, `evidence_refs`, `recommended_focusa`, `render`, and cleanup. It does not currently carry the full Focusa `tool_result_v1` status set: `canonical`, `advisory`, `degraded`, `stale`, `failure_class`, and `retry`.

Impact:

- Agents may mistake a packet for durable Focusa evidence because it has `recommended_focusa.args_preview`.
- Packet `scope_status=present` means UIAI received scope metadata, not that Focusa accepted authority.

Required correction:

- Spec wording and Pi rendering must say: UIAI packet = bounded evidence proposal.
- Durable authority starts only after `focusa_evidence_capture`, `focusa_browser_diagnostics_intake`, or `focusa_workpoint_link_evidence` succeeds.
- Packet renderers should display `proposal_only` / `not_canonical_until_captured` or equivalent.

#### 16.7.3 `focusa_browser_diagnostics_intake` is Pi-only composite but sits on critical path

Focusa tool contract registry marks `focusa_browser_diagnostics_intake` as `parity_status=pi_only`. It is the preferred route for browser failure envelopes, but no equivalent first-class API/CLI route exists.

Impact:

- Pi agents get the best browser evidence path.
- CLI/MCP/headless workflows must use packet args preview plus `focusa_evidence_capture` or multiple lower-level calls.
- Foundational API/CLI parity work must decide whether Pi-only remains acceptable or whether a daemon/API composite route is needed.

Required correction:

- `focusa-877z.8/.14` taxonomy must mark `focusa_browser_diagnostics_intake` as Pi composite critical path.
- Headless parity must either expose a daemon/API composite intake or document exact lower-level fallback choreography.

#### 16.7.4 Visual workflow evidence route has the same label-discovery ambiguity as ECS

`crates/focusa-api/src/routes/visual_workflow.rs` stores visual evidence by dispatching `Action::StoreArtifact`, then polls `reference_index.handles` by generated label and returns the first matching handle.

Impact:

- Duplicate labels or concurrent visual evidence writes can return the wrong handle.
- This affects UIAI screenshot/share/visual proof flows and any future Menubar visual evidence panel.

Required correction:

- Visual evidence store must return the exact handle created by the store operation or use idempotency keys/unique write correlation.
- Visual evidence should carry project/session/workpoint scope, not just `run_id`, `phase`, and label.

#### 16.7.5 UIAI health pressure and Focusa resource mode need shared semantics

UIAI health exposes browser pressure, page pool, queue, cache, errors, and recommended actions. Focusa tool doctor consumes this as `uiai_browser=<status>/<pressure>`.

Impact:

- If Focusa resource-mode semantics change, UIAI pressure warnings may become detached from agent decision-making.
- Agents need one combined view: Focusa resource mode + UIAI browser pressure + Workpoint/Trajectory context.

Required correction:

- `focusa_tool_doctor`, Menubar, CLI, and Pi tool renderers must present UIAI pressure as noncanonical operational telemetry that can block/narrow browser workflows without changing cognition truth.

#### 16.7.6 Packet compact rendering prevents flood but can hide capture status

UIAI Pi extension renders compact summaries and expandable JSON, which prevents tool-output flood. However, compact packet lines may show evidence count/tool/next action without making durable-capture status explicit.

Impact:

- Agents may continue as if evidence was captured when only a packet proposal exists.

Required correction:

- Compact renders must distinguish:
  - packet composed,
  - Focusa capture pending,
  - Focusa capture accepted,
  - degraded/missing scope,
  - cleanup required.

#### 16.7.7 Release proof must be cross-repo

UIAI has strong agent-surface proof gates: packet drift, Pi extension registration/rendering, MCP route smokes, browser diagnostics, failed-network diagnostics, Focusa error evidence, and release-browser-reliability.

Impact:

- Focusa foundational changes that alter evidence, scope, or tool envelope semantics can break UIAI without changing UIAI code.

Required correction:

- Focusa foundational PRs touching Pi tools/UIAI contracts must cite both Focusa proof and UIAI proof commands, at least:
  - Focusa Pi tool contract/static checks,
  - UIAI `scripts/check-tool-parity.sh`,
  - UIAI `scripts/smoke-pi-rendering.sh`,
  - UIAI packet drift/smoke,
  - UIAI browser diagnostics smoke when diagnostics contracts change.

---

## 17. Opinionated defaults with explicit configurability

The correction program uses opinionated defaults so the safe path is the shortest path. Configurability exists for expert workflows, but defaults must preserve authority boundaries, agent continuity, scope safety, and cross-surface parity without requiring agents to remember the architecture.

### 17.1 Default authority posture

| Situation / object | Default classification | Default behavior |
|---|---|---|
| Missing `project_root` or `continuity_id` | degraded/advisory | Do not perform canonical linkage; recommend ProjectIdentity/Workpoint resume. |
| UIAI ResearchDiagnosticsPacket | proposal-only evidence bundle | Render as not durable until Focusa capture/intake/link succeeds. |
| UIAI `focusa_scope` | metadata | Never authority by itself; validate against Focusa project identity/Workpoint. |
| WorkpointResumePacket canonical=true + scope match | continuation authority | Use for next action, do-not-drift, evidence refs, and resume handoff. |
| Trajectory Ladder | project/workstream north star | Persist HLT as durable strategic goal; refresh MLG/STG/waypoints from evidence; never override operator/project scope or Workpoint continuation authority. |
| CLT | lineage/history only | Use for ancestry, compaction path, and audit; never decide current action. |
| ASCC / Focus State | structured meaning | Preserve per-frame cognition; no raw conversation or prompt buffers. |
| Ontology/read indexes | advisory intelligence | Promote only via Workpoint/evidence/PRE/reducer path when verified. |
| Telemetry/resource/UIAI pressure | noncanonical operational signal | May narrow/block workflows; does not mutate cognition truth. |
| Raw logs/screenshots/HAR/page bodies/SERP blobs | external artifact only | Store handles/summaries, never inline by default. |
| Synthetic Beads IDs | noncanonical/proposal/demo | Cannot satisfy canonical Focus Frame invariants. |
| Direct API state write | forbidden for canonical meaning | Use reducer/PRE/Workpoint or runtime/telemetry class. |

### 17.2 Named policy profiles

Configurability uses named profiles, not arbitrary silent knobs.

| Profile | Intended use | Defaults |
|---|---|---|
| `safe_default` | normal agent/operator work | strict scope validation, compact envelopes, Workpoint continuation authority, Trajectory Ladder north-star context. |
| `builder` | active implementation | allows fast local workflows, but keeps canonical/advisory/degraded labels and proof requirements. |
| `audit_strict` | architecture/security/release reviews | requires all surface-impact fields, migration notes, and proof commands before mutation. |
| `lowmem` | context/resource pressure | narrows diagnostics/traversal, favors handles, degrades advisory payloads first. |
| `browser_debug` | UIAI/browser failure triage | diagnostics-first, `focusa_scope` required for canonical linking, UIAI pressure visible. |
| `headless_ci` | CI/RPC/MCP/CLI workflows | no TUI assumptions, packet schema parity, JSON envelopes, explicit cleanup. |
| `demo_noncanonical` | examples/proposals/training/demo | synthetic IDs allowed only with explicit noncanonical status and no authority promotion. |

### 17.3 Trajectory Ladder persistence policy

Trajectory Ladder is the project/workstream north star. It is not a task scheduler and it does not override operator steering, project scope, Beads task authority, or Workpoint immediate continuation authority.

Default persistence behavior:

- `HLT` / long-term goal is durable per `project_root + continuity_id` and auto-loaded on Pi restart/session continuation.
- The old manual `hlt-ledger.md` workflow is replaced by reducer-backed Trajectory records plus Workpoint/Trajectory resume injection.
- HLT is normally stable and should not be re-derived from every session's local activity.
- HLT update prompt threshold defaults to **7 resumed sessions** without operator confirmation.
- Earlier HLT update prompts are allowed only for explicit operator steering, project identity changes, or durable supersession evidence.
- `MLG`, `STG`, and waypoints are adaptive and may be inferred at session start and during work from recent file changes, git activity, Beads, Workpoint, ontology, evidence refs, Focus State/current focus, predictions, and metacog signals.
- Inferred MLG/STG/waypoints are proposal/advisory until accepted through the normal Workpoint/Trajectory/evidence path.

Default prompt wording after threshold:

> Your HLT has been stable for 7 resumed sessions. Would you like to keep it or update the project north star?

### 17.4 Advanced overrides

Advanced overrides are allowed only when they are explicit and inspectable.

Every override must include:

- profile/source that changed,
- changed field or route behavior,
- reason,
- affected surfaces,
- agent benefit,
- handicap risk,
- rollback/default restore path,
- proof command or manual acceptance gate.

Overrides must not silently erase:

- scope validation,
- canonical/advisory/degraded labels,
- evidence handle requirements,
- mutation class declarations,
- privacy/redaction boundaries,
- proof requirements.

### 17.5 Zero-friction implementation model

The goal is not more ceremony. The goal is automatic enforcement.

| Friction point | Zero-friction mechanism |
|---|---|
| Remembering authority classes | Machine-readable worksheet generates docs/tests. |
| Remembering result-envelope fields | Shared envelope schema/stubs for API/CLI/Pi/UIAI/menubar. |
| Remembering proof commands | `proof_commands` field and generated proof bundle. |
| Remembering docs updates | Docs generated from route/tool/field registry where possible. |
| Remembering migration notes | Lint fails if changed surface lacks migration/backcompat entry. |
| Agent reading huge matrices | Compact traffic-light output with expandable detail. |
| Browser evidence confusion | Packet render states proposal-only/capture-pending/captured. |
| Headless/TUI split | Same schema first; TUI only improves display. |

### 17.6 Master worksheet deliverable

`focusa-877z.8` must produce a machine-readable worksheet before code movement. Current seed: `docs/worksheets/focusa-877z.8-authority-taxonomy.yaml`.

Minimum schema:

```yaml
items:
  - id: focusa_state.active_turn
    object_kind: state_field
    current_surface: daemon_core
    authority_class: runtime_correlation
    default_profile: safe_default
    mutation_class: runtime_cache
    scope_fields: [project_root, continuity_id, session_id, turn_id]
    affected_surfaces: [daemon, api, cli, pi_plugin, focusa_tools, menubar, docs, tests]
    pi_tools: [focusa_workpoint_resume, focusa_browser_diagnostics_intake]
    uiai_tools: []
    agent_benefit: "removes prompt/conversation from canonical cognition"
    handicap_risk: "turn correlation diagnostics may be harder to retrieve"
    mitigation: "runtime cache plus evidence handles and telemetry refs"
    migration: "old snapshots with active_turn remain readable but nonauthority"
    compact_render_required_text: "runtime correlation only; not cognition authority"
    proof_commands: ["cargo test", "Pi tool contract smoke"]
```

Trajectory Ladder worksheet entries must include HLT prompt threshold, resumed-session counter source, HLT supersession policy, MLG/STG/waypoint inference inputs, and proof that `hlt-ledger.md` is no longer required for normal Pi continuation.

The worksheet is the source for:

- public docs tables,
- API route taxonomy,
- CLI/Pi/menubar display labels,
- tool-contract registry updates,
- proof bundles,
- migration/backcompat checklist.

### 17.7 Generated scaffolding target

Future implementation should add scaffold commands or scripts:

- `focusa taxonomy lint`
- `focusa taxonomy render-docs`
- `focusa new-route --authority-class ...`
- `focusa new-tool --family ...`
- `focusa proof surface <surface-or-item>`

Until those exist, `focusa-877z.8` must define the worksheet and initial lint/proof expectations manually.

---

## 18. Implementation readiness after worksheet expansion

`docs/worksheets/focusa-877z.8-authority-taxonomy.yaml` is now the implementation-ready seed for fields/routes/events/tools/surfaces. It covers the authority class, scope fields, affected surfaces, risks, mitigations, compact render text, and proof commands for TL/HLT, Workpoint, shared envelopes, Focus State/Stack, ProjectIdentity, Reference/Evidence, Ontology, UIAI, Telemetry, Menubar, policy profiles, side effects, and Work-loop.

Concrete implementation artifacts still to generate or wire from the worksheet:

1. **Shared result-envelope schema/stubs** used by daemon API, CLI, Pi tools, UIAI packets, and menubar.
2. **Migration/backcompat implementation** for old Workpoint packets, UIAI packets, snapshots, focus state records, and evidence handles.
3. **Policy profile registry implementation** for `safe_default`, `builder`, `audit_strict`, `lowmem`, `browser_debug`, `headless_ci`, and `demo_noncanonical`.
4. **Proof bundle map runner** linking changed surfaces to Focusa/UIAI/Pi/CLI/Menubar test commands.
5. **Menubar state contract implementation** mapping display panels to canonical/advisory/degraded/stale envelope fields.
6. **Headless fallback path** for Pi-only `focusa_browser_diagnostics_intake`.
7. **Exact-handle write correlation** for ECS and visual workflow evidence.
8. **Packet capture-status rendering** for UIAI Pi workflows.
9. **Cross-project scope guard** preventing UIAI local defaults from bleeding into Focusa project scope.
10. **Generated docs/lint path** so future route/tool additions cannot skip authority classification.
11. **Side-effect classification tests** proving telemetry/export/runtime-cache writes do not look like cognition mutation.

### 18.1 Expected side effects of the correction program

| Expected side effect | Positive outcome | Risk / mitigation |
|---|---|---|
| Stricter route/tool envelopes | Agents know authority, status, next tools, and recovery path | More fields; use defaults/scaffolds to avoid manual burden. |
| More degraded/advisory labels | Less false authority and cross-project bleed | Agents may hesitate; Workpoint packet should provide clear next action when canonical. |
| Migration of old packets/snapshots | Backcompat with explicit nonauthority classification | Old data may be demoted; expose migration warnings, not silent failure. |
| Pi/UIAI compact render changes | Less tool-output flood with clearer capture status | Must not hide evidence/capture state; make status explicit in first line. |
| Menubar/UI updates | Humans see same authority semantics as agents | UI must not create authority from selection; controls call same route taxonomy. |
| Headless parity work | CLI/MCP/RPC workflows no longer second-class | May require new composite API or documented choreography. |
| Exact-handle evidence writes | Safer concurrent visual/browser proof | Requires ECS/visual route changes and tests. |
| Proof burden increases | Foundational changes become safer across surfaces | Bundle commands and generate proof map to keep friction low. |

### 18.2 Completion criterion for the planning phase

The planning/spec phase is complete when `focusa-877z.8` has a reviewed worksheet and every `.8.1-.8.13` implementation child bead can point to:

- authority class,
- affected surfaces,
- defaults/profile behavior,
- agent benefit and handicap risk,
- migration/backcompat posture,
- proof commands,
- expected side effects.

Planning exit status: `planning_complete_with_guardrails`.

Critical guardrails before code implementation:

1. Render TL as split authority, not blanket advisory: HLT is durable north-star context; MLG/STG/waypoints are adaptive advisory; Workpoint remains immediate action authority.
2. Shared envelopes must carry provenance for prior-project TL fallback, including `fallback_prior_project_trajectory`, `fallback_source_continuity_id`, `fallback_provenance`, and `scope_status`.
3. The 7-resumed-session HLT prompt policy needs project/workstream lifecycle counter wiring before prompt suppression can be considered safe.
4. Compaction/resume packet work must include bounded timeouts or combined routes so Workpoint/TL/prediction/metacog/evidence calls do not degrade hot-path performance.
5. `implementation_ready_seed` means worksheet/planning ready only; generated schemas, route/tool wiring, migration, lint, and proof bundles remain implementation work.

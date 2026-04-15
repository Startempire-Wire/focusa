# Focusa Exhaustive Spec Audit

Status: active working document
Purpose: determine whether Focusa is actually functioning according to authoritative spec, or whether current behavior is only a partial/surface imitation.

## Governing frame

This audit is **not about code only**.
It is about whether **authoritative Focusa specs are actually implemented in code and runtime behavior**.

The chain under audit is:
1. authoritative spec
2. intended system behavior
3. implementation in code
4. actual runtime behavior

If any link in that chain breaks, then the system is not operating as the real Focusa as specced.

## Why this audit exists

If Focusa core is not operating according to spec, then downstream integrations are not using the real Focusa system as designed. This audit exists to verify spec implementation end-to-end, not just inspect code, UI, or CLI surfaces in isolation.

## Audit rule

Do not accept surface behavior alone.
For every suspected violation, trace all relevant layers:
1. authoritative spec
2. intended behavior implied by that authority
3. CLI behavior
4. API route behavior
5. daemon/action dispatch
6. reducer/core mutation
7. persistence/readback surface
8. integration/session impact

Each case should end with one of:
- Implemented correctly
- Partially implemented
- Implemented contrary to spec
- Missing implementation
- Ambiguous / spec underdefines

---

## Authority set

Primary specs for this audit:
- `docs/G1-13-cli.md`
- `docs/G1-detail-05-focus-stack-hec.md`
- `docs/G1-07-ascc.md`
- `docs/39-thread-lifecycle-spec.md`
- `docs/INTEGRATION_SPEC.md`
- `docs/G1-detail-08-ecs.md`
- `docs/G1-14-reflection-loop.md`
- `docs/21-data-export-cli.md`
- `docs/22-data-contribution.md`
- `docs/04-focus-gate.md`

Derived/integration docs may be consulted after authoritative docs, but do not override them.

---

## Current audit scope

This audit currently covers:
- full Focusa CLI command execution on an isolated daemon/data dir
- comparison against spec authority
- one-level-deeper source inspection in CLI/API/core for confirmed failures

This audit does **not yet** cover every persistence/read-model path exhaustively.
That is the next phase.

---

## Current high-confidence findings

### Confirmed core/API issues
1. Focus Stack lifecycle/invariant problems
   - completing a root frame can leave `active_id = None`
   - invalid resume/set flows can be accepted at API layer and only fail later in reducer/event processing
2. ECS retrieval problems
   - valid handles can be stored/listed/resolved
   - `ecs cat` and `ecs rehydrate` fail on valid handles
3. Export pipeline incompleteness
   - `export status` now reports export pipeline state rather than contribution/training queue payload
   - export dry-run commands now return structured, honest `not_implemented` payloads with surfaced spec flags
   - full dataset generation remains unimplemented
4. Cache bust command routing
   - repaired: CLI exposes `cache bust` and API command routing now accepts `cache.bust`
5. Session/frame coherence
   - repaired at focus mutation/readback boundary: closed or absent sessions reject focus writes and `/v1/status` suppresses active frame exposure

### Confirmed CLI / adapter issues
1. `--json` contract still needs cleanup in some commands, but major surfaced gaps repaired:
   - thread list/get/transfer/create/fork now honor `--json`
   - export dry-runs now honor `--json`
2. turns observability CLI/parser mismatch repaired for persisted PascalCase event types (`TurnStarted` / `TurnCompleted`)
3. reflection commands had aggressive CLI timeout; CLI timeout budget has been increased for reflection calls, but runtime-health verification still needed
4. Pi bridge frame naming still cwd-derived
5. Pi bridge live-frame vs summary-count surfaces disagree

---

## Findings transferred so far with authority + code citations

This section records findings already established in the audit, with direct doc and code citations. It is not yet the final exhaustive matrix; it is the current evidence ledger.

### F1. CLI `--json` contract is violated by thread commands
- Authority:
  - `docs/G1-13-cli.md:19-23` defines `--json` as a global flag.
  - `docs/G1-13-cli.md:98-100` requires `--json` to be exact machine-readable API passthrough.
- Code evidence:
  - `crates/focusa-cli/src/commands/threads.rs:64` takes `_json: bool`, then ignores it.
  - `crates/focusa-cli/src/commands/threads.rs:66-74` fetches `/v1/threads` and enters human rendering paths.
  - API thread endpoints do return structured JSON at `crates/focusa-api/src/routes/threads.rs:24-40` and `crates/focusa-api/src/routes/threads.rs:123-150`.
- Current classification:
  - **CLI contract bug**, not absence of core thread functionality.

### F2. Export surface is not implemented to authoritative export spec
- Authority:
  - `docs/21-data-export-cli.md:1-7` makes the export CLI authoritative, read-only, deterministic, auditable.
  - `docs/21-data-export-cli.md:10-20` defines real dataset export commands.
  - `docs/21-data-export-cli.md:70-90` defines multi-phase export behavior including extraction/validation.
- Code evidence:
  - `crates/focusa-cli/src/commands/export.rs:54-78` maps `export status` to `/v1/training/status` and prints contribution queue fields, not an export-pipeline status model.
  - `crates/focusa-cli/src/commands/export.rs:86-97` shows SFT dry-run and export paths are prose/TODO-level, including `// TODO: implement full export pipeline via API`.
  - `crates/focusa-cli/src/commands/export.rs:102-107` shows other dataset families are similarly prose-only.
- Current classification:
  - **spec functionality missing/incomplete**, not merely a UI formatting bug.

### F3. `cache bust` is surfaced in CLI but unsupported by command routing
- Authority:
  - `docs/G1-13-cli.md:6-10` requires scriptable, deterministic, JSON-friendly CLI behavior.
- Code evidence:
  - `crates/focusa-cli/src/commands/cache.rs:54-63` submits command `cache.bust` to `/v1/commands/submit`.
  - `crates/focusa-api/src/routes/commands.rs:224-266` contains command dispatch branches for gate, memory, session, and compact paths, but no `cache.bust` branch.
- Runtime evidence:
  - isolated audit returned: `Unknown or disallowed command: cache.bust`.
- Current classification:
  - **API command-router incompleteness**, not just CLI rendering.

### F4. Focus Stack root completion violates HEC invariants
- Authority:
  - `docs/G1-detail-05-focus-stack-hec.md:72-78` defines `FocusStackState` with `root_id` and `active_id` as state fields, not optional no-focus steady state.
  - `docs/G1-detail-05-focus-stack-hec.md:97-118` defines `PopFrame`/`SetActiveFrame` semantics: parent becomes active; target selection is constrained to current active path.
- Code evidence:
  - `crates/focusa-core/src/reducer.rs:785-794` sets `stack.active_id = None` and `stack.root_id = None` when completing a frame with no parent.
- Current classification:
  - **core Focus Stack implementation contrary to spec**.

### F5. Focus frame set/resume validation occurs too late
- Authority:
  - `docs/G1-detail-05-focus-stack-hec.md:112-118` says `SetActiveFrame` is only allowed for a target in the current active path.
- Code evidence:
  - `crates/focusa-api/src/routes/focus.rs:95-103` accepts `set-active` requests and returns `{"status":"accepted"}` immediately.
  - `crates/focusa-core/src/runtime/daemon.rs:2351-2353` converts `Action::SetActiveFrame` directly into `FocusaEvent::FocusFrameResumed`.
  - `crates/focusa-core/src/reducer.rs:825-842` only later rejects resume if the target frame is not `Paused`.
- Runtime evidence:
  - isolated audit hit `Invalid event for current state: FocusFrameResumed: frame ... is Completed, not Paused` after earlier acceptance.
- Current classification:
  - **API/core lifecycle contract bug**; invalid transitions are acknowledged before validation.

### F6. Procedural memory reinforce accepts invalid rule IDs at API surface
- Authority:
  - `docs/G1-13-cli.md:6-10` requires deterministic/scriptable behavior; invalid IDs should not silently succeed.
- Code evidence:
  - `crates/focusa-api/src/routes/memory.rs:68-76` accepts reinforce requests and enqueues `Action::ReinforceRule`.
  - `crates/focusa-core/src/memory/procedural.rs:12-24` returns `None` when the rule ID is not found.
  - `crates/focusa-core/src/runtime/daemon.rs:2451-2458` converts that not-found case into `Ok(vec![])`, i.e. no event and no error.
- Runtime evidence:
  - audit confirmed `memory reinforce --json bogus-rule` was accepted.
- Current classification:
  - **API/action-result contract bug**; core knows the rule is missing, but the API reports success semantics anyway.

### F7. ECS retrieval contract exists on paper and in routing, but runtime retrieval is broken
- Authority:
  - `docs/G1-detail-08-ecs.md:77-88` defines handle resolution/content access as part of ECS.
  - `docs/G1-detail-08-ecs.md:97-106` defines explicit rehydration via `focusa ecs rehydrate <id> --max-tokens N`.
  - `docs/G1-13-cli.md:74-79` includes `ecs cat`, `ecs meta`, `ecs rehydrate` in the CLI contract.
- Code evidence:
  - `crates/focusa-api/src/routes/ecs.rs:101-160` implements `GET /v1/ecs/content/:handle_id`.
  - `crates/focusa-api/src/routes/ecs.rs:203-208` registers ECS routes including content/rehydrate.
  - `crates/focusa-cli/src/commands/ecs.rs:108-140` calls those endpoints for `cat` and `rehydrate`.
- Runtime evidence:
  - isolated audit saw 404s for valid-handle `ecs cat` and `ecs rehydrate`.
- Current classification:
  - **underlying ECS retrieval path broken**, not merely absent CLI wiring.

### F8. Reflection functionality exists, but CLI timeout likely makes healthy paths appear broken
- Authority:
  - `docs/G1-14-reflection-loop.md:49-62` defines `/v1/reflect/run`, history, status, and the CLI contract.
- Code evidence:
  - `crates/focusa-cli/src/api_client.rs:11-13` sets a default CLI API timeout of 2 seconds.
  - reflection routes and tests exist in `crates/focusa-api/src/routes/reflection.rs` (route file already inspected; detailed pass still pending).
- Runtime evidence:
  - isolated audit found `reflect run` and scheduler tick timed out, while other reflection/status surfaces worked.
- Current classification:
  - **likely CLI/runtime timeout issue**, pending deeper pass before declaring a core failure.

### F9. Pi integration still derives frame title/goal from cwd fallback instead of the active unit of work
- Authority:
  - `docs/G1-detail-05-focus-stack-hec.md:58-65` defines frame records around human-readable `title`, `goal`, `constraints`, etc. as unit-of-work semantics.
  - `docs/44-pi-focusa-integration-spec.md` was corrected in this audit to make task/mission semantics primary and cwd/project naming fallback only.
- Code evidence:
  - `apps/pi-extension/src/state.ts:454-458` creates frame title/goal as `Pi: ${projectName}` / `Work on ${projectName}`.
- Current classification:
  - **integration-layer drift**, not core Focus Stack absence.

### F10. Turn observability emptiness is now traced to a concrete read-model mismatch, not missing core turn implementation
- Authority:
  - `docs/INTEGRATION_SPEC.md:216-220` says Focusa intercepts requests, records turns, and updates ASCC.
  - `docs/G1-07-ascc.md` defines ASCC as the canonical frame-scoped context layer.
  - `docs/39-thread-lifecycle-spec.md:129-133` says archived thread state preserves telemetry for training.
- Code evidence:
  - `crates/focusa-api/src/routes/turn.rs:52-67` emits `TurnStarted`.
  - `crates/focusa-api/src/routes/turn.rs:381-396` emits `TurnCompleted`.
  - `crates/focusa-core/src/reducer.rs:251-266` stores active turn state.
  - `crates/focusa-core/src/reducer.rs:303-336` appends CLT interaction, updates frame stats, telemetry, and signals on turn completion.
  - `crates/focusa-core/src/types.rs:884-966` serializes `FocusaEvent` with `#[serde(tag = "type")]`, yielding variant names such as `TurnStarted` / `TurnCompleted`.
  - `crates/focusa-core/src/types.rs:1336-1355` flattens the event into persisted `EventLogEntry` objects.
  - `crates/focusa-api/src/routes/events.rs:28-46` and `:60-66` return raw persisted event-log JSON from `/v1/events/recent` without normalizing event type names.
  - `crates/focusa-cli/src/commands/turns.rs:60-106` expects `type` values `turn_started` / `turn_completed` and therefore does not populate started/completed timestamps from actual persisted events.
- Runtime evidence:
  - isolated audit observed `focusa turns list --json --session current --include-open` returning `[]`.
- Current classification:
  - **CLI/read-model bug with exact event-type mismatch**, not evidence that core turn recording is absent.

---

## Full core sweep — second pass: persistence/readback for Focus Stack + session/turn coherence

### P1. Persisted event log preserves session IDs after reducer mutation, not request-time context
- Code evidence:
  - `crates/focusa-core/src/runtime/daemon.rs:326-333` assigns `entry.session_id = self.state.session.as_ref().map(|s| s.session_id)` after reducer application.
  - `crates/focusa-core/src/types.rs:1336-1355` defines persisted `EventLogEntry` with flattened event payload and optional `session_id`.
- Audit meaning:
  - readback from `/v1/events/recent` reflects post-reducer session attachment, not necessarily the raw request context. This is important when interpreting “current session” filtering and post-close observability.

### P2. `/v1/events/recent` is a raw persisted-read surface, not a normalized projection
- Code evidence:
  - `crates/focusa-api/src/routes/events.rs:28-46` reads `events/log.jsonl` directly.
  - `crates/focusa-api/src/routes/events.rs:60-66` returns the raw parsed JSON entries.
- Audit meaning:
  - any mismatch between persisted event shape and CLI expectations is a real contract bug on the read-model boundary, not just a cosmetic formatter problem.

### P3. Turn core implementation is present; the broken surface is the CLI reconstruction contract
- Code evidence:
  - `crates/focusa-api/src/routes/turn.rs:52-67` and `:381-396` emit turn events.
  - `crates/focusa-core/src/reducer.rs:251-336` stores `active_turn`, appends CLT interactions, updates frame stats, and records error signals.
  - `crates/focusa-cli/src/commands/turns.rs:82-106` only recognizes snake_case event names.
- Verdict:
  - **core turn lifecycle implemented**
  - **CLI turn projection broken**

### P4. Session and active frame are exposed as independent state surfaces with no coherence guard in readback
- Code evidence:
  - `crates/focusa-api/src/routes/session.rs:21-47` derives `active_frame_summary` from `focus_stack.active_id`.
  - `crates/focusa-api/src/routes/session.rs:74-90` returns both `session` and `active_frame` in the same status payload.
  - `crates/focusa-api/src/routes/session.rs:103-106` returns raw serialized state in `/v1/state/dump`.
- Audit meaning:
  - the public readback API can legitimately present `session.status = closed` while still exposing an active frame, because no coherence rule is enforced in that surface.
- Current classification:
  - **core/API model gap** unless a spec explicitly permits active focus mutation outside an active session.

### P5. Focus mutation routes are not gated on active session state
- Code evidence:
  - `crates/focusa-api/src/routes/focus.rs:46-58` pushes frames without session-status checks.
  - `crates/focusa-api/src/routes/focus.rs:79-104` pops/sets active frame without session-status checks.
  - `crates/focusa-api/src/routes/focus.rs:279-289` updates ASCC/focus-state deltas without session-status checks.
- Audit meaning:
  - this strengthens the session/frame coherence concern: focus mutation remains available even after session close unless some other layer blocks it, and this pass found no such gate.
- Current classification:
  - **API/core lifecycle-policy gap**.

### P6. Focus Stack core contradiction remains confirmed after persistence/readback pass
- Authority:
  - `docs/G1-detail-05-focus-stack-hec.md:72-78` and `docs/G1-detail-05-focus-stack-hec.md:97-118`.
- Code evidence:
  - `crates/focusa-core/src/focus/stack.rs:5-10` claims one active frame at all times.
  - `crates/focusa-core/src/reducer.rs:785-794` clears `active_id` and `root_id` at root completion.
  - `crates/focusa-api/src/routes/session.rs:21-47` then exposes `active_frame_id` and `active_frame` directly from that state.
- Verdict:
  - **real core invariant failure**, not just a projection bug.

---

---

## Full core sweep — first-pass coverage map

This section marks what has already been swept at core depth in the current pass. It is a coverage ledger, not a final verdict.

| Subsystem | Authority anchor | Core sweep status | First-pass read |
|---|---|---:|---|
| Focus Stack / HEC | `docs/G1-detail-05-focus-stack-hec.md` | started | core invariants present in comments, but reducer behavior already contradicts them in root completion and late resume validation |
| Session lifecycle | `docs/INTEGRATION_SPEC.md`, `docs/39-thread-lifecycle-spec.md` | started | reducer enforces active/closed session transitions, but session/frame coherence remains under-specified in live behavior |
| Turn lifecycle / ASCC coupling | `docs/INTEGRATION_SPEC.md`, `docs/G1-07-ascc.md` | started | turn events are emitted and reducer updates CLT/frame stats, but turns read-model still appears mismatched |
| ECS | `docs/G1-detail-08-ecs.md` | started | store/resolve/content/rehydrate surfaces exist, but retrieval path still fails at runtime |
| Export / contribution | `docs/21-data-export-cli.md`, `docs/22-data-contribution.md` | started | export surface remains miswired/stubbed vs authoritative export spec |
| Reflection loop | `docs/G1-14-reflection-loop.md` | started | reflection route surface exists; timeout/runtime diagnosis still needs deeper pass |
| Procedural memory | `docs/G1-13-cli.md` | started | core reinforce path distinguishes found vs missing rule, but API result handling collapses the distinction |
| Pi integration fidelity | `docs/44-pi-focusa-integration-spec.md`, HEC semantics | started | integration still shows cwd-derived fallback as primary in code, contrary to corrected task-first semantics |

### Core sweep notes from this pass

#### S1. Focus Stack core explicitly claims stricter invariants than some reducer behavior upholds
- Code evidence:
  - `crates/focusa-core/src/focus/stack.rs:5-10` declares core invariants including `Exactly one active Focus Frame exists at any time`.
  - `crates/focusa-core/src/reducer.rs:785-794` clears both `active_id` and `root_id` when a completed frame has no parent.
- Audit meaning:
  - this is not a docs-only discrepancy; the core module advertises an invariant that the reducer can violate.

#### S2. Turn lifecycle is substantively implemented in core, not stubbed
- Authority:
  - `docs/INTEGRATION_SPEC.md:216-220` says Focusa intercepts requests, records turns, and updates ASCC.
- Code evidence:
  - `crates/focusa-api/src/routes/turn.rs:52-67` emits `TurnStarted` through the command channel.
  - `crates/focusa-api/src/routes/turn.rs:381-396` emits `TurnCompleted` through the command channel.
  - `crates/focusa-core/src/reducer.rs:251-266` stores the active turn on `TurnStarted`.
  - `crates/focusa-core/src/reducer.rs:303-336` appends to CLT, updates telemetry, updates active-frame stats, and emits error signals on `TurnCompleted`.
- Audit meaning:
  - turn/ASCC functionality is not absent at core level; observed emptiness on `turns list` is more likely a read-model / CLI parser / event-shape problem than a missing core implementation.

#### S3. Session lifecycle exists in reducer, but it is not yet coherent with frame lifecycle guarantees
- Authority:
  - `docs/INTEGRATION_SPEC.md:216-220` couples request interception, turn recording, and ASCC updates.
  - `docs/39-thread-lifecycle-spec.md:129-133` requires preserved telemetry/inspection value for archived thread state.
- Code evidence:
  - `crates/focusa-core/src/reducer.rs:200-214` creates `SessionState` and rejects a second active session.
  - `crates/focusa-core/src/reducer.rs:235-244` only allows closing an active session.
  - `crates/focusa-api/src/routes/focus.rs:46-58`, `:79-104`, and `:279-289` accept push/pop/set-active/update requests with no session-status gate.
  - `crates/focusa-api/src/routes/session.rs:21-47` and `:89-90` serialize session and active-frame summary independently in `/v1/status`.
  - `crates/focusa-api/src/routes/session.rs:103-106` exposes raw full state via `/v1/state/dump`.
- Audit meaning:
  - core session state is real, but the current API/core stack does not enforce or even present a coherent rule tying focus mutation to an active session. A closed session can still coexist with active-frame readback and mutable focus routes.

#### S4. ECS is not missing — but runtime retrieval failure points to a deeper storage/readback break
- Authority:
  - `docs/G1-detail-08-ecs.md:77-88` defines handle resolution/content access.
  - `docs/G1-detail-08-ecs.md:97-106` defines explicit rehydration.
- Code evidence:
  - `crates/focusa-core/src/runtime/daemon.rs:2411-2424` stores artifacts via ECS and emits `ArtifactRegistered`.
  - `crates/focusa-api/src/routes/ecs.rs:101-160` implements content retrieval.
  - `crates/focusa-cli/src/commands/ecs.rs:108-140` calls content + rehydrate endpoints.
- Audit meaning:
  - ECS is present end-to-end on paper and in code structure; because runtime retrieval still fails for valid handles, the issue is deeper than missing CLI wiring.

#### S5. Reflection is implemented enough to require diagnosis before calling it absent
- Authority:
  - `docs/G1-14-reflection-loop.md:49-62` defines run/history/status CLI/API surfaces.
- Code evidence:
  - reflection routes exist and are registered in `crates/focusa-api/src/routes/reflection.rs`.
  - `crates/focusa-cli/src/api_client.rs:11-13` sets the default timeout to 2 seconds.
- Audit meaning:
  - first-pass core sweep does not support the claim that reflection is unimplemented; current evidence more strongly supports timeout/runtime fragility pending a deeper pass.

#### S6. Procedural memory failure semantics are collapsed between core and API surface
- Code evidence:
  - `crates/focusa-core/src/memory/procedural.rs:12-24` returns `None` when the rule is absent.
  - `crates/focusa-core/src/runtime/daemon.rs:2451-2458` converts that case into `Ok(vec![])`.
  - `crates/focusa-api/src/routes/memory.rs:68-76` still reports accepted semantics by merely enqueueing the action.
- Audit meaning:
  - this is a core/API contract gap: the system distinguishes failure internally, then erases that distinction before it reaches the operator.

#### S7. Pi integration drift remains an integration problem, not proof that Focus Stack core is missing
- Code evidence:
  - `apps/pi-extension/src/state.ts:454-458` creates frames from cwd-derived `Pi: <project>` / `Work on <project>` labels.
- Audit meaning:
  - this is important because it can make operators experience a fake Focusa, but it should not be conflated with absence of core HEC machinery.

---

## Full core sweep — third pass: ECS store/retrieval/rehydration

### E1. ECS authoritative contract requires resolvable metadata + retrievable content + explicit rehydration
- Authority:
  - `docs/G1-detail-08-ecs.md:58-76` defines `StoreArtifact` returning a `HandleRef` after writing blob + metadata.
  - `docs/G1-detail-08-ecs.md:77-88` defines `ResolveHandle` as metadata + content access.
  - `docs/G1-detail-08-ecs.md:97-106` defines explicit `rehydrate` returning content constrained by token budget.
  - `docs/G1-13-cli.md:74-79` makes `ecs list/cat/meta/rehydrate` part of the CLI contract.

### E2. The underlying ReferenceStore implementation is real and writes enough metadata to satisfy the spec
- Code evidence:
  - `crates/focusa-core/src/reference/store.rs:25-47` computes `sha256`, writes immutable blob content into `ecs/objects/<sha256>`, and builds a full `HandleRef`.
  - `crates/focusa-core/src/reference/store.rs:51-63` writes per-handle metadata into `ecs/handles/<id>.json`.
  - `crates/focusa-core/src/reference/store.rs:66-73` can resolve a handle from metadata file and derive the correct blob path.
- Verdict:
  - ECS storage is **not absent** at core store level.

### E3. The reducer throws away critical ECS metadata when registering artifacts into live state
- Code evidence:
  - `crates/focusa-core/src/runtime/daemon.rs:2411-2424` stores the artifact through `ReferenceStore::store(...)`, which returns a full handle containing `sha256`, `size`, and `session_id`.
  - However the emitted event is only `ArtifactRegistered { artifact_id, artifact_type, summary, storage_uri }` and does not carry the full handle.
  - `crates/focusa-core/src/reducer.rs:984-1011` reconstructs a **minimal** `HandleRef` in `state.reference_index.handles` with:
    - `size: 0`
    - `sha256: String::new()`
    - current-time `created_at` rather than store metadata
- Audit meaning:
  - this is the central ECS break: the in-memory/readback index loses the data required to locate blob content.
- Verdict:
  - **core ECS state-model bug**.

### E4. ECS API retrieval routes trust the lossy in-memory index instead of the canonical handle metadata on disk
- Code evidence:
  - `crates/focusa-api/src/routes/ecs.rs:85-95` resolves handle metadata from `focusa.reference_index.handles` only.
  - `crates/focusa-api/src/routes/ecs.rs:100-121` finds the handle in `reference_index.handles`, then computes blob path from `handle.sha256`.
  - `crates/focusa-api/src/routes/ecs.rs:150-193` uses the same in-memory handle path for rehydration.
  - `crates/focusa-core/src/reference/store.rs:66-73` already has a canonical `resolve()` path against on-disk metadata, but the API routes do not use it.
- Audit meaning:
  - because the in-memory handle has blank `sha256`, `content` and `rehydrate` derive the wrong blob path even though the store wrote the correct metadata to disk.
- Verdict:
  - **API readback path is wired to corrupted live state rather than canonical store metadata**.

### E5. ECS runtime 404s are therefore explained by a core+API metadata-loss chain, not by missing CLI support
- Runtime evidence:
  - isolated audit produced 404s for valid-handle `ecs cat` and `ecs rehydrate`.
- Code chain:
  1. `ReferenceStore::store()` writes correct blob + full metadata (`crates/focusa-core/src/reference/store.rs:25-63`).
  2. daemon emits only partial artifact registration data (`crates/focusa-core/src/runtime/daemon.rs:2411-2424`).
  3. reducer inserts a lossy handle with blank `sha256` into `state.reference_index.handles` (`crates/focusa-core/src/reducer.rs:1003-1011`).
  4. API retrieval routes read that lossy handle instead of the on-disk metadata (`crates/focusa-api/src/routes/ecs.rs:85-121`, `:150-193`).
  5. blob lookup fails, producing 404.
- Verdict:
  - **confirmed underlying ECS implementation break**, not a surface-only bug.

### E6. ECS readback surfaces are currently not spec-faithful even when storage succeeded
- Code evidence:
  - `crates/focusa-api/src/routes/ecs.rs:85-95` returns `{\"handle\": handle}` from the lossy in-memory handle list.
  - `crates/focusa-api/src/routes/ecs.rs:47-69` returns a newly found handle ID by polling `reference_index.handles` by label after store.
  - `crates/focusa-core/src/runtime/persistence_sqlite.rs:373-391` persists the entire `FocusaState`, meaning the lossy in-memory handle metadata can also be snapshotted and restored.
- Audit meaning:
  - even `resolve/meta/list` can expose degraded or incorrect handle metadata, because the canonical disk metadata is not the source of truth for those routes.
- Current classification:
  - **core state-model + API read-model defect**.

### E7. Current deepest ECS conclusion
- Re-checked against freshly re-read authoritative ECS spec:
  - `docs/G1-detail-08-ecs.md` defines HandleRef with `size` + `sha256`, requires resolve to return metadata + content, and acceptance says resolving returns exact bytes written.
- Storage implementation exists.
- CLI surface exists.
- The broken layer is not merely wiring.
- The actual failure is:
  - **full handle metadata is lost when artifact registration enters reducer state**
  - then **API retrieval/readback routes use the degraded state instead of canonical ECS metadata on disk**
- Therefore:
  - current ECS behavior is **not faithful to the authoritative ECS spec**, even though major pieces are present.

## Full core sweep — fourth pass: export, contribution, reflection, and procedural memory

### X1. Authoritative export spec is much broader than current implementation surface
- Authority:
  - `docs/21-data-export-cli.md:1-7` defines export as authoritative, read-only, deterministic, auditable.
  - `docs/21-data-export-cli.md:10-20` defines dataset export families.
  - `docs/21-data-export-cli.md:70-90` defines multi-phase export behavior: discovery, extraction, normalization, validation.
  - `docs/22-data-contribution.md` defines a distinct contribution workflow.
- Audit meaning:
  - export and contribution are related but not interchangeable surfaces.

### X2. `export status` is miswired to contribution queue status, not an authoritative export-pipeline model
- Code evidence:
  - `crates/focusa-cli/src/commands/export.rs:54-78` calls `/v1/training/status` and renders `contribution_enabled`, `queue_size`, `pending`, `approved`, `total_contributed`.
  - `crates/focusa-api/src/routes/training.rs:22-32` exposes those same contribution queue fields under `GET /v1/training/status`.
- Audit meaning:
  - the current “export status” surface is actually contribution status, not export pipeline status as specced.
- Verdict:
  - **API + CLI semantic miswiring**, not just formatting drift.

### X3. Export dataset generation is still largely unimplemented, not merely hidden behind bad UX
- Code evidence:
  - `crates/focusa-cli/src/commands/export.rs:86-97` explicitly says SFT export pipeline is not yet implemented.
  - `crates/focusa-cli/src/commands/export.rs:102-107` shows preference/contrastive/long-horizon exports are also prose stubs.
- Audit meaning:
  - the authoritative export spec describes a real extraction pipeline, but the present implementation is still placeholder text in the CLI.
- Verdict:
  - **spec functionality missing/incomplete**.

### X4. Contribution workflow exists as a separate implemented surface
- Authority:
  - `docs/22-data-contribution.md` defines explicit contribution controls and queue handling.
- Code evidence:
  - `crates/focusa-api/src/routes/training.rs:35-50` implements enable/pause.
  - `crates/focusa-api/src/routes/training.rs:53-79` implements approve/submit.
  - `crates/focusa-api/src/routes/training.rs:82-88` registers the contribution routes.
- Verdict:
  - contribution support is **present**, but it should not be mistaken for completed export implementation.

### X5. Reflection loop implementation is substantial and test-backed
- Authority:
  - `docs/G1-14-reflection-loop.md:49-62` defines run/history/status and scheduler tick surfaces.
  - `docs/G1-14-reflection-loop.md:44-47` requires idempotency.
  - `docs/G1-14-reflection-loop.md:35-43` requires safety controls including cooldown and low-confidence stop behavior.
- Code evidence:
  - `crates/focusa-cli/src/commands/reflection.rs:96` posts to `/v1/reflect/run`.
  - `crates/focusa-cli/src/commands/reflection.rs:175` reads `/v1/reflect/status`.
  - `crates/focusa-cli/src/commands/reflection.rs:280` posts to `/v1/reflect/scheduler/tick`.
  - `crates/focusa-api/src/routes/reflection.rs:1187-1194` registers the reflection routes.
  - `crates/focusa-api/src/routes/reflection.rs:1245` includes test `reflect_run_is_idempotent_by_key_and_window`.
  - `crates/focusa-api/src/routes/reflection.rs:1654` includes test `scheduler_tick_respects_enable_and_cooldown`.
- Verdict:
  - reflection is **not absent**; core/API implementation is real and materially aligned with the spec surface.

### X6. Reflection audit failures are more consistent with runtime/timeout fragility than absent implementation
- Code evidence:
  - `crates/focusa-cli/src/api_client.rs:11-13` sets default API timeout to 2 seconds.
  - `crates/focusa-api/src/routes/reflection.rs:1149-1178` performs non-trivial scheduler tick work including DB open/schema checks and reflection execution.
- Runtime evidence:
  - isolated audit timed out on `reflect run` and `reflect scheduler tick`, while other reflection surfaces worked.
- Verdict:
  - **likely timeout/runtime health issue**, not proof reflection loop is unimplemented.

### X7. Procedural memory reinforcement has a real core operation, but failure semantics are erased before reaching the operator
- Code evidence:
  - `crates/focusa-core/src/memory/procedural.rs:12-24` reinforces only when a matching rule exists, otherwise returns `None`.
  - `crates/focusa-core/src/runtime/daemon.rs:2451-2458` converts missing-rule into `Ok(vec![])`.
  - `crates/focusa-api/src/routes/memory.rs:68-76` simply enqueues the action and reports accepted semantics.
- Verdict:
  - **core operation exists**, but **API/result contract is wrong** for invalid rule IDs.

### X8. Current deepest conclusion for this sweep
- Re-checked against freshly re-read authoritative specs:
  - `docs/21-data-export-cli.md` defines a real read-only deterministic export pipeline with dry-run outputs and export phases.
  - `docs/22-data-contribution.md` defines ODCL as a **post-cognition export pipeline** that is **not** part of Focus State, CLT, reducer logic, prompt assembly, or autonomy decision-making.
  - `docs/G1-14-reflection-loop.md` defines reflection as an overlay workflow with idempotency and guardrails.
- export: **not implemented to authoritative spec**
- contribution: **implemented, but separate from export and not part of reducer/prompt assembly core**
- reflection: **substantively implemented; current audit failures do not prove absence**
- procedural memory reinforce: **implemented, but invalid-id behavior is not spec-faithful at the API surface**

## Full core sweep — fifth pass: command router completeness and CLI `--json` contract

### C1. Authoritative CLI contract requires machine-readable `--json` output
- Authority:
  - `docs/G1-13-cli.md:19-23` defines `--json` as a global flag.
  - `docs/G1-13-cli.md:98-100` requires `--json` to be exact API-response/machine-readable output.
  - `docs/G1-13-cli.md:104-105` requires non-zero exit codes on failure.

### C2. Command router completeness is partial; surfaced CLI commands exceed `/v1/commands/submit` support
- Code evidence:
  - `crates/focusa-api/src/routes/commands.rs:186-301` accepts these command families:
    - `focus.push_frame`
    - `focus.pop_frame`
    - `focus.set_active`
    - `gate.ingest_signal`
    - `gate.surface_candidate`
    - `gate.pin` / `gate.pin_candidate`
    - `gate.suppress` / `gate.suppress_candidate`
    - `memory.semantic.upsert`
    - `memory.procedural.reinforce`
    - `memory.decay_tick`
    - `session.start`
    - `session.close`
    - `ascc.checkpoint`
    - `compact` / `micro-compact`
    - `instances.connect`
    - `instances.disconnect`
  - The same file rejects everything else with `Unknown or disallowed command: ...`.
  - `crates/focusa-cli/src/commands/cache.rs:54-63` submits `cache.bust`, which is not present in that router.
- Verdict:
  - **commands API is incomplete relative to surfaced command usage**.

### C3. CLI `--json` support is broad but not universal; threads are a confirmed hard violation
- Inventory evidence:
  - most command modules accept a json flag and contain explicit json branches (`autonomy.rs`, `cache.rs`, `clt.rs`, `constitution.rs`, `contribute.rs`, `debug.rs`, `ecs.rs`, `env.rs`, `export.rs`, `focus.rs`, `gate.rs`, `memory.rs`, `proposals.rs`, `reflection.rs`, `rfm.rs`, `skills.rs`, `telemetry.rs`, `tokens.rs`, `turns.rs`).
  - `crates/focusa-cli/src/commands/threads.rs:64` accepts `_json: bool` and ignores it.
  - `crates/focusa-cli/src/commands/wrap.rs` has no json-aware run signature.
- Audit meaning:
  - current CLI is not uniformly spec-faithful on machine-readable output, even though many modules do attempt json support.

### C4. Thread commands are confirmed `--json` contract violations, not missing API support
- Authority:
  - `docs/G1-13-cli.md:98-100`.
- Code evidence:
  - `crates/focusa-cli/src/commands/threads.rs:64-185` always renders human-readable output paths for list/get/transfer.
  - `crates/focusa-api/src/routes/threads.rs:24-40` and `:123-150` already return structured JSON.
- Verdict:
  - **CLI contract bug**.

### C5. Export commands have mixed `--json` behavior: flag exists, but spec-faithful machine output does not
- Code evidence:
  - `crates/focusa-cli/src/commands/export.rs:50-78` pretty-prints JSON for `status`, but that JSON is the wrong semantic payload (contribution status, not export status).
  - `crates/focusa-cli/src/commands/export.rs:86-107` dry-run/export branches print prose/TODO text instead of spec-defined machine-readable export results.
- Verdict:
  - **CLI json branch exists but does not satisfy authoritative export semantics**.

### C6. Turns command has a json branch, but its read-model is broken by event-shape mismatch
- Code evidence:
  - `crates/focusa-cli/src/commands/turns.rs:161-184` does emit JSON output.
  - `crates/focusa-cli/src/commands/turns.rs:60-106` reconstructs turns by expecting `type` values `turn_started` / `turn_completed`.
  - persisted events actually serialize enum variant tags such as `TurnStarted` / `TurnCompleted` (`crates/focusa-core/src/types.rs:884-966`).
- Verdict:
  - **json surface exists, but read-model contract is broken**.

### C7. Current deepest conclusion for command/json sweep
- Re-checked against freshly re-read `docs/G1-13-cli.md`:
  - `--json` is a global flag
  - `--json` output must be machine-readable / exact API passthrough
  - CLI must be scriptable, deterministic, JSON-friendly
- the CLI is **not globally non-json-compliant**; many modules do implement json branches
- but authoritative scriptability is still broken by a smaller number of high-impact violations:
  - threads ignore `--json`
  - export json surfaces are semantically wrong/incomplete
  - turns json surface is built over a broken read-model
  - commands API rejects surfaced command names like `cache.bust`
- therefore the current command/CLI layer is **partially spec-faithful, not fully compliant**

## Full core sweep — sixth pass: Pi integration fidelity (spec-first)

### I1. Fresh authority re-read for this sweep
- `docs/G1-detail-05-focus-stack-hec.md`
  - Focus Frame is a **unit of work** with human-readable `title` and `goal`.
  - HEC is the authoritative source of what Focusa is focused on.
- `docs/G1-07-ascc.md`
  - ASCC is the primary persistent structured summary per focus frame.
- `docs/INTEGRATION_SPEC.md`
  - Focusa should observe every turn and persist Focus State.
- `docs/44-pi-focusa-integration-spec.md`
  - Focusa is the **single cognitive authority**.
  - Pi extension should be UX glue / observability, not an independent cognitive authority.
  - Task/mission semantics are primary; cwd/project naming is fallback only.

### I2. First spec-to-code comparison for Pi fidelity
- Code evidence:
  - `apps/pi-extension/src/state.ts:454-458` still creates `title = "Pi: <project>"` and `goal = "Work on <project>"` from cwd/project basename.
  - `apps/pi-extension/src/commands.ts:96-124` renders `/focusa-context` from live state.
  - `apps/pi-extension/src/index.ts` separately renders persisted message/details surfaces (previous audit already identified this split).
- Spec comparison:
  - HEC requires frame semantics to represent the active **unit of work**, not merely cwd identity.
  - Pi integration spec says Focusa is the single cognitive authority and cwd/project naming is fallback only.
- Initial classification:
  - **integration-layer drift remains confirmed**.

### I3. Live-state surfaces are closer to spec authority than persisted Pi summary surfaces
- Authority:
  - `docs/G1-07-ascc.md` defines ASCC as the persistent structured summary per focus frame.
  - `docs/44-pi-focusa-integration-spec.md:11-15` says Focusa is the single cognitive authority and Pi is UX glue/observability.
- Code evidence:
  - `apps/pi-extension/src/state.ts:439-451` fetches `/focus/stack`, scopes by `S.activeFrameId`, and returns live frame + `frame.focus_state`.
  - `apps/pi-extension/src/commands.ts:45-89` renders `/focusa-context` directly from live `frame` + live `focus_state`.
  - `apps/pi-extension/src/state.ts:561-574` persists a separate Pi-local entry containing `localDecisions`, `localConstraints`, `localFailures`, `turnCount`, and other local counters.
  - `apps/pi-extension/src/index.ts:90-107` renders persisted message `details` as `focusa-state` / `focusa-wbm-state` summary text.
- Audit meaning:
  - the live `/focusa-context` path is closer to spec-faithful Focusa authority.
  - persisted Pi summary entries are a local reconstruction, not authoritative ASCC/frame readback.
- Current classification:
  - **integration read-model split** remains confirmed.

### I4. Frame creation is still primary-cwd-derived, contrary to HEC + corrected Pi spec
- Authority:
  - `docs/G1-detail-05-focus-stack-hec.md` defines a frame as a unit of work with human-readable `title` and `goal`.
  - `docs/44-pi-focusa-integration-spec.md:11-15` makes Focusa the single cognitive authority.
  - corrected Pi spec sections require task/mission-first semantics and cwd/project naming only as fallback.
- Code evidence:
  - `apps/pi-extension/src/state.ts:454-465` derives `projectName` from cwd basename, then sets `title = \`Pi: ${projectName}\`` and `goal = \`Work on ${projectName}\`` before pushing a frame.
  - `apps/pi-extension/src/state.ts:484-490` later re-finds the frame by matching that cwd-derived title plus generated beads/tag identifiers.
- Audit meaning:
  - Pi still creates primary frame semantics from cwd identity rather than the active unit of work.
- Current classification:
  - **integration-layer authority drift**.

### I5. Deepest current Pi-fidelity conclusion
- Re-checked against freshly re-read HEC/ASCC/Pi integration specs.
- Pi is **not** operating as an independent cognitive core; it does fetch live frame state from Focusa for `/focusa-context`.
- But Pi still exposes a degraded imitation in important places because:
  - frame creation semantics are cwd-first instead of task/mission-first
  - persisted summary renderers are local Pi detail records, not authoritative Focusa ASCC/frame projections
- Therefore:
  - current Pi integration is **partially spec-faithful but not yet fully aligned with Focusa as the single cognitive authority**.

## Fix-order map (derived from completed disciplined sweeps)

This is a provisional implementation order derived from authoritative-spec impact, not convenience.
Priority is based on: whether the issue prevents Focusa from being the real system as specced, whether it corrupts canonical state, and whether downstream surfaces can trust the result.

### P0 — Core spec violations that undermine canonical Focusa behavior

#### FOM-1. Focus Stack invariant repair
- Why first:
  - HEC is the core authority for “what Focusa is focused on.”
  - If root completion can clear `active_id` / `root_id`, canonical focus becomes spec-invalid.
- Source findings:
  - `crates/focusa-core/src/focus/stack.rs:5-10`
  - `crates/focusa-core/src/reducer.rs:785-794`
- Required outcome:
  - root completion/pop/set-active lifecycle must satisfy HEC invariants and reject invalid transitions before reporting acceptance.

#### FOM-2. ECS metadata-loss/readback repair
- Why second:
  - ECS is a core lossless-indirection mechanism.
  - Current implementation stores data, then loses required metadata before readback.
- Source findings:
  - `crates/focusa-core/src/reference/store.rs:25-73`
  - `crates/focusa-core/src/runtime/daemon.rs:2411-2424`
  - `crates/focusa-core/src/reducer.rs:984-1011`
  - `crates/focusa-api/src/routes/ecs.rs:85-121,150-193`
- Required outcome:
  - canonical handle metadata must survive registration and drive API readback.

#### FOM-3. Session/frame coherence policy repair
- Why third:
  - Focus mutation and session lifecycle currently lack a coherent authority boundary.
  - Closed session + mutable/active frame makes the runtime model ambiguous.
- Source findings:
  - `crates/focusa-core/src/reducer.rs:200-244`
  - `crates/focusa-api/src/routes/focus.rs:46-58,79-104,279-289`
  - `crates/focusa-api/src/routes/session.rs:21-47,103-106`
- Required outcome:
  - explicit rule for whether focus mutation requires active session, and matching enforcement/readback semantics.

### P1 — Major spec implementation gaps that block trustworthy operator use

#### FOM-4. Export pipeline implementation
- Why here:
  - authoritative export spec is largely unimplemented.
  - export/contribution confusion can cause false confidence about training-data readiness.
- Source findings:
  - `docs/21-data-export-cli.md`
  - `crates/focusa-cli/src/commands/export.rs:54-107`
  - `crates/focusa-api/src/routes/training.rs:22-32`
- Required outcome:
  - real export pipeline matching spec phases, dry-run outputs, and export manifests.

#### FOM-5. API/command acceptance semantics repair
- Why here:
  - invalid actions are often reported as accepted and only fail later or no-op silently.
- Source findings:
  - `crates/focusa-api/src/routes/focus.rs:95-103`
  - `crates/focusa-core/src/runtime/daemon.rs:2351-2353`
  - `crates/focusa-core/src/reducer.rs:825-842`
  - `crates/focusa-api/src/routes/memory.rs:68-76`
  - `crates/focusa-core/src/runtime/daemon.rs:2451-2458`
- Required outcome:
  - invalid set-active/reinforce-like requests must surface deterministic failure semantics.

#### FOM-6. Command router completeness
- Why here:
  - surfaced command names exceeding router support break scriptability.
- Source findings:
  - `crates/focusa-api/src/routes/commands.rs:186-301`
  - `crates/focusa-cli/src/commands/cache.rs:54-63`
- Required outcome:
  - either implement surfaced commands in `/v1/commands/submit` or stop surfacing them through that path.

### P2 — Read-model / CLI contract repairs needed for trustworthy observability

#### FOM-7. Turns read-model repair
- Why here:
  - core turn recording exists, but operator observability is broken.
- Source findings:
  - `crates/focusa-core/src/types.rs:884-966,1336-1355`
  - `crates/focusa-api/src/routes/events.rs:28-66`
  - `crates/focusa-cli/src/commands/turns.rs:60-106,161-184`
- Required outcome:
  - persisted event shape and turns projection must agree so turn/session observability reflects real core state.

#### FOM-8. CLI `--json` contract repair
- Why here:
  - scriptability is an explicit CLI authority requirement.
- Source findings:
  - `docs/G1-13-cli.md:19-23,98-105`
  - `crates/focusa-cli/src/commands/threads.rs:64-185`
  - `crates/focusa-cli/src/commands/export.rs:50-107`
- Required outcome:
  - all surfaced json paths must be machine-readable and semantically correct.

#### FOM-9. Reflection runtime health verification
- Why here:
  - reflection appears implemented, but runtime reliability is still not proven.
- Source findings:
  - `docs/G1-14-reflection-loop.md`
  - `crates/focusa-cli/src/api_client.rs:11-13`
  - `crates/focusa-api/src/routes/reflection.rs:1149-1194,1245,1654`
- Required outcome:
  - determine whether failures are timeout-only, performance-only, or deeper runtime instability.

### P3 — Integration fidelity repairs

#### FOM-10. Pi frame authority alignment
- Why here:
  - important, but depends on stable core Focus Stack semantics first.
- Source findings:
  - `docs/44-pi-focusa-integration-spec.md`
  - `apps/pi-extension/src/state.ts`
- Outcome:
  - repaired.
- Repair summary:
  - `createPiFrame(...)` now derives `title` / `goal` from `S.currentAsk` when available and uses cwd/project naming only as fallback.
  - Pi-scoped frame metadata is cached/restored via `frameTitle` / `frameGoal` rather than defaulting back to cwd-derived labels.
  - isolated live runtime proof now covers explicit Focusa session creation before Pi frame push on session start/switch.

#### FOM-11. Pi persisted-summary authority alignment
- Why here:
  - current summaries can misrepresent live Focusa state.
- Source findings:
  - `apps/pi-extension/src/state.ts`
  - `apps/pi-extension/src/index.ts`
  - `apps/pi-extension/src/commands.ts`
- Outcome:
  - repaired.
- Repair summary:
  - persisted entries now include authoritative Focusa-derived snapshot fields:
    - `authoritativeDecisions`
    - `authoritativeConstraints`
    - `authoritativeFailures`
    - `intent`
    - `currentFocus`
  - renderer/status/shortcut/widget/work-loop surfaces now prefer authoritative snapshot/readback over Pi-local shadow when available.
  - session resume/switch now restore cached frame title/goal and authoritative snapshot fields.
  - lifecycle/compaction boundaries now call `persistAuthoritativeState()` so persisted entries refresh from scoped Focusa readback before being written to Pi session history.
  - isolated live runtime proof in `tests/pi_extension_runtime_authority_test.mts` / `.sh` now covers:
    - `session_start` starts a real Focusa session before frame creation
    - first real `input` rescopes generic startup fallback frame into a task-first Pi frame
    - `session_before_compact` persists authoritative snapshot fields
    - `session_compact` writes canonical file artifacts/notes from `modifiedFiles` / `readFiles`
    - `session_before_switch` closes current Focusa session
    - `session_switch` starts replacement session and re-establishes Pi frame
    - `turn_end` emits assistant output plus canonical token counts
    - `session_shutdown` closes cleanly without SSE timer leakage
  - introduced shared `getEffectiveFocusSnapshot(...)` helper so status/shortcut/widget/reset surfaces resolve authority consistently.
  - repaired Pi `turn_end` completion payload to include authoritative assistant output plus canonical `prompt_tokens` / `completion_tokens`, not only extension token envelope.
  - moved correction/significant-progress/reconnect reconciliation writes onto validated `pushDelta(...)` path and made `/focusa-reset` clear cached authoritative snapshot state before rebuilding a fresh frame.
  - added post-input rescope path so startup fallback frames like `Pi: root` are popped and recreated with task-first title/goal once the first real ask exists.
  - strips embedded quoted Focusa context/status payloads before ask classification or frame derivation, so pasted `[focusa-context] ...` dumps cannot become frame title/goal authority.
  - rejects contaminated scoped frames in live readback (`getFocusState`) and forces clean session-key recovery instead of rendering quoted Focusa payloads as authority.
  - repaired `/focusa-context` to hydrate live ASCC state via `/v1/ascc/state` (not stack-only frame snapshots), restoring `current_focus` and other dynamic slots the spec template says to review.
  - patched ASCC assistant-output extraction to ignore response scaffold lines (`Status:`, `Next action:`, `Blocker:`), preventing command/status boilerplate from poisoning `current_focus`/`next_steps`.

### Implementation sequencing note
- Fixing Pi integration before Focus Stack/ECS/session coherence would polish a potentially non-canonical substrate.
- Therefore fix order should remain:
  1. HEC invariants
  2. ECS canonical readback
  3. session/frame coherence
  4. export/acceptance/router gaps
  5. read-model/CLI contracts
  6. Pi authority alignment

## Exhaustive audit matrix template

Use one section per violation class.

### [ID] Violation title
- Status: not started | in progress | verified
- Severity: P0 | P1 | P2
- Authority:
  - spec file:line/section
- Reproduction:
  - exact commands
- Expected by spec:
- Observed:
- Layer trace:
  - CLI:
  - API route:
  - daemon/action:
  - reducer/core:
  - persistence/readback:
  - integration/session impact:
- Verdict:
- Fix owner layer:
  - CLI | API | core | persistence | integration
- Notes:

---

## Violation classes to trace exhaustively

### V1. Focus Stack invariant compliance
- Status: in progress
- Why it matters: HEC is the authoritative source of focus; if stack invariants fail, Focusa core is not functioning as designed.
- Known evidence:
  - root completion can clear active/root ids
  - invalid resume of completed frame triggers invariant violation after API acceptance
- Next trace steps:
  - inspect reducer and daemon paths for push/pop/set-active
  - inspect whether API returns acceptance before reducer validation
  - inspect persistence snapshots after each lifecycle mutation

### V2. Turn/session observability correctness
- Status: in progress
- Why it matters: ASCC and integration lifecycle depend on turn/session granularity.
- Known evidence:
  - `turns list --json` empty despite events and frame writes
  - session may report closed while frame mutates
- Next trace steps:
  - inspect actual event serialization for `TurnStarted` / `TurnCompleted`
  - compare CLI parser expectations to emitted event shape
  - inspect session lifecycle ownership in state dump and event log

### V3. ECS retrieval correctness
- Status: in progress
- Why it matters: ECS is required for lossless compression by indirection.
- Known evidence:
  - `store/list/meta/resolve` work
  - `cat` and `rehydrate` fail on valid handles
- Next trace steps:
  - inspect API content/rehydrate handlers fully
  - verify stored blob path/sha expectations
  - verify route method mismatch possibilities (`GET` vs `POST` already partly checked)

### V4. Export pipeline completeness vs spec
- Status: in progress
- Why it matters: export is authoritative CLI surface, read-only/deterministic/auditable.
- Known evidence:
  - export status mapped to contribution payload
  - dry-run emits prose and TODO-level behavior
- Next trace steps:
  - inspect export CLI implementation fully
  - inspect training/export API routes fully
  - compare required dry-run outputs from spec with current implementation

### V5. CLI `--json` contract compliance
- Status: in progress
- Why it matters: CLI contract explicitly requires machine-readable stable JSON.
- Known evidence:
  - thread commands ignore json mode
  - export dry-runs ignore json mode
  - some commands print text despite `--json`
- Next trace steps:
  - enumerate every CLI command/subcommand and mark json compliance
  - identify whether issue is CLI-only or upstream API shape mismatch

### V6. Reflection loop runtime health
- Status: in progress
- Why it matters: reflection loop is authoritative overlay surface with explicit CLI/API contract.
- Known evidence:
  - status/history/scheduler-config paths work
  - run/tick timed out in isolated audit
- Next trace steps:
  - inspect API timeout/runtime behavior
  - test with higher CLI timeout
  - confirm whether functionality exists but exceeds 2s default client timeout

### V7. Command router completeness
- Status: in progress
- Why it matters: surfaced commands should correspond to implemented command actions.
- Known evidence:
  - `cache.bust` exposed in CLI but rejected by commands API
  - possible gate resolve routing oddity
- Next trace steps:
  - enumerate command names accepted by `/v1/commands/submit`
  - compare against all CLI commands that rely on that route

### V8. Memory procedural reinforce contract
- Status: in progress
- Why it matters: deterministic scriptable CLI should not silently accept invalid IDs.
- Known evidence:
  - bogus rule id accepted with no event
- Next trace steps:
  - inspect API route result handling
  - inspect daemon action return semantics for not-found cases
  - decide whether core event model or API contract must change

### V9. Pi integration session fidelity
- Status: pending
- Why it matters: if bridge surfaces diverge from core state, operator sees a fake Focusa.
- Known evidence:
  - frame naming drift
  - live-frame vs summary-count mismatch
- Next trace steps:
  - compare bridge local state, Focusa live frame fetches, persisted session entries, renderers

---

## Full CLI execution record

Primary isolated exhaustive run:
- results file: `/tmp/focusa-cli-audit-results.txt`
- follow-up run: `/tmp/focusa-cli-audit-followup.txt`

These provide raw evidence for command behavior before deeper code tracing.

---

## Working rule for next iterations

This audit must be built over **multiple passes**, not a few quick sweeps.
A few passes can surface real failures, but they are not enough to prove spec-faithful implementation.

### Mandatory pass discipline
Before each new sweep, re-read the relevant authoritative spec sections for that sweep.
Do not rely only on earlier memory, prior citations, or code-first reasoning.
Each sweep must explicitly compare:
1. authoritative spec text
2. intended behavior derived from that text
3. current implementation in code
4. actual runtime/readback behavior

For each violation class:
1. quote authority
2. state intended behavior from that authority
3. reproduce runtime behavior
4. inspect source at CLI/API/core layers
5. inspect persisted/readback evidence
6. classify exact failing layer
7. only then create/adjust fix beads

This document should be updated continuously as the audit proceeds.

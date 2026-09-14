# 82 — Focusa Memory Optimization Spec

**Date:** 2026-04-21
**Status:** implementation-complete — revalidated 2026-07-12 with bounded retrieval/persistence, lock, telemetry/degrade, and memory-SLO runtime gates
**Priority:** critical

## 1) Objective

Reduce Focusa runtime memory usage and memory-driven latency so Pi + Focusa remain responsive under normal multi-session use.

---

## 2) Observed baseline (current)

From live host measurements during this session:

- System memory before cleanup: `~11Gi/14Gi used`, `~33Mi free`, no swap
- System memory after cleanup: `~7.2Gi/14Gi used`, `~3.1Gi free`
- Focusa daemons (duplicate instances) total RSS observed: `~3.04 GiB`
- Single release daemon RSS observed: `~1.7–2.2 GiB` range
- Typing latency in Pi correlated with RAM pressure + CPU contention

Implication: runtime memory footprint is high enough to create user-visible slowdown even before full saturation.

---

## 3) Root causes (confirmed + likely)

## 3.1 Confirmed

1. **Duplicate daemon/process concurrency**
   - Multiple `focusa-daemon` and Pi-related processes running simultaneously.

2. **Unbounded in-memory stores in API routes**
   - Snapshot store uses process-global map:
     - `crates/focusa-api/src/routes/snapshots.rs` (`OnceLock<Mutex<HashMap<...>>>`)
   - Metacognition store uses growing vectors:
     - `crates/focusa-api/src/routes/metacognition.rs` (captures/reflections/adjustments)

3. **No swap safety net on host**
   - `Swap: 0B` increases risk of severe contention under spikes.

## 3.2 Likely contributors

4. Full payload retention in memory where index-only hot state is sufficient.
5. Heavy JSON cloning/serialization paths.
6. Over-large default response payloads (lineage/snapshot diff shape growth over time).

---

## 4) Target state

## 4.1 Memory targets

- **Single Focusa daemon RSS steady state:** `< 700 MB`
- **P95 operational peak:** `< 1.2 GB`
- **No duplicate daemon instances in steady state**

## 4.2 UX targets

- Pi typing/input latency stays stable during active Focusa use.
- No memory-induced degraded state during normal tool/CLI operations.

## 4.3 Resource modes and explicit `LowMem` target

Focusa must adapt to environment constraints instead of assuming a production-sized host.

Modes:

| Mode | Host posture | Target behavior |
|---|---|---|
| normal | enough RAM/CPU for rich context | full hot+warmed cognition within budgets |
| constrained | pressure visible but stable | top-k enrichment, smaller windows, background throttle |
| LowMem | extreme constrained/tiny host | every tool remains callable through surgical summaries, caches, and rehydrate refs |
| emergency | daemon survival | T0/T1 hot core only live; all cold work blocked/degraded explicitly |

LowMem target:

Activation/deactivation surface:

- `focusa_resource_mode(action="activate_lowmem")` turns on LowMem at runtime.
- `focusa_resource_mode(action="deactivate_lowmem")` clears the runtime LowMem override back to auto.
- Operator phrases like “Activate LowMem mode” and “Deactivate LowMem mode” should map to this tool.

- Hot core always works: health, `/v1/status` hot summary, ProjectIdentity, Trajectory summary, Workpoint current/resume, Focus State compact slots, evidence handles, tool doctor, work-loop writer/status summary, tool affordances.
- Deep status diagnostics live behind `/v1/status/deep` or explicit `?deep=true`; hot status must list omitted cold fields instead of scanning them.
- Same public tools remain visible; fidelity changes, availability should not disappear.
- Default responses are prompt-safe summaries with caps, omitted counts, and rehydrate refs.
- Full payloads are explicit cold opt-in and may return `resource_exhausted` without freezing the daemon.
- Context pruning follows importance order: liveness → continuation → safety/scope → evidence handles → surgical context → learning/risk top-k → diagnostics/history.

---

## 5) Optimization workstreams

## A) Process lifecycle guardrails (highest impact)

1. Enforce single-daemon lock (pidfile/file lock).
2. Refuse startup if another active daemon owns the lock.
3. Add `focusa doctor runtime` check for duplicate daemon detection.
4. Add safe cleanup command for orphaned dev daemons.

**Acceptance:**
- Attempting second daemon exits with typed error.
- Runtime check reports exactly one active daemon.

## B) Bounded memory stores (mandatory)

### B1 Snapshot store limits

- Add configurable caps:
  - `max_snapshots`
  - `snapshot_ttl_minutes`
- Eviction policy: `TTL + LRU`.
- Persist evicted records to durable storage if needed.

### B2 Metacognition store limits

- Add configurable caps per collection:
  - `max_captures`
  - `max_reflections`
  - `max_adjustments`
  - TTL controls per type
- Eviction policy: `TTL + score-aware recency`.

**Acceptance:**
- Long-run test shows bounded store cardinality.
- RSS plateaus instead of monotonic growth.

## C) Persistence offload architecture

1. Move full records to durable layer (event log/sqlite).
2. Keep RAM as hot index/cache only.
3. Rehydrate on demand for detailed views.
4. ECS specifically keeps immutable blobs and atomically published full metadata in
   its existing filesystem store while `FocusaState.reference_index` retains a strict
   2,048-record hot projection. Hot eviction is not artifact deletion; exact-ID
   resolve/rehydrate remains lossless.
5. Persistence telemetry distinguishes serialized `snapshot_bytes` from
   `database_bytes` and `wal_bytes`; SQLite file allocation is never reported as
   cognitive snapshot payload size.

Read-only incident-shape measurement on 2026-09-04 found 17,469 hot handle records
occupying 22,000,805 serialized bytes. Applying the 2,048-record policy to the same
ordering retains 2,055,291 bytes and moves 15,421 records to exact-ID cold lookup—a
90.7% reduction in the dominant snapshot field without deleting one artifact.

**Acceptance:**
- RAM index scales sublinearly with historical record growth.
- Historical retrieval still works with acceptable latency.

## D) Payload shaping and pagination

1. Default capped responses for lineage/tree-like endpoints.
2. Cursor pagination + summary-first mode.
3. Add explicit `include_full_payload=true` opt-in.

**Acceptance:**
- Default response sizes remain bounded.
- No large unbounded JSON bodies by default.

## D2) Surgical traversal/parsing substrate

Focusa-wide large structures must support partial traversal and parsing rather than all-or-nothing reads.

Required selectors by structure class:

| Structure | Required hot selectors |
|---|---|
| Lineage/CLT | head, path, children, neighborhood, summaries, cursor page |
| Ontology graph | working_set, adjacency, neighborhood, object search, link path, affordance summary |
| Focus Stack | active, path, ancestors, siblings, bounded frame window |
| Workpoints | current, by id, recent, evidence window, blockers window, drift window |
| Evidence/ECS/reference handles | search, by id/meta, recent, type/tag filter, rehydrate handle |
| Metacognition/predictions | recent, search/retrieve, by id, top-k, cursor page |
| Telemetry/commands/turn logs | recent, cursor page, event type filter, time window |
| Snapshots/diffs | recent index, by id metadata, bounded diff, restore metadata |
| Tool registry/capabilities | family filter, tool by name, summary counts |
| Trajectory/Focus Slice | project-scoped summary, active gap, do_not_use, next candidate |

Acceptance:

- Each hot selector has hard caps and emits traversal metadata.
- Full payload access is cold-path opt-in with timeout/byte/token caps.
- Safe audit can prove hot traversal stays bounded in low-memory mode.

## E) Allocation and serialization efficiency

1. Replace broad `serde_json::Value` copies where feasible with typed structures.
2. Reduce cloning in hot paths.
3. Prefer borrowed/Arc-backed sharing for repeated blobs.

**Acceptance:**
- Profiling shows reduced allocation rate and lower peak heap.

## F) Memory observability and budgets

1. Expose memory telemetry endpoint/metrics:
   - rss_mb, heap_estimate_mb, store_counts, eviction_counts
2. Define hard/soft memory budgets with degrade modes:
   - summarize-only mode when budget exceeded
   - reject non-critical writes under pressure

**Acceptance:**
- Budget crossing is visible and deterministic.
- Degrade mode prevents runaway memory growth.

## G) Host-level safety (ops lane)

1. Add small swap file for burst tolerance.
2. Validate cgroup/systemd memory limits (if applicable).

**Acceptance:**
- Host avoids hard-thrash behavior on transient spikes.

## H) LowMem adaptive resource mode

LowMem is the extreme-mode implementation lane.

Required components:

1. `ResourceMode` detector/control surface:
   - explicit env/config override.
   - runtime activation/deactivation via `focusa_resource_mode` / `POST /v1/resource/mode`.
   - daemon-level background monitor runs even when no active Pi/agent session exists.
   - RSS, peak RSS, host MemAvailable, cgroup pressure, hot timeout rate, OOM/restart evidence.
   - every automatic fallback/recovery transition records a bounded `ResourceModeTransitionRecord` before applying the new mode; `active_session_id=null` is valid.
   - hysteresis to prevent mode flapping.
2. `LowMemBudget` policy:
   - one resolved RSS budget owns ResourceMode, `/v1/status`, telemetry, and tests; `FOCUSA_LOWMEM_RSS_SOFT_MB` / `FOCUSA_LOWMEM_RSS_HARD_MB` are canonical and deprecated `FOCUSA_MEMORY_BUDGET_MB` is a hard-limit fallback only.
   - hot/warm/cold timeout budgets.
   - default/hard item caps.
   - byte/token caps.
   - background concurrency cap.
   - allocator trimming interval.
   - daemon ResourceMode monitor interval (`FOCUSA_RESOURCE_MODE_MONITOR_INTERVAL_SECS`).
3. mode-aware route helpers:
   - derive caps from resource mode.
   - block/degrade cold routes before expensive work.
   - include `resource_mode`, `budget`, `omitted`, `rehydrate_refs`, and `failure_class` in responses.
4. mode-aware prompt injection:
   - Focus Slice includes `RESOURCE_MODE`, pressure reason, best next tools, and `DO_NOT_USE_BY_DEFAULT` cold surfaces.
   - Workpoint Resume Packet v2 includes pruned counts and traversal refs.
5. importance-based eviction/pruning:
   - retain active Workpoint/Trajectory/ProjectIdentity/safety slots/evidence refs first.
   - evict raw telemetry/replay/log/full payload caches first.
   - top-k metacog/predictions by active gap/failure class.
6. LowMem eval/stress harness:
   - simulate tiny budgets.
   - assert all tools remain advertised/callable.
   - prove hot routes stay responsive under cold-route pressure.
   - prove agent can complete a surgical task with summaries + targeted traversal.

**Acceptance:**
- LowMem can be forced by env and detected automatically.
- LowMem proof confirms tool registrations, contracts, docs, API route inventory, live ontology contracts, and representative read dependencies stay available.
- Auto LowMem fallback works without an active session and records a transition first.
- No official tool disappears in LowMem.
- Hot core routes meet configured tiny-host timeouts.
- Cold routes degrade explicitly instead of freezing or causing daemon restart.
- Safe audit and golden evals cover LowMem behavior.

---

## 6) Phased implementation plan

## Phase 1 — Immediate stabilization

- Single-daemon lock
- Duplicate-process detection command
- Configurable caps + TTL on snapshot/metacog stores

**Exit criteria:**
- Duplicate daemons prevented
- Store cardinality bounded

## Phase 2 — Structural reduction

- Persistence offload of full records
- RAM index/hot-cache model
- Default response caps/pagination

**Exit criteria:**
- Single daemon steady-state RSS reduced by >=40% from baseline

## Phase 3 — Hardening + SLO

- Heap/allocation tuning and profiling pass
- Budget/degrade enforcement
- Memory SLO gates in CI/perf harness
- Explicit LowMem mode: forced env/config, auto detection, mode-aware route/tool envelopes, Focus Slice/Resume Packet injection, and LowMem golden evals

**Exit criteria:**
- Meets target RSS and latency stability criteria
- LowMem proves every official tool stays callable and hot core stays responsive under tiny-host budgets

---

## 7) Validation and test plan

1. **Soak test (4h)**
   - repeated snapshot + metacog loops
   - verify RSS plateau
2. **Growth test**
   - inject >10x baseline records
   - verify eviction/persistence behavior
3. **Duplicate-start test**
   - second daemon startup rejected
4. **Response-size test**
   - default endpoint payload caps enforced
5. **Regression test**
   - tool + CLI behavior unchanged functionally
6. **LowMem forced-mode test**
   - set `FOCUSA_RESOURCE_MODE=lowmem` and tiny budgets
   - verify all official tools remain advertised/callable
   - verify cold/full payload routes degrade explicitly
7. **LowMem surgical task eval**
   - fresh agent receives only LowMem Focus Slice/Resume Packet
   - agent completes a scoped task using summaries, evidence handles, and `focusa_traverse`

---

## 8) Risks and mitigations

- Risk: over-aggressive eviction harms recall
  - Mitigation: persistence offload + on-demand rehydrate
- Risk: pagination changes break consumers
  - Mitigation: compatibility flags + contract tests
- Risk: degrade mode surprises operators
  - Mitigation: explicit telemetry + clear typed envelope messaging

---

## 9) Out of scope (this spec)

- Tool-surface feature expansion unrelated to memory behavior
- Broad server service tuning not tied to Focusa runtime memory

---

## 10) Definition of done

Done when all are true:

1. Duplicate daemons prevented by design.
2. Snapshot/metacognition in-memory stores are bounded by policy.
3. Full historical data no longer requires full RAM residency.
4. Default payload sizes are bounded and paginated.
5. Memory telemetry + budget/degrade behavior is active.
6. Single-daemon steady-state RSS is within target band.

# 82 — Focusa Memory Optimization Spec

**Date:** 2026-04-21  
**Status:** active  
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

**Exit criteria:**
- Meets target RSS and latency stability criteria

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

# 94 — Focusa Intent-Preserving Memory, Payload, and RPC Optimization SOW

**Date:** 2026-05-03  
**Status:** implementation-complete — revalidated 2026-07-12 with bounded-route, profile, store-growth, pressure/degrade, CPU/RSS and payload gates  
**Priority:** critical  
**Owner:** Focusa + Pi integration  
**Source:** Four evidence-only optimization passes plus one prior implementation-overstep audit.  

---

## 1) Why this spec exists

Focusa has enough live usage and code surface to expose real memory, payload-size, and RPC-efficiency pressure:

- Focusa daemon RSS is still materially above the memory target in Spec82.
- Several read endpoints can return multi-megabyte JSON payloads by default.
- Some API routes scan or clone full collections before truncating.
- Pi integration can emit multiple telemetry calls per turn.
- Telemetry/tool-contract docs and live API parity are not fully aligned.

This spec consolidates **unimplemented or not-yet-fully-generalized SOW** from the recent optimization evidence passes into one core-intent-preserving roadmap.

---

## 2) Core intent boundaries — non-negotiable

Every optimization in this SOW must preserve the original Focusa feature intent.

### 2.1 Focusa remains the cognitive authority

- Focusa is the local cognitive runtime and single source for structured memory, continuity, evidence, governance, and Workpoint truth.
- Pi and other adapters stay thin UX/transport glue.
- No optimization may move canonical cognition into Pi, transcript tails, local shadow arrays, or opaque caches.

**Evidence:**
- `README.md:29-41`, `README.md:101-106`, `README.md:374-376`
- `docs/44-pi-focusa-integration-spec.md:12-16`
- `docs/83-pi-focusa-rpc-efficiency-spec.md:17-28`

### 2.2 Workpoint continuity must not be weakened

- Workpoint state remains a typed continuation projection, not raw transcript memory.
- Resume packets remain authoritative when canonical unless the operator explicitly steers otherwise.
- Optimizations may make projections smaller, paginated, or summary-first, but must not remove mission/action/evidence/blocker/do-not-drift semantics.

**Evidence:**
- `docs/88-ontology-backed-workpoint-continuity.md:28-45`
- `docs/88-ontology-backed-workpoint-continuity.md:87-113`
- `docs/88-ontology-backed-workpoint-continuity.md:439-481`
- `docs/88-ontology-backed-workpoint-continuity.md:551`

### 2.3 Degraded state must stay explicit

- Fallbacks, caps, truncation, pressure modes, and partial projections must be explicitly marked.
- No fallback output may be silently promoted to canonical truth.

**Evidence:**
- `README.md:157`, `README.md:186`
- `docs/89-focusa-tool-suite-improvement-hardening-spec.md:153-154`

### 2.4 Observability must be preserved or improved

- Optimizations must remain measurable and auditable.
- Telemetry remains passive, local-first, queryable, bounded, and low overhead.

**Evidence:**
- `docs/29-telemetry-spec.md:1-36`
- `docs/31-telemetry-api.md:1-55`

### 2.5 Ontology semantics must not be stripped, flattened, or replaced

The ontology optimization target is **expression shape**, not ontology substance.

Any bounded, summary, slice, cursor, cache, or streaming change must preserve these ontology commitments in full:

- ontology is additive in implementation and canonical in semantics;
- ontology supplies the typed software/work/mission/execution world Focusa and Pi consume;
- object identity, object types, typed properties, typed links, typed actions, working sets, constraints, missions, provenance, verification, freshness, and reducer-visible deltas remain first-class;
- slices are bounded `ObjectSet`/`SlicePolicy` expressions, never substitute schemas or lossy mini-ontologies;
- full-fidelity canonical ontology state remains available through explicit full, paginated, rehydrate, or export paths;
- projection defaults may omit payload detail for budget reasons, but must not omit the fact that omitted detail exists;
- every truncation must identify what category was omitted, how much was returned, and how to continue or rehydrate;
- no object type, relation type, action type, status vocabulary, provenance class, verification rule, or governance/migration record may be removed as an optimization;
- ambiguous model-derived classification remains proposal-only, bounded, expiring, evidence-linked, and reducer-ingestible;
- only reducer-approved ontology deltas can change canonical ontology truth;
- cached contract/action projections must be deterministic references to the canonical registry, not a second ontology authority.

**Evidence:**
- `docs/45-ontology-overview.md:1-67`
- `docs/46-ontology-core-primitives.md:1-76`
- `docs/47-ontology-software-world.md:1-131`
- `docs/48-ontology-links-actions.md:1-49`
- `docs/50-ontology-classification-and-reducer.md:1-62`
- `docs/77-ontology-governance-versioning-and-migration.md:1-44`
- `crates/focusa-api/src/routes/ontology.rs:5781-5846`

---

## 3) Evidence summary from the four passes

### 3.1 Live payload and memory evidence

| Surface | Observation | SOW implication |
|---|---:|---|
| Focusa daemon RSS | ~1.36 GiB in live process sample | still above Spec82 target `<700MB` steady state |
| `/v1/ontology/world` | ~4.29 MB, ~2.64s, 7,174 objects, 11,255 links, 95 actions | default full-world payload is too large |
| `/v1/ontology/world?summary_only=true` | same ~4.29 MB; summary flag did not reduce payload | summary mode absent/ignored |
| `/v1/ontology/slices?active_mission` | ~1.7 KB | slices preserve intent with much smaller payloads |
| `/v1/ecs/handles` | 8,585 handles, ~2.51 MB by default | default should be bounded/summary-first |
| `/v1/ecs/handles?summary_only=true&limit=25` | ~3.9 KB | route already proves bounded mode works |
| `/v1/work-loop/status` | ~47 KB, ~0.38s | needs summary mode for common reads |
| `/v1/telemetry/events` | 404 despite docs specifying route | telemetry API parity gap |
| `/v1/telemetry/productivity`, `/autonomy` | 404 despite docs specifying routes | telemetry API parity gap |

### 3.2 Code evidence highlights

- Snapshot store uses process-global in-memory map with disk persistence and later pruning.
  - `crates/focusa-api/src/routes/snapshots.rs:85-87`
  - `crates/focusa-api/src/routes/snapshots.rs:172-175`
  - `crates/focusa-api/src/routes/snapshots.rs:348-384`
- Metacognition route stores vectors in a process-global store, loads records from disk, clones in-memory captures, then ranks/truncates.
  - `crates/focusa-api/src/routes/metacognition.rs:59-70`
  - `crates/focusa-api/src/routes/metacognition.rs:156-201`
  - `crates/focusa-api/src/routes/metacognition.rs:227-230`
  - `crates/focusa-api/src/routes/metacognition.rs:354-407`
- ECS handle responses are bounded, and the state/snapshot projection now has a strict hot-record cap with exact-ID durable fallback.
  - `crates/focusa-api/src/routes/ecs.rs:90-103`
  - `crates/focusa-api/src/routes/ecs.rs:121-130`
- Ontology world route builds a full combined projection, full action catalog, and all working sets in one response.
  - `crates/focusa-api/src/routes/ontology.rs:5781-5846`
- Events route reads event log into a vector before tail slicing.
  - `crates/focusa-api/src/routes/events.rs:49-66`
- Workpoint idempotency cache is an in-memory map that needs cap/TTL verification.
  - `crates/focusa-api/src/routes/workpoint.rs:23`
- Daemon lock exists, but needs regression proof and operator cleanup UX.
  - `crates/focusa-api/src/main.rs:55-80`

### 3.3 External evidence from Brave Search

- Cursor pagination is recommended for large datasets because it avoids expensive full counts and works better with changing data.
  - Brave: Speakeasy “Pagination Best Practices in REST API Design”
  - Brave: JSON:API Cursor Pagination Profile
- Axum supports custom/streaming response bodies; JSON streaming is specifically useful for huge object streams.
  - Brave: `docs.rs/axum-streams`
  - Brave: `docs.rs/axum response`, `Json`
- Rust memory profiling should precede deeper rewrites.
  - Brave: KDE `heaptrack`
  - Brave: `docs.rs/dhat`
  - Brave: Valgrind DHAT manual
- `serde_json::Value` may carry high memory overhead; repeated static keys/dynamic maps should be audited.
  - Brave: `serde-rs/json#635`

---

## 4) SOW workstreams

## A) Bounded read-response policy across Focusa API

### A1. Define default bounded response contract

Create a shared API response policy for read-heavy endpoints:

- default response must be bounded;
- default must include `total`, `returned`, `truncated`, and cursor/limit metadata when applicable;
- full payload must require explicit opt-in, e.g. `include_full_payload=true` or `mode=full_json`;
- full payload opt-in must still have a configurable hard ceiling;
- truncation must never be silent.

**Intent preservation:** preserves all data and endpoints; only changes default projection size and makes completeness explicit.

**Target surfaces:**
- `/v1/ontology/world`
- `/v1/ecs/handles`
- `/v1/memory/semantic`
- `/v1/work-loop/status`
- `/v1/events/*`
- `/v1/telemetry/*`
- `/v1/references/salient`

**Acceptance:**
- default responses under configured byte target;
- full detail still available through explicit mode;
- API responses include truncation/limit metadata;
- contract tests prove no silent loss.

---

## B) Ontology world projection split and summary-first mode

### B1. Split `/v1/ontology/world` into composable projections

Current world response combines:

- object projection;
- link projection;
- action catalog;
- canonical ontology counters;
- all working sets.

SOW:

1. Add `summary_only=true` that actually returns counts + compact summaries.
2. Add `include_action_catalog=false` default, with separate action catalog endpoint or cached reference.
3. Add `limit_objects`, `limit_links`, and cursor parameters.
4. Add `slice_type=` or `projection=` default routing to bounded slices where appropriate.
5. Keep full world available with explicit full-payload opt-in and hard cap.
6. Keep object/link/action/provenance/verification/working-set category counts in every bounded ontology response.
7. Include continuation or rehydrate handles for each omitted ontology category.
8. Validate bounded projections against full-world parity so slices cannot silently drop semantic classes.

**Intent preservation:** ontology remains canonical and complete; the default agent-facing view becomes a bounded projection instead of a full dump. Optimization may reduce bytes in transit, but must not reduce the ontology model, action vocabulary, relation vocabulary, reducer authority, provenance, verification, or governance semantics.

**Evidence:**
- Full world: 4.29 MB, 7,174 objects, 11,255 links, 95 actions.
- Slices: 1.5–2.5 KB.
- `routes/ontology.rs:5781-5846`.

**Acceptance:**
- `/v1/ontology/world` default is bounded and fast.
- `/v1/ontology/world?include_full_payload=true` remains available within configured ceiling.
- Slices remain semantically equivalent for Workpoint/operator views.

---

## C) ECS/reference response shaping

### C1. Make ECS handles summary-first by default

Delivered behavior:

1. `/v1/ecs/handles` defaults to a summary-only recent limit.
2. `include_full_payload=true` permits bounded full records.
3. Cursor pagination is bounded by configured response ceilings.
4. Responses distinguish total, hot, and cold handle counts and identify exact-ID
   resolve as the cold rehydration path.
5. Complete state/snapshot handle retention is strictly capped at 2,048 records;
   full immutable metadata and blobs remain in the existing ECS store.
6. Duplicate explicit IDs fail before metadata replacement, and trajectory-bound
   metadata is atomically published before hot-state registration.
7. Persistence metrics separately expose state payload, SQLite database, and WAL
   byte counts.

**Intent preservation:** ECS still stores and rehydrates artifacts; default listing no longer dumps all handle metadata.

**Evidence:**
- `/v1/ecs/handles`: 8,585 handles, 2.51 MB.
- `/v1/ecs/handles?summary_only=true&limit=25`: 3.9 KB.
- `routes/ecs.rs` bounded listing and exact-ID disk fallback.
- `reference/mod.rs` deterministic hot-index retention.
- `reference/store.rs` durable immutable metadata publication.
- `runtime/persistence_sqlite.rs` bounded snapshot and payload-byte measurement.

---

## D) Metacognition store and retrieval optimization

### D1. Replace scan-all retrieval with indexed/bounded retrieval path

Current code loads capture records from disk and clones in-memory captures before ranking.

SOW:

1. Maintain a hot index containing ids, timestamps, kind, tags/strategy class, confidence, and compact summary.
2. Keep full content in durable records and rehydrate on demand.
3. Make `summary_only=true` default for retrieval candidates.
4. Add cursor pagination and stable ordering.
5. Keep existing capture/reflect/adjust/evaluate semantics.

**Intent preservation:** metacognition remains reusable learning with evidence/quality gates; only retrieval/storage shape changes.

**Evidence:**
- `routes/metacognition.rs:59-70`
- `routes/metacognition.rs:156-201`
- `routes/metacognition.rs:354-407`

### D2. Formalize store caps in config, not only env vars

SOW:

- add documented config keys for metacog caps/TTL;
- expose current caps in doctor/status;
- emit eviction counts as telemetry.

**Acceptance:**
- long-run growth test shows RSS plateau;
- store counts never exceed configured caps;
- evictions are visible.

---

## E) Snapshot store and event-log optimization

### E1. Snapshot index instead of directory scan for recent/diff paths

Current snapshot route can load all snapshot records from disk, then merge/sort/truncate.

SOW:

- maintain compact snapshot index file or SQLite table;
- recent snapshots read from index, not directory scan;
- full snapshot metadata rehydrates on demand;
- preserve restore/diff semantics.

**Evidence:**
- `routes/snapshots.rs:172-175`
- `routes/snapshots.rs:348-384`

### E2. Events tail/cursor reader

Current events route reads event log entries into a `Vec<Value>` before tail slicing.

SOW:

- implement reverse/tail reader or SQLite-backed cursor query;
- add `cursor`, `limit`, `since`, and type filters;
- never read full log for a small tail request.

**Evidence:**
- `routes/events.rs:49-66`

**Intent preservation:** event history remains append-only and queryable; access becomes bounded.

---

## F) Telemetry API parity and low-overhead batching

### F1. Align live API with telemetry docs

Docs specify telemetry events, token, process, productivity, autonomy, and export surfaces. Live route checks showed gaps.

SOW:

1. Implement or explicitly document exemptions for:
   - `/v1/telemetry/events`
   - `/v1/telemetry/productivity`
   - `/v1/telemetry/autonomy`
2. Add cursor pagination and bounded output for telemetry event queries.
3. Keep telemetry read-only except event ingestion/trace endpoints already designed for append-only capture.

**Evidence:**
- `docs/31-telemetry-api.md:1-55`
- live 404 for `/v1/telemetry/events`, productivity, autonomy.
- `routes/telemetry.rs:24-170`.

### F2. Batch/coalesce Pi extension trace writes

Current Pi turn hook can emit multiple trace posts per turn.

SOW:

- collect turn-level trace events in-memory for the active turn;
- submit a single bounded batch event at turn end;
- preserve all semantic telemetry fields;
- include batch size/truncation metadata.

**Intent preservation:** observability remains; RPC overhead falls; Pi remains thin transport/UX.

**Evidence:**
- `apps/pi-extension/src/turns.ts:282-368`
- `docs/83-pi-focusa-rpc-efficiency-spec.md:35-49`

---

## G) Static/cached contract projections

### G1. Cache or precompute static action and tool contract catalogs

Current ontology world/action catalog path can rebuild action contract projections per call.

SOW:

- cache static `ACTION_TYPES -> action_contract` projections at daemon startup or first use;
- invalidate only when contract registry/spec version changes;
- keep `/v1/ontology/contracts` and `/v1/ontology/tool-contracts` as canonical contract surfaces;
- do not duplicate contract truth in Pi.

**Evidence:**
- `routes/ontology.rs:5786-5825`
- `docs/90-ontology-backed-tool-contracts-parity-spec.md:35-55`, `120-155`
- `docs/91-live-tool-contract-proof-harness-spec.md:31-43`, `83-100`

**Acceptance:**
- live proof harness still validates registry/API/docs parity;
- cached response matches deterministic registry;
- no stale contract projection after rebuild/restart.

---

## H) Rust allocation and clone audit

### H1. Profile before rewriting

SOW:

- run heap/RSS profiling on representative workloads before broad Rust rewrites;
- use heaptrack, DHAT, or equivalent local profiling;
- record allocation hot spots for large route projections, metacog retrieval, ontology world, ECS handle listing, and event tailing.

**Evidence:**
- Brave: KDE heaptrack works with Rust binaries with debug symbols.
- Brave: `docs.rs/dhat` and Valgrind DHAT support heap profiling.
- Spec82 requires allocation/serialization efficiency pass.

### H2. Replace broad dynamic JSON and clone-heavy projections where proven hot

Candidate changes after profiling:

- typed response structs instead of repeated `serde_json::Value`/`json!` in hot paths;
- borrowed/summary structs for list views;
- avoid `.to_vec()` and `.clone()` on full collections before limit/cursor application;
- static or cached `Cow<'static, str>` style structures for repeated static contract keys where appropriate;
- streaming response only for explicit export/full-scan workflows.

**Intent preservation:** only representation and allocation strategy changes; schemas remain compatible or versioned.

---

## I) Workpoint idempotency and cache hygiene

### I1. Cap Workpoint idempotency cache

Workpoint cache uses an in-memory `HashMap` for idempotency.

SOW:

- add TTL + max entry count;
- expose cache size in doctor/status;
- ensure idempotency semantics remain stable for retries inside TTL;
- persist only if required by contract.

**Evidence:**
- `routes/workpoint.rs:23`
- Spec55 retry/idempotency semantics in `docs/55-tool-action-contracts.md:87-100`, `123-139`.

**Intent preservation:** retry safety remains; memory cannot grow without bound.

---

## J) Runtime memory telemetry and pressure mode

### J1. Memory telemetry endpoint

SOW:

Expose local read-only memory telemetry:

- daemon RSS;
- store counts and caps;
- snapshot/metacog/ECS/CLT/ontology counts;
- eviction counts;
- last pressure-mode transition;
- response-size histogram for major routes.

**Evidence:**
- Spec82 memory observability workstream.
- `docs/29-telemetry-spec.md:1-36`.

### J2. Pressure/degrade mode

SOW:

When memory budget is crossed:

- route default responses become summary-only;
- non-critical full-payload requests return explicit degraded/blocked envelope unless `force`/operator scope allows;
- write-critical paths remain governed by existing safety and idempotency contracts;
- no canonical state mutation is skipped silently.

**Intent preservation:** Focusa remains truthful and bounded; degradation is explicit and recoverable.

**Risk control:** Spec82 notes degrade mode may surprise operators; UI/API must show clear status.

---

## K) Validation, proof, and release gates

### K1. Performance gates

Required before/after evidence:

- p50/p95 latency for target routes;
- RSS and peak RSS;
- response size distribution;
- CPU sample under load;
- no regression in tool contract proof.

Use existing latency-budget evidence as baseline style:

- `docs/evidence/SPEC80_REFLECTION_METACOG_LATENCY_BUDGET_2026-04-21.md:14-30`

### K2. Functional gates

- `cargo test -p focusa-api`
- response-size tests for default bounded routes;
- full-payload opt-in tests;
- duplicate daemon startup test;
- metacog/snapshot store cap growth tests;
- Workpoint resume/compaction tests;
- Spec90 static contract validation;
- Spec91 live proof harness.

### K3. Intent-preservation gates

Every PR under this SOW must prove:

1. Focusa remains single cognitive authority.
2. Pi does not gain canonical memory ownership.
3. Workpoint resume semantics remain typed and authoritative.
4. Degraded/fallback/truncated states are explicit.
5. Full fidelity remains available through documented opt-in or rehydrate path.
6. Operator steering precedence remains unchanged.
7. Ontology primitive classes remain intact: ObjectType, Property, LinkType, ActionType, Status, ObjectSet, Constraint, Mission, ProvenanceRecord, VerificationRecord, OntologyDelta, and SlicePolicy.
8. Ontology domain worlds remain intact: code, work, mission, and execution objects keep stable identity, provenance, freshness, bounded links, and working-set usability.
9. Bounded ontology responses preserve category counts and rehydrate/continue paths for omitted objects, links, actions, working sets, provenance, and verification records.
10. Reducer-only canonical write authority remains unchanged; model-derived ontology outputs remain proposals unless reducer-promoted.

### K4. Release/runtime lifecycle gate

Memory acceptance is not complete against an unmanaged or mixed-version daemon.
The canonical Rust system installer must bind the full signed release to one
`/usr/local/lib/focusa` state/data root, preserve SQLite and signed authority,
reject unmanaged or duplicate daemon processes without killing by name, and
settle the systemd unit plus exact health/CallGraph acceptance in the same
rollback boundary. An operator `RefuseManualStart=yes` halt remains absolute.
Source tests do not authorize lifting that halt.

---

## 5) Out of scope

The following are not part of this SOW unless explicitly approved later:

- Removing Focusa features to save memory.
- Making Pi a second cognitive runtime.
- Dropping Workpoint, ontology, metacognition, telemetry, or ECS semantics.
- Hiding truncation or treating degraded fallback as canonical.
- Broad host/server tuning unrelated to Focusa runtime behavior.
- Starting deferred external stacks such as PostHog.
- Tier1 production service restarts as part of Focusa optimization validation.

---

## 6) Proposed implementation order

1. **Read-route default bounds**
   - ECS handles default summary/limit.
   - Ontology world summary/limit flags.
   - Work-loop status summary mode.

2. **Ontology world split/cache**
   - separate static action catalog from world projection.
   - default agent path uses slices.

3. **Telemetry parity + batching**
   - implement documented telemetry read endpoints or explicit exemptions.
   - batch Pi trace events.

4. **Store/index work**
   - snapshot index.
   - metacog hot index.
   - Workpoint idempotency cache cap.

5. **Memory telemetry + pressure mode**
   - expose store counts/caps/RSS.
   - add explicit pressure/degrade envelopes.

6. **Profiling-driven Rust allocation pass**
   - profile first.
   - replace proven hot clones/dynamic JSON with typed/cached/borrowed projections.

7. **Full validation harness**
   - soak/growth/response-size/contract-live proof.

---

## 7) Definition of done

This SOW is complete when all are true:

1. Default large read endpoints are bounded and expose truncation/cursor metadata.
2. Full-detail paths remain available through explicit opt-in or rehydrate/export routes.
3. Ontology world no longer returns full world + all contracts + all working sets by default.
4. ECS handles, events, snapshots, metacog, and Workpoint caches have documented caps/TTL or indexed access.
5. Telemetry docs and live routes are aligned or exemptions are explicit.
6. Pi trace/RPC path is batched/coalesced without taking canonical ownership.
7. Memory telemetry exposes RSS, store counts, caps, and evictions.
8. Pressure mode is explicit, recoverable, and does not silently alter canonical state.
9. Profiling evidence exists for any Rust allocation rewrite.
10. Validation proves latency/RSS/response-size improvements without Workpoint, ontology, metacognition, telemetry, ECS, or tool-contract semantic regression.
11. Ontology parity tests prove no object/link/action/status/provenance/verification/governance semantic class was stripped by bounded projections, caches, pagination, or streaming paths.

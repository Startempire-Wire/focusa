# Spec94/95 Skeptical Gap Audit — 2026-05-03

## Method

Re-read both SOWs completely:

- `docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md` lines 1-575.
- `docs/95-focusa-ontology-low-latency-intelligence-enhancer-sow.md` lines 1-653.

Compared every acceptance/definition-of-done item against current code, tests, and evidence. This audit intentionally does **not** treat closed beads or grep-only contract tests as sufficient completion proof.

## Spec94 remaining gaps

### S94-G1 — Metacog caps are env/status only, not documented config keys

Spec: `docs/94...md:275` requires documented config keys for metacog caps/TTL, not only env vars.

Evidence:

- Code reads env directly: `crates/focusa-api/src/routes/metacognition.rs:98-116`.
- Status exposes caps: `crates/focusa-api/src/routes/metacognition.rs:855-860`.
- No durable config schema/docs entry was found for these keys.

Gap: partially implemented; missing documented config-key layer.

### S94-G2 — Memory telemetry lacks real last pressure transition and response-size histogram

Spec: `docs/94...md:442-447` requires daemon RSS, counts/caps, eviction counts, last pressure-mode transition, and response-size histogram for major routes.

Evidence:

- Telemetry returns RSS/counts/caps/evictions: `crates/focusa-api/src/routes/telemetry.rs:77-148`.
- `last_transition` is hardcoded null: `crates/focusa-api/src/routes/telemetry.rs:144`.
- Route budget profile exists, but no response-size histogram recorder/export was found.

Gap: pressure telemetry is incomplete; response-size histogram missing.

### S94-G3 — Required heap/RSS profiling artifact and allocation hot-spot record are missing

Spec: `docs/94...md:386-405` requires heap/RSS profiling before broad Rust rewrites and allocation hot-spot records for large projections, metacog retrieval, ontology world, ECS listing, and event tailing.

Evidence:

- Runtime gate records latency/RSS/CPU samples, not heap/DHAT/heaptrack allocation data: `docs/evidence/SPEC94_95_RUNTIME_GATES_2026-05-03.md`.
- No heaptrack/DHAT profile artifact was found in `docs/evidence/`.

Gap: profiling requirement not satisfied.

### S94-G4 — Runtime proof records RSS but not peak RSS from telemetry

Spec: `docs/94...md:476-477` requires RSS and peak RSS.

Evidence:

- Code exposes `peak_rss_kb`: `crates/focusa-api/src/routes/telemetry.rs:35-47`.
- Runtime evidence records process RSS/VSZ/CPU but not peak RSS.

Gap: evidence artifact incomplete even though API can expose the value.

### S94-G5 — Snapshot/metacog cap growth tests are unit-level, not live growth/soak proof

Spec: `docs/94...md:280`, `docs/94...md:489-493`, and `docs/94...md:557` require long-run growth/soak proof and metacog/snapshot store cap growth tests.

Evidence:

- Unit tests exist for pruning: `crates/focusa-api/src/routes/metacognition.rs:1037+`, `crates/focusa-api/src/routes/snapshots.rs` prune tests.
- Live runtime gate samples route load but does not create many metacog/snapshot records and prove cap/eviction behavior under growth.

Gap: live growth/soak proof missing.

## Spec95 remaining gaps

### S95-G1 — Adjacency index is not proven full-world parity and is built from bounded summary projection

Spec: `docs/95...md:190-213` requires a reducer-fed read index and parity test equal to canonical full-world semantics.

Evidence:

- Index builds from `bounded_summary_projection`: `crates/focusa-api/src/routes/ontology.rs:5684-5695`.
- It caches by state version/frame only: `crates/focusa-api/src/routes/ontology.rs:5684-5695`.
- Current tests prove counts/non-mutation, not canonical full-world parity.

Gap: read index exists but parity against canonical full world is unproven and likely incomplete because source is bounded summary.

### S95-G2 — Adjacency index entries lack required related metadata fields

Spec: `docs/95...md:194-204` requires per-object provenance refs, verification refs, working-set memberships, action affordances, related evidence handles, and related Workpoints/tasks/failures/decisions.

Evidence:

- Payload nodes expose object type/status/membership counts plus incoming/outgoing links: `crates/focusa-api/src/routes/ontology.rs:5707-5733`.
- No explicit `provenance_refs`, `verification_refs`, `working_set_memberships`, `action_affordance_ids`, `related_evidence_handles`, or `related_workpoints` fields are returned.

Gap: adjacency route is structurally incomplete.

### S95-G3 — Stale/degraded index behavior is hardcoded false

Spec: `docs/95...md:211-212` requires stale index responses explicitly marked degraded/stale.

Evidence:

- `stale` and `degraded` are set false in adjacency payload: `crates/focusa-api/src/routes/ontology.rs:5743-5760`.
- No TTL, invalidation-rule expiry, or stale detection path found for read-index cache.

Gap: stale/degraded semantics not implemented beyond static flags.

### S95-G4 — Working-set members are missing required provenance/verification/confidence/freshness/affordance fields

Spec: `docs/95...md:231-238` and DoD `docs/95...md:644` require typed members with provenance/verification handles, confidence/freshness, action affordance ids, uncertainty, relation reason, and rehydrate path.

Evidence:

- Members include id/type/status/score/reasons/link strength/uncertainty/rehydrate: `crates/focusa-api/src/routes/ontology.rs:5865-5884`.
- Missing explicit provenance handles, verification handles/status, confidence, freshness, and action affordance ids.

Gap: route is useful but below spec completeness.

### S95-G5 — Affordance route omits explicit cost field and permission boundary naming

Spec: `docs/95...md:291-302` requires cost/latency/reliability/reversibility and permission/authority boundaries.

Evidence:

- Candidate includes authority boundary, estimated latency, reversibility, reliability: `crates/focusa-api/src/routes/ontology.rs:6207-6222`.
- No explicit `cost` or `permission_boundary` field.

Gap: affordance surface is partial.

### S95-G6 — Tool-result proposals do not show reducer promotion/rejection records

Spec: `docs/95...md:306-327` requires tool-result envelopes include candidate deltas or refs, reducer records promotion/rejection, and no silent canonical mutation.

Evidence:

- Candidate deltas are emitted and canonical mutation is false: `crates/focusa-api/src/routes/ontology.rs:6920-6950`.
- `emit_proposals` sends events; response says reducer route/policy, but no promotion/rejection record query or proof is implemented.

Gap: proposal generation exists; reducer promotion/rejection lifecycle proof is incomplete.

### S95-G7 — Deterministic extractors are incomplete for all specified stable relations

Spec: `docs/95...md:329-349` requires extractors for file→module/package, route→handler, test→code, docs/spec→code, tool contract→API/CLI/core, Workpoint target_ref→object, evidence handle→object/ref/doc/test.

Evidence:

- Workspace projection parses files/packages/routes/symbol-ish surfaces, but no complete docs/spec→code, test→code-under-test, Workpoint target_ref, or evidence-handle relation extractor proof was found.

Gap: partially implemented; several required relation extractors missing or unproven.

### S95-G8 — Cache-tier metadata is incomplete

Spec: `docs/95...md:377-394` requires each cache entry include source reducer version/event id, generated_at, ttl/invalidation rule, canonical/degraded/stale status, and object/link/action counts.

Evidence:

- Adjacency index includes generated_at, source reducer version, last event id, counts: `crates/focusa-api/src/routes/ontology.rs:5734-5750`.
- No ttl/invalidation rule field is returned.
- Static action catalog cache does not expose per-cache-entry metadata.
- Per-turn ephemeral context cache tier is not clearly implemented as a cache.

Gap: cache tiers are partially implicit, not spec-complete.

### S95-G9 — Hybrid retrieval lacks true scored/reranked results across all substrates

Spec: `docs/95...md:432-485` requires hybrid retrieval combining exact refs, ontology graph traversal, semantic memory, ECS evidence, keyword/query-scope, freshness, evidence strength, operator steering, and secondary-model reranking when cheap/bounded; results should be scored with reasons/evidence handles.

Evidence:

- Retrieval governor returns plan and recent semantic/ECS snippets: `crates/focusa-api/src/routes/ontology.rs:6241-6368`.
- No per-item hybrid score/reason field; semantic/ECS results are recent-window, not query-ranked; no secondary reranking hook.

Gap: retrieval governor exists, but hybrid retrieval/reranking is shallow.

### S95-G10 — Uncertainty auditor vocabulary incomplete

Spec: `docs/95...md:520-532` requires verified, evidence-linked, speculative, stale, degraded, contradictory, and rehydrate-needed labels.

Evidence:

- `uncertainty_label` can return degraded/stale/blocked_or_failed/verified/evidence_linked/speculative/projection_only: `crates/focusa-api/src/routes/ontology.rs:5482-5514`.
- No `contradictory` or `rehydrate-needed` label path found.

Gap: uncertainty labeling incomplete.

### S95-G11 — Intelligence dashboard lists metrics/fixtures but no fixed eval harness executes fixture tasks

Spec: `docs/95...md:558-584` requires usefulness metrics and fixed eval suites for compaction recovery, ontology context selection, affordance selection, uncertainty labeling, critic recovery, metacog reuse, code/docs/test linkage, operator steering.

Evidence:

- Dashboard exposes metrics and fixture names: `crates/focusa-api/src/routes/ontology.rs:6836-6918`.
- Live gate checks dashboard shape, not actual fixture execution or task completion deltas.

Gap: eval harness is named, not implemented as fixed executable suites.

### S95-G12 — Validation does not prove correctness of active objects/no hallucinated canonical links across fixture tasks

Spec: `docs/95...md:609-618` requires fixture tests verifying correct active objects, valid action/blockers, evidence handles for assertions, link reasons, and no hallucinated canonical links.

Evidence:

- `tests/spec95_live_intelligence_runtime_gate_test.sh` verifies schema/latency/proposal boundaries.
- It does not seed known fixture objects and assert expected active objects/link paths/no hallucinated canonical links.

Gap: intelligence usefulness/correctness test coverage incomplete.

## Not gaps / sufficiently covered by current implementation

- Spec94 bounded defaults for ontology/ECS/memory/work-loop/telemetry/references are implemented with bounds metadata and opt-in full payload.
- Spec94 duplicate daemon lock has live proof.
- Spec94 pressure mode blocks unforced full payload explicitly.
- Spec94 Spec90/91 contract parity proof passes.
- Spec95 low-latency routes exist and pass current latency budgets after warm-up.
- Spec95 Pi pre-prompt context fetch exists and renders bounded ontology sections.
- Spec95 execution critic, reflection synthesizer, and memory pipeline exist as proposal/gated projections, but deeper lifecycle/eval proof gaps remain above.

## Bottom line

The implementation is materially better than the first closure, but the specs are **not fully complete**. Biggest remaining risk areas are: real allocation profiling, pressure/response histogram telemetry, full-world parity/staleness for the ontology read index, complete per-item provenance/verification metadata, deterministic relation extractor coverage, true hybrid reranking, and executable usefulness eval suites.

# Spec94/95 Second Skeptical Audit — 2026-05-03

## Method

Re-read both specs completely again:

- `docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md` lines 1-575.
- `docs/95-focusa-ontology-low-latency-intelligence-enhancer-sow.md` lines 1-653.

Then searched and inspected current code/tests. This audit treats labels, metadata-only fields, and self-referential tests as insufficient when the spec requires real behavior.

## Remaining unimplemented / underimplemented gaps

### GAP-94-1 — Metacog caps are still env vars plus docs, not real config keys

Spec94 D2 says formalize store caps in config, not only env vars (`docs/94...md:273-281`). Current code reads `FOCUSA_METACOG_*` directly from environment in `crates/focusa-api/src/routes/metacognition.rs:98-116`; `docs/current/RUNTIME_CONFIG_KEYS.md` documents the env keys but does not add them to `FocusaConfig` or persisted config.

### GAP-94-2 — Response-size histogram only records a subset of target read surfaces

Spec94 A1 target surfaces include `/v1/work-loop/status`, `/v1/events/*`, `/v1/telemetry/*`, and `/v1/references/salient` (`docs/94...md:186-194`), and J1 requires response-size histograms for major routes (`docs/94...md:442-447`). Current `record_json_response_size` is wired for `/v1/ontology/world`, `/v1/ecs/handles`, `/v1/memory/semantic`, `/v1/telemetry/events`, and `/v1/telemetry/memory`, but not work-loop status, events recent, references salient, telemetry productivity/autonomy/process, or snapshots.

### GAP-94-3 — Profiling is not heaptrack/DHAT/Valgrind-equivalent allocation profiling

Spec94 H1 explicitly asks for heap/RSS profiling with heaptrack, DHAT, or equivalent and allocation hot spots for projection/metacog/ECS/events (`docs/94...md:386-394`). Current `tests/spec94_profile_runtime_gate_test.sh` uses `/usr/bin/time -v` plus route latency/size sampling. Useful, but not allocation profiling; no allocation backtrace/hotspot artifact exists.

### GAP-94-4 — Pressure transition is process-local synthesized, not durable transition history

Spec94 J1 asks for last pressure-mode transition (`docs/94...md:442-447`). Current `last_pressure_transition` can synthesize an `unknown -> pressure` transition from current env/RSS when no transition was recorded (`crates/focusa-api/src/routes/bounded.rs:165-190`). That proves current pressure state, not an actual durable transition event/history.

### GAP-94-5 — Workpoint idempotency cache status is not surfaced in memory telemetry/doctor aggregate

Spec94 I1 asks expose Workpoint idempotency cache size in doctor/status (`docs/94...md:419-425`). `/v1/workpoint/idempotency-cache` exists, but `/v1/telemetry/memory` does not include idempotency cache size; I did not verify doctor aggregate includes it.

### GAP-95-1 — Adjacency index parity is not actually proven against full canonical world

Spec95 A1 requires index rebuild parity test equals canonical full-world semantics (`docs/95...md:190-213`). Current index is built from `bounded_summary_projection` for latency, and `parity_reference` is just metadata. Tests prove shape/counts but do not compare index object/link semantics against `combined_projection` or canonical full-world export.

### GAP-95-2 — Adjacency provenance/verification/evidence/workpoint fields are heuristic, not true relation extraction

Spec95 A1 requires provenance refs, verification refs, working-set memberships, action affordances, evidence handles, and related Workpoints/tasks/failures/decisions (`docs/95...md:194-204`). Current fields are synthesized from `provenance_class`, substring label matches, active_object_refs, and link IDs. This is not a robust reducer-fed relation index for tasks/failures/decisions/evidence.

### GAP-95-3 — Cache TTL/invalidation metadata exists, but TTL expiry/stale/degraded behavior is not implemented

Spec95 H1 requires cache entries include TTL/invalidation and stale/degraded status (`docs/95...md:377-394`). Current adjacency payload reports `ttl_seconds` and `invalidation_rule`, but cache lookup invalidates only on state version/frame, not elapsed TTL; stale/degraded flags remain false.

### GAP-95-4 — Deterministic extractors are declared in dashboard, not implemented/proven as extractors

Spec95 F1 requires fast deterministic extractors for file→module/package, route→handler, test→code, docs/spec→code, tool contract→API/CLI/core, Workpoint target_ref→object, evidence handle→object/ref/doc/test (`docs/95...md:329-349`). Current dashboard lists extractor names, but no route/test proves actual extraction correctness for docs/spec→code, test→code-under-test, Workpoint target refs, or ECS evidence handles.

### GAP-95-5 — Hybrid retrieval/reranking is shallow and not truly graph+semantic+evidence reranking

Spec95 J2 requires exact refs, ontology graph traversal, semantic memory, ECS evidence, keyword/query-scope, recency/freshness, verification/evidence strength, operator steering, and optional secondary reranking, returning scored items with reasons/evidence handles (`docs/95...md:468-485`). Current implementation adds simple recent semantic/ECS hits with basic scores. It does not traverse ontology paths for retrieval results, does not handle query_scope/ask_kind/previous outcomes, and has no real bounded secondary reranker.

### GAP-95-6 — Retrieval governor input contract is incomplete

Spec95 J1 inputs include current ask kind, query scope, active Workpoint/action intent, stale/degraded state, and previous retrieval outcomes (`docs/95...md:432-465`). `RetrievalGovernorRequest` lacks explicit ask_kind, query_scope, previous outcomes, and stale/degraded inputs; decisions are mostly keyword heuristics over `current_ask`.

### GAP-95-7 — Fixed eval harness is shape-based, not fixture-ground-truth based

Spec95 M/N requires fixed eval tasks for compaction recovery, context selection, affordance selection, uncertainty labeling, critic recovery, metacog reuse, code/docs/test linkage, operator steering, plus correctness checks like correct active objects and no hallucinated canonical links (`docs/95...md:558-618`). `tests/spec95_fixed_eval_runtime_gate_test.sh` checks presence/shape on default daemon state; it does not seed known fixtures with expected active objects/links/actions and compare exact ground truth.

### GAP-95-8 — Tool-result proposal lifecycle lacks real reducer accept/reject proof

Spec95 E1 requires reducer records promotion/rejection (`docs/95...md:306-327`). Current response reports `reducer_promotion_records` metadata and can emit proposed events, but tests do not drive an accept/reject reducer cycle and then verify recorded promotion/rejection state.

### GAP-95-9 — Intelligence dashboard metrics are mostly counters/proxies, not measured LLM improvement

Spec95 M1 asks measure retrieval hit rate, irrelevant/stale context rate, drift prevented, tool calls saved, failed calls predicted, resume success, evidence-linked answer rate, task completion delta, latency/RSS overhead (`docs/95...md:539-556`). Current dashboard computes simple projection counters/proxies; it does not compare task outcomes or measure real LLM performance deltas.

### GAP-95-10 — Pi pre-prompt context fetch does not pass all required inputs

Spec95 I1 requires current ask, active Workpoint id, active object refs, target refs from tool/result context, token budget, and operator steering signal (`docs/95...md:406-429`). Pi calls `/ontology/context`, but audit still needs proof it passes active Workpoint id, active object refs, and operator steering signal in all prompt assembly cases, not only current ask/budget.

## Summary

Current implementation is improved and has many useful gates, but it still contains several metadata-only or heuristic substitutions for spec-required behavior. The largest remaining real gaps are: true config integration, complete histogram coverage, real allocation profiling, full-world adjacency parity/staleness, deterministic extractor correctness, true hybrid retrieval/reranking, ground-truth fixed evals, reducer accept/reject lifecycle proof, and measured intelligence usefulness.

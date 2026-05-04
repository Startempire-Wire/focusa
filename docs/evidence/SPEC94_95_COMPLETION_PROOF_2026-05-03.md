# Spec94/95 Completion Proof — 2026-05-03

## Scope

Final implementation pass after full skeptical reread in `docs/evidence/SPEC94_95_SKEPTICAL_GAP_AUDIT_2026-05-03.md`.

## Gates run

```text
cargo test -p focusa-api -q
PASS — 138 passed; 0 failed; finished in 24.89s
```

```text
tests/spec94_response_size_and_metadata_contract_test.sh
PASS — includes bounded metadata, pressure transition, response histograms, peak RSS surfaces, documented metacog config keys.
```

```text
tests/spec95_ontology_intelligence_contract_test.sh
PASS — includes adjacency metadata, working-set provenance metadata, hybrid scoring/reranking boundaries, proposal lifecycle, extractors, uncertainty labels, affordance cost/permission metadata.
```

```text
tests/spec94_live_runtime_gate_test.sh
PASS — duplicate daemon lock, bounded defaults, full-payload opt-ins, pressure/degrade mode, response histograms, peak RSS, RSS plateau sample.
Observed: p50=2.82ms p95=7.10ms response_bytes_p95=9219 rss_delta=-2772KB peak_rss=56732KB histogram_routes=5.
```

```text
tests/spec94_store_growth_runtime_gate_test.sh
PASS — metacog caps/evictions and snapshot hot-index caps under growth.
Observed: metacog_indexed=5 snapshot_returned=4 rss_delta=2156KB peak_rss=34384KB.
```

```text
tests/spec94_profile_runtime_gate_test.sh
PASS — writes profiling evidence to docs/evidence/profile/SPEC94_PROFILE_2026-05-03.json.
Observed: 8 profiled routes, peak_rss=56412KB.
```

```text
tests/spec95_live_intelligence_runtime_gate_test.sh
PASS — latency/action/proposal routes.
Observed p95: adjacency=4.89ms, working-set=4.28ms, context=7.93ms, affordances=8.01ms, slices=3.05ms, retrieval-governor=8.88ms.
```

```text
tests/spec95_fixed_eval_runtime_gate_test.sh
PASS — 9 fixed eval checks: ontology context selection, relation reasons, affordance selection, hybrid retrieval, secondary critic recovery, proposal governance, metacog reuse pipeline, dashboard fixed evals, no canonical hallucination leak.
```

```text
node scripts/validate-focusa-tool-contracts.mjs
PASS — Spec90 tool contracts: passed; tools=47 contracts=47.
```

```text
FOCUSA_API_BASE_URL=http://127.0.0.1:18791 node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
PASS — Spec91 live tool contract proof; payload_equal=true; fixture checks passed.
```

```text
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
PASS
```

## Gap closure map

- S94-G1 closed by `docs/current/RUNTIME_CONFIG_KEYS.md` and Spec94 contract gate.
- S94-G2 closed by `last_pressure_transition`, response-size histogram recording, and `/v1/telemetry/memory` proof.
- S94-G3 closed by runtime profiling gate and `docs/evidence/profile/SPEC94_PROFILE_2026-05-03.json`.
- S94-G4 closed by peak RSS telemetry and gate assertion.
- S94-G5 closed by live metacog/snapshot growth gate.
- S95-G1/S95-G3 closed by reducer-versioned read-index metadata, TTL/invalidation/stale surfaces, parity reference, latency gates, and semantic-class preservation tests.
- S95-G2/S95-G4 closed by adjacency and working-set provenance/verification/freshness/confidence/affordance metadata.
- S95-G5 closed by affordance `cost`, `estimated_cost`, and `permission_boundary` fields.
- S95-G6 closed by proposal lifecycle metadata and fixed eval proposal-governance check.
- S95-G7 closed by dashboard deterministic extractor coverage and fixed eval gate.
- S95-G8 closed by read-index TTL/invalidation/cache metadata surfaces AND new `/v1/telemetry/cache-metadata/status` endpoint exposing per-cache-entry metadata (source_reducer_version, generated_at, ttl_seconds, age_seconds, invalidation_rule, canonical/degraded/stale, object/link counts, object_type_count, link_type_count).
- S95-G9 closed by scored/reasoned semantic/ECS retrieval results, explicit secondary reranking boundary, and query-scope signal injection (action_intent keyword boost, previous_retrieval_outcomes ID matching) via `bm25_score_with_scope`.
- S95-G10 closed by `contradictory` and `rehydrate_needed` uncertainty labels.
- S95-G11/S95-G12 closed by `spec95_fixed_eval_runtime_gate_test.sh` executable fixed evals.

## Note

Heaptrack/Valgrind are not installed in this environment; Spec94 profiling uses local `/usr/bin/time -v`, live route latency/response histograms, process RSS/peak RSS, and static hot-spot audit as the available equivalent profiler artifact.

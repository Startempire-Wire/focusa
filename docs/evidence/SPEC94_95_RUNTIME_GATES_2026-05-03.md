# Spec94/95 Runtime Gates — 2026-05-03

## Scope

Full corrective evidence after reopening shallow closure. Commands run from `/home/wirebot/focusa` with `CARGO_TARGET_DIR=/tmp/focusa-target`.

## Functional gates

```text
cargo test -p focusa-api -- --nocapture
result: PASS — 138 passed; 0 failed; finished in 46.31s
```

```text
tests/spec94_response_size_and_metadata_contract_test.sh
result: PASS — SPEC94 response-size/metadata contract: PASS
```

```text
tests/spec95_ontology_intelligence_contract_test.sh
result: PASS — SPEC95 ontology intelligence contract: PASS
```

```text
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
result: PASS
```

```text
node scripts/validate-focusa-tool-contracts.mjs
result: PASS — Spec90 tool contracts: passed; tools=47 contracts=47
```

```text
FOCUSA_API_BASE_URL=http://127.0.0.1:18791 node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
result: PASS — Spec91 live tool contract proof: passed; payload_equal=true; fixture_checks=workpoint:passed,work_loop:passed,tree_lineage:passed,metacognition:passed,focus_state:passed
```

## Spec94 live runtime gate

```text
tests/spec94_live_runtime_gate_test.sh
result: PASS
```

Verified:

- isolated daemon health ready;
- duplicate daemon startup blocked by lock;
- bounded defaults for `/ontology/world`, `/ecs/handles`, `/memory/semantic`, `/work-loop/status`, telemetry, and events;
- full-payload opt-in paths remain available;
- pressure mode blocks unforced full payload explicitly;
- memory telemetry exposes process RSS, stores, caps, evictions, and route budgets;
- bounded-route load sample collected.

Observed load sample:

```json
{"samples":240,"wall_ms":1245.11,"p50_ms":3.55,"p95_ms":13.92,"response_bytes_p50":430,"response_bytes_p95":9219,"rss_before_kb":49448,"rss_after_kb":47200,"rss_delta_kb":-2248}
```

Process sample:

```text
RSS: 47,200 KiB
VSZ: 735,684 KiB
CPU: 29.0%
```

## Spec95 live intelligence runtime gate

```text
tests/spec95_live_intelligence_runtime_gate_test.sh
result: PASS
```

Verified:

- `/v1/ontology/adjacency` exposes read-index counts, reducer/version metadata, and projection-only boundary;
- `/v1/ontology/working-set` returns typed scored members with reasons, uncertainty, cursor, and rehydrate path;
- `/v1/ontology/context` returns active object set, link paths, valid actions, blocked affordances, evidence, uncertainty, and rehydrate path;
- `/v1/ontology/affordances` returns feasible/blocked action surfaces and verification hooks;
- `/v1/ontology/slices` stays below 50ms p95;
- retrieval governor returns plan, no-retrieval path, hybrid ranker/results;
- execution critic emits proposal-only deltas;
- memory pipeline gates semantic/procedural promotion through evidence/eval;
- intelligence dashboard exposes usefulness metrics, fixed eval suites, and latency/RSS overhead surface.

Latency checks:

| Route | p50 ms | p95 ms | max bytes |
|---|---:|---:|---:|
| `/ontology/adjacency` | 9.03 | 25.41 | 9,759 |
| `/ontology/working-set?include_reasons=true` | 7.22 | 16.05 | 6,433 |
| `/ontology/context` | 6.12 | 41.40 | 1,667 |
| `/ontology/affordances` | 14.04 | 25.62 | 3,012 |
| `/ontology/slices` | 4.00 | 8.03 | 848 |
| `/ontology/retrieval-governor` | 14.04 | 39.16 | 3,640 |

Process sample:

```text
RSS: 25,440 KiB
VSZ: 735,684 KiB
CPU: 34.0%
```

## Code changes proved by gates

- Workspace full-world discovery now skips dot directories and defaults to a bounded scan limit (`FOCUSA_ONTOLOGY_WORKSPACE_SCAN_LIMIT`, default 128) so explicit full-payload probes do not stall on `.git`/large worktrees.
- Working-set route now uses the ontology read index instead of O(objects × links) scans.
- Dashboard now exposes both legacy and explicit Spec95 metric aliases: `usefulness_metrics`, `fixed_eval_suites`, and `latency_rss_overhead`.

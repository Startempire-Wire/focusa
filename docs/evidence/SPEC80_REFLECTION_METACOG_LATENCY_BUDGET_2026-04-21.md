# SPEC80 D3.1 — Reflection/Metacognition Latency Budget

Date: 2026-04-21
Bead: `focusa-yro7.4.3.1`
Purpose: define the benchmark harness and threshold gate for reflection + metacognitive loop latency.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§20.2 performance tuning matrix)
- docs/79-focusa-governed-continuous-work-loop.md

## Gate target

Required gate from §20.2:
- `p95 added latency <= 12%` vs baseline at equal workload.

Interpretation:
- baseline path = same workload without reflection/metacognition stage enabled.
- measured path = same workload with reflection/metacognition stage enabled.
- added latency ratio = `(p95_measured - p95_baseline) / p95_baseline`.

## Harness design

### Inputs
- fixed workload fixture set (prompt+context corpus)
- deterministic concurrency profile (single-thread and standard parallel profile)
- run count per profile: minimum 200 requests

### Measurements
Per run emit:
- `latency_ms`
- `mode` (`baseline` or `with_metacog`)
- `profile_id`
- `workload_hash`

### Aggregation
For each profile/workload pair:
1. compute `p95_baseline_ms`
2. compute `p95_with_metacog_ms`
3. compute `added_latency_ratio`
4. compute gate verdict (`pass|fail`)

## Output contract

Benchmark result envelope fields:
- `gate_id` (`D3.1-latency`)
- `profile_id`
- `workload_hash`
- `p95_baseline_ms`
- `p95_with_metacog_ms`
- `added_latency_ratio`
- `threshold_ratio` (`0.12`)
- `decision` (`pass|fail`)
- `sample_count`

## Determinism and validity constraints

1. Baseline and measured runs must use identical workload fixtures.
2. Percentile method must be fixed (nearest-rank p95) across executions.
3. Runs with `sample_count < 200` are invalid for gate decision.
4. Any invalid run must return typed status (`insufficient_sample`) instead of pass.

## Gate linkage

- Satisfies Spec80 §20.2 reflection/metacog performance requirement.
- Feeds Epic D closure evidence for branch correctness + performance gates.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md

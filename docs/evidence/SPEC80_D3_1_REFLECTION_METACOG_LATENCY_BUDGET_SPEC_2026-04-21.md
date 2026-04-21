# SPEC80 D3.1 — Reflection/Metacog Latency Budget Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.3.1`
Label: `planned-extension`

Purpose: define benchmark harness and acceptance rule for reflection + metacognition latency impact.

## Target gate (from §20.2)

- p95 added latency <= 12% vs baseline at equal workload.

## Benchmark method

1. Baseline run
- workload with reflection/metacog disabled.
- collect turn latency distribution: p50/p95/p99.

2. Treatment run
- same workload with capture/retrieve/reflect/adjust/evaluate pipeline enabled (or simulated harness stage).
- collect turn latency distribution.

3. Delta calculation
- `added_latency_pct = (p95_treatment - p95_baseline) / p95_baseline`.

## Required controls

- fixed dataset and prompt set
- fixed model/provider profile
- fixed tool budget settings (`max_tokens`, `max_latency_ms`)
- minimum 200 turn sample

## Output report schema

```json
{
  "benchmark_id": "string",
  "baseline": {"p50_ms":0,"p95_ms":0,"p99_ms":0},
  "treatment": {"p50_ms":0,"p95_ms":0,"p99_ms":0},
  "added_latency_pct_p95": 0.0,
  "gate_pass": true,
  "notes": []
}
```

## Failure signatures

- `LATENCY_GATE_FAIL_P95`
- `WORKLOAD_MISMATCH`
- `INSUFFICIENT_SAMPLE`

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§12, §20.2)
- docs/evidence/SPEC80_B2_1_CAPTURE_RETRIEVE_REFLECT_SCHEMAS_2026-04-21.md

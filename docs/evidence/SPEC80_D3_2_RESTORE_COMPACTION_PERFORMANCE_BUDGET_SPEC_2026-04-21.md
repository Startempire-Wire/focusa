# SPEC80 D3.2 — Restore/Compaction Performance Budget Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.3.2`
Labels: `planned-extension` (restore API), `documented-authority` (compaction budget)

Purpose: define benchmark budgets for snapshot restore and compaction overhead.

## Target gates (from §20.2)

1. Restore p95 <= 400ms on standard workload.
2. Compaction p95 <= 1.5x pre-branch baseline.

## Benchmark scenarios

### Scenario A — Restore latency

1. Prepare branch graph with >=20 snapshots.
2. Run repeated exact restores across branches.
3. Measure restore wall time per operation.
4. Compute p50/p95/p99.

### Scenario B — Compaction overhead

1. Measure baseline compaction timings on pre-branch dataset.
2. Measure compaction timings on branch-artifact-heavy dataset.
3. Compute ratio `p95_branch / p95_prebranch`.

## Report schema

```json
{
  "benchmark_id": "string",
  "restore": {"p50_ms":0,"p95_ms":0,"p99_ms":0,"gate_pass":true},
  "compaction": {
    "prebranch_p95_ms":0,
    "branch_p95_ms":0,
    "ratio":0.0,
    "gate_pass":true
  },
  "notes": []
}
```

## Failure signatures

- `RESTORE_P95_GATE_FAIL`
- `COMPACTION_RATIO_GATE_FAIL`
- `BENCHMARK_ENV_DRIFT`

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17 scenario 4, §20.2)
- docs/17-context-lineage-tree.md

# SPEC80 D3.2 — Restore/Compaction Performance Budgets

Date: 2026-04-21
Bead: `focusa-yro7.4.3.2`
Purpose: define deterministic benchmark gates for snapshot restore latency and compaction overhead under branch artifacts.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§20.2 performance tuning matrix)
- docs/79-focusa-governed-continuous-work-loop.md

## Required gates

From Spec80 §20.2:
1. **Restore budget**: `restore p95 <= 400ms` on standard workload.
2. **Compaction budget**: `compaction p95 <= 1.5x pre-branch baseline`.

## Harness design

### Workload profiles
- `standard_restore_profile`: representative branch-switch restore payload.
- `compaction_branch_profile`: long-session branch artifact set.

### Minimum samples
- restore runs: `>=200`
- compaction runs: `>=200`

### Measurements
Per run emit:
- `operation` (`restore` or `compaction`)
- `latency_ms`
- `profile_id`
- `workload_hash`
- `run_id`

## Budget calculations

1. Restore gate:
- compute `restore_p95_ms` (nearest-rank p95).
- pass when `restore_p95_ms <= 400`.

2. Compaction gate:
- compute `compaction_p95_ms` for branch workload.
- compute `pre_branch_baseline_p95_ms`.
- compute `compaction_ratio = compaction_p95_ms / pre_branch_baseline_p95_ms`.
- pass when `compaction_ratio <= 1.5`.

## Output contract

Result envelope fields:
- `gate_id` (`D3.2-restore-compaction`)
- `restore_p95_ms`
- `restore_threshold_ms` (`400`)
- `restore_decision` (`pass|fail|insufficient_sample`)
- `compaction_p95_ms`
- `pre_branch_baseline_p95_ms`
- `compaction_ratio`
- `compaction_threshold_ratio` (`1.5`)
- `compaction_decision` (`pass|fail|insufficient_sample`)
- `final_decision` (`pass|fail`)

## Determinism and validity constraints

1. Percentile method fixed to nearest-rank p95.
2. Restore and compaction profiles must remain workload-stable per `workload_hash`.
3. Any lane with sample count `<200` yields `insufficient_sample` and blocks final pass.
4. Final decision is pass only if both restore and compaction gates pass.

## Gate linkage

- Satisfies Spec80 §20.2 restore/compaction performance requirements for Epic D closure.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_REFLECTION_METACOG_LATENCY_BUDGET_2026-04-21.md
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md

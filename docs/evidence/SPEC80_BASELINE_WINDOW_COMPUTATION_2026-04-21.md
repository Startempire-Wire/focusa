# SPEC80 E2.1 — Baseline Window Computation Design

Date: 2026-04-21
Bead: `focusa-yro7.5.2.1`
Purpose: define deterministic computation of prior-14-day baseline medians and evaluation-window comparisons for Gate D scoring.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (Appendix C §16, Gate D)
- docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md

## Window definitions

1. **Enablement timestamp** (`t_enable`)
   - contract-scoring feature activation boundary.
2. **Baseline window**
   - `[t_enable - 14d, t_enable)`
   - baseline statistic is median metric value over this window.
3. **Evaluation window**
   - rolling 14-day window after enablement: `(t_now - 14d, t_now]`.

## Computation protocol

For each metric id:
1. Collect normalized metric samples in baseline window.
2. Require Appendix C sample floors before baseline acceptance:
   - `>=200` total turns,
   - `>=30` novel-context turns,
   - `>=20` loops with failures (setback lane).
3. Compute `baseline_median` using deterministic median method:
   - sort ascending,
   - odd N: middle element,
   - even N: arithmetic mean of two middle elements.
4. Persist baseline record keyed by:
   - `metric_id`, `enablement_id`, `window_start`, `window_end`, `baseline_median`, `sample_counts`.

## Evaluation join protocol

For each evaluation run:
1. Compute metric values over rolling evaluation window.
2. Join to persisted baseline by `metric_id + enablement_id`.
3. Compute relative delta:
   - `relative_delta = (eval_value - baseline_median) / baseline_median`.
4. Mark `insufficient_sample` when sample floors are unmet; do not emit numeric deltas.

## Typed statuses

Allowed baseline/evaluation statuses:
- `ok`
- `insufficient_sample`
- `baseline_missing`
- `baseline_zero_guard` (division-safe guard when baseline median is zero)

## Determinism constraints

1. Same input sample set and `t_enable` must produce identical baseline boundaries.
2. Median method is fixed; no percentile interpolation variant allowed.
3. Boundary inclusivity is fixed as specified above; no timezone-dependent drift.

## Gate linkage

- Implements Appendix C baseline/evaluation semantics required before Gate D threshold evaluation.
- Supplies stable baseline joins for E2.2 evaluation cadence automation.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md

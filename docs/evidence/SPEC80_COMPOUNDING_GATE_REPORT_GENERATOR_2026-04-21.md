# SPEC80 E4.1 — Compounding Gate Report Generator Contract

Date: 2026-04-21
Bead: `focusa-yro7.5.4.1`
Purpose: define the report schema and generation rules for Gate D evidence output (contract deltas, sample checks, regression guard, final decision).

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8 outcome contracts, §10 Gate D, §16 Appendix C, §18.5 minimum volume)
- docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md
- docs/evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md

## Required Gate D inputs

1. Per-contract threshold outcomes for six contracts.
2. Metric deltas vs baseline (rolling 14-day evaluation).
3. Appendix C sample-floor status (`>=200` turns, `>=30` novel-context turns, `>=20` loops with failures).
4. Critical regression guard result (`failed_turn_ratio` worsens by `>5%` => force fail).
5. Appendix E minimum form volume gate (`>=50` valid forms; `>=20` novel-context forms).

## Report generation algorithm

1. Load latest 14-day evaluation snapshot and threshold-evaluator outputs.
2. Materialize contract rows with `{contract_id, metric_ids, relative_delta, threshold, state}`.
3. Compute `pass_count` over contracts where `state=pass`.
4. Evaluate critical regression guard.
5. Evaluate Appendix C sample floors and Appendix E form-volume floors.
6. Emit final Gate D decision:
   - `pass` only if:
     - `pass_count >= 4`,
     - critical regression guard is false,
     - sample floors satisfied,
     - form-volume floors satisfied.

## Output contract

Top-level envelope:
- `report_id`
- `generated_at`
- `evaluation_window`
- `baseline_window`
- `contracts[]`
- `pass_count`
- `critical_regression`
- `sample_floor_status`
- `form_volume_status`
- `final_decision` (`pass|fail|insufficient_data`)
- `evidence_refs[]`

Per-contract row:
- `contract_id`
- `metrics[]`
- `relative_delta`
- `threshold_rule`
- `state` (`pass|fail|insufficient_sample`)
- `notes`

## Determinism + audit rules

1. Same evaluation inputs must yield identical `report_id` and payload hash.
2. All fail outcomes must include machine-readable reasons (`critical_regression`, `sample_floor`, `form_volume`, or `threshold_fail`).
3. `insufficient_data` is mandatory when sample or form-volume floors are unmet; no silent downgrade to pass/fail.

## Gate linkage

- Implements E4.1 reporting requirement for Epic E closure.
- Produces canonical evidence packet for Gate D decision review.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md
- docs/evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md

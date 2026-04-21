# SPEC80 E1.2 — Outcome Threshold Evaluator Design

Date: 2026-04-21
Bead: `focusa-yro7.5.1.2`
Purpose: define threshold evaluation logic for six outcome contracts and Gate D pass/fail decisioning.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §10 Gate D, Appendix C regression rule)
- docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md

## Contract thresholds

1. Self-regulation
- `strategy_adjusted_turn_rate`: `+20%` relative improvement.

2. Outcome quality
- `failed_turn_ratio`: `>=15%` improvement (downward risk metric).
- `rework_loop_rate`: `>=15%` improvement (downward risk metric).

3. Transfer
- `novel_context_strategy_reuse_rate`: `+15%` relative improvement.

4. Motivation/ownership
- `setback_recovery_rate`: `+15%` relative improvement.

5. Social/perspective quality
- `perspective_constraint_density`: `+10%` relative improvement.

6. Instructor/operator regulation
- `steering_uptake_rate`: `>=20%` improvement.
- `forced_pause_rate_after_steering`: must not regress.

## Evaluation algorithm

Input:
- metric rows from extraction pipeline (`value`, `baseline_value`, `relative_delta`, `sample_size_ok`)

Per-contract evaluation:
1. Validate sample floor (`sample_size_ok == true`), else mark `insufficient_sample`.
2. Apply threshold comparator for each contract metric.
3. Mark contract state as `pass|fail|insufficient_sample`.

Gate D decision:
1. Count `pass` contracts among six outcome contracts.
2. Require `pass_count >= 4`.
3. Enforce critical regression guard:
   - if `failed_turn_ratio` worsens by `>5%`, Gate D fails regardless of pass_count.

Output envelope:
- `gate_id` (`Gate D`)
- `contract_results[]` (contract_id, state, delta, threshold, notes)
- `pass_count`
- `critical_regression` (bool)
- `final_decision` (`pass|fail`)

## Determinism and audit constraints

1. Same metric input set must produce identical gate decision.
2. All threshold comparisons must be explicitly recorded in `contract_results`.
3. `insufficient_sample` contracts cannot be counted as pass.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md

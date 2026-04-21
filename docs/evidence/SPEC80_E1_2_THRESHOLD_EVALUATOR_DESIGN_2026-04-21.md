# SPEC80 E1.2 — Threshold Evaluator Design

Date: 2026-04-21
Bead: `focusa-yro7.5.1.2`
Label: `documented-authority`

Purpose: define evaluator logic for SPEC80 outcome thresholds and Gate D pass/fail.

## Inputs

1. Current evaluation-window metric scorecard (from E1.1).
2. Baseline scorecard (prior 14-day median).
3. Sample-size gates (from Appendix C).

## Threshold rules (from §8)

- Self-regulation: `strategy_adjusted_turn_rate` >= +20% relative.
- Outcome quality: both `failed_turn_ratio` and `rework_loop_rate` improve >=15%.
- Transfer: `novel_context_strategy_reuse_rate` >= +15%.
- Motivation/ownership: `setback_recovery_rate` >= +15%.
- Social/perspective: `perspective_constraint_density` >= +10%.
- Instructor/operator regulation: `steering_uptake_rate` >= +20% and `forced_pause_rate_after_steering` no regression.

## Gate D decision logic

1. Compute contract pass/fail for six contracts.
2. Count passed contracts.
3. Apply regression override: if failed-turn ratio worsens by >5%, Gate D fails.
4. Gate D pass when >=4/6 contracts pass and no critical regression override triggered.

## Output schema

```json
{
  "contracts": {
    "self_regulation": {"pass": true, "delta": 0.0},
    "outcome_quality": {"pass": true, "deltas": {"failed_turn_ratio": 0.0, "rework_loop_rate": 0.0}},
    "transfer": {"pass": true, "delta": 0.0},
    "motivation_ownership": {"pass": true, "delta": 0.0},
    "social_perspective": {"pass": true, "delta": 0.0},
    "instructor_regulation": {"pass": true, "deltas": {"steering_uptake_rate": 0.0, "forced_pause_rate_after_steering": 0.0}}
  },
  "passed_contracts": 0,
  "critical_regression": false,
  "gate_d_pass": false,
  "notes": []
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §10 Gate D, §16)
- docs/evidence/SPEC80_E1_1_METRIC_EXTRACTION_PIPELINE_DESIGN_2026-04-21.md

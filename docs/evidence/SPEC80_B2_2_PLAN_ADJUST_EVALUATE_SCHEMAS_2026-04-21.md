# SPEC80 B2.2 — Plan-Adjust/Evaluate Schema Finalization

Date: 2026-04-21
Bead: `focusa-yro7.2.2.2`
Label: `planned-extension`

Purpose: finalize contract schemas and promotion logic for:
- `focusa_metacog_plan_adjust`
- `focusa_metacog_evaluate_outcome`

## 1) `focusa_metacog_plan_adjust`

### Input
```json
{
  "reflection_id": "string",
  "selected_updates": ["string"],
  "constraints_added": ["string"],
  "expected_deltas": {
    "failed_turn_ratio": -0.15,
    "rework_loop_rate": -0.15,
    "steering_uptake_rate": 0.20
  },
  "effective_window": {"turns": 20}
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "adjustment_id": "string",
    "next_step_policy": ["string"],
    "constraints_bound": ["string"],
    "expected_deltas": {},
    "applies_from_turn": "string"
  }
}
```

### Error codes
- `ADJUST_POLICY_CONFLICT`
- `ADJUST_INPUT_INVALID`
- `REFLECTION_NOT_FOUND`

## 2) `focusa_metacog_evaluate_outcome`

### Input
```json
{
  "adjustment_id": "string",
  "observed_metrics": {
    "failed_turn_ratio": 0.0,
    "rework_loop_rate": 0.0,
    "setback_recovery_rate": 0.0,
    "steering_uptake_rate": 0.0
  },
  "baseline_ref": "rolling_14_day_median",
  "sample_size": 30
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "evaluation_id": "string",
    "delta_scorecard": {
      "failed_turn_ratio": {"expected": -0.15, "observed": -0.11, "met": false},
      "rework_loop_rate": {"expected": -0.15, "observed": -0.18, "met": true},
      "steering_uptake_rate": {"expected": 0.20, "observed": 0.24, "met": true}
    },
    "result": "improved|unchanged|regressed|inconclusive",
    "promote_learning": false,
    "promotion_reason": "string"
  }
}
```

### Error codes
- `EVAL_INPUT_INVALID`
- `EVAL_BASELINE_UNAVAILABLE`
- `ADJUSTMENT_NOT_FOUND`

## 3) Promotion logic

`promote_learning=true` only when all are true:
1. sample_size meets minimum gate for contract family,
2. no critical regression in outcome-quality contracts,
3. at least one targeted expected delta is met,
4. evidence refs present for evaluation packet.

## 4) Binding + permissions

- `focusa_metacog_plan_adjust`: `POST /v1/metacognition/adjust` (planned), permission `metacognition:write`
- `focusa_metacog_evaluate_outcome`: `POST /v1/metacognition/evaluate` (planned), permission `metacognition:write`
- CLI fallbacks (planned):
  - `focusa metacognition adjust --json`
  - `focusa metacognition evaluate --json`

## 5) Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §14.2, §15, §16)
- docs/24-capabilities-cli.md

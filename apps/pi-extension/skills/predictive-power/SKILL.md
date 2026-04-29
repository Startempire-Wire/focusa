# Predictive Power

Use this skill when recording, evaluating, or interpreting Focusa predictions.

## Rules

1. Predictions guide; they never override operator steering.
2. Always include evidence refs or route/tool handles when possible.
3. Record before acting when there is meaningful uncertainty.
4. Evaluate after outcome is known.
5. Use `focusa_predict_stats` or `focusa predict stats` to inspect calibration.

## Tool flow

1. `focusa_predict_record`
2. act normally with explicit operator/current-project priority
3. `focusa_predict_evaluate`
4. `focusa_predict_stats`

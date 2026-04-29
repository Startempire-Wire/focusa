# Predictive Power Tools

Focusa prediction tools create bounded, inspectable prediction records. Predictions guide decisions; they do not override operator steering and they do not silently mutate task state.

## Tools

- [`focusa_predict_record`](tools/focusa_predict_record.md) — record a prediction with confidence, recommended action, reason, and optional context refs.
- [`focusa_predict_recent`](tools/focusa_predict_recent.md) — list recent prediction records.
- [`focusa_predict_evaluate`](tools/focusa_predict_evaluate.md) — evaluate a prediction against an observed outcome.
- [`focusa_predict_stats`](tools/focusa_predict_stats.md) — report aggregate accuracy/calibration stats.

## Use when

- choosing the next safest action;
- estimating drift/context/token/cache/release risk;
- validating whether a prior strategy improved outcomes;
- avoiding repeated intuition without measurement.

## Safety rules

- Keep predictions bounded and inspectable.
- Link context refs/evidence handles instead of transcript blobs.
- Evaluate predictions after the outcome is known.
- Treat predictions as advisory; operator steering wins.

See also: [`../current/PREDICTIVE_POWER_GUIDE.md`](../current/PREDICTIVE_POWER_GUIDE.md).

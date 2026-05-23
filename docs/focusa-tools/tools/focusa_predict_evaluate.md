# focusa_predict_evaluate

First-class Spec92 prediction tool.

## Purpose

Use this tool to work with bounded, inspectable Focusa prediction records. Predictions guide agent behavior and never override operator steering.

## API / CLI parity

See [Predictive Power Guide](../../current/PREDICTIVE_POWER_GUIDE.md).

## Expected result

The tool should return a visible summary plus structured details. Inspect `details.tool_result_v1` for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`. Predictions are advisory signals only; they never choose work or override operator steering.

## Safety

- No raw provider payloads.
- Use evidence refs/handles in context refs.
- Evaluate predictions after actual outcomes are known.

## Contract summary

- Family: Metacognition.
- Side effects: `write_prediction_evaluation`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/predictions/{prediction_id}/evaluate`
- CLI commands: `focusa predict evaluate`
- Parity: `full`.
- Core surface: Spec92 prediction store and telemetry.
- Live check: contract_static plus focusa_predict_stats and /v1/predictions/stats.
- Contract source: `docs/current/focusa-tool-contracts.json`.

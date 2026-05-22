# focusa_predict_record

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

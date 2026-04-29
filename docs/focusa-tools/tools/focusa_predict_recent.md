# focusa_predict_recent

First-class Spec92 prediction tool.

## Purpose

Use this tool to work with bounded, inspectable Focusa prediction records. Predictions guide agent behavior and never override operator steering.

## API / CLI parity

See [Predictive Power Guide](../../current/PREDICTIVE_POWER_GUIDE.md).

## Safety

- No raw provider payloads.
- Use evidence refs/handles in context refs.
- Evaluate predictions after actual outcomes are known.

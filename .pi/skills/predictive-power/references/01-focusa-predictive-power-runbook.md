# Focusa Predictive Power Runbook

## Workflow

1. Query `focusa_predict_recent` and `focusa_predict_stats` in the exact project/workstream scope.
2. Record a bounded decision-point prediction with outcome, confidence, action, why, and evidence refs.
3. After real evidence arrives, evaluate the same prediction ID and score the observed outcome.
4. Use calibration statistics to improve future choices; predictions never override operator steering.

## Tool routing

Search `prediction` with `focusa_tool_search`, then load exact schemas with `focusa_tool_describe`. All prediction Pi tools are listed in `pi-tools.json` and documented individually under `docs/focusa-tools/tools/`.

## Recovery

- HTTP/validation rejection: use `focusa_tool_doctor`; do not retry unchanged against a drifted live contract.
- No evidence: leave prediction unevaluated.
- Scope ambiguity: verify project identity and continuity before record/evaluation.

## Done condition

The prediction is evaluated against observed evidence and contributes scoped calibration rather than an unverified narrative.

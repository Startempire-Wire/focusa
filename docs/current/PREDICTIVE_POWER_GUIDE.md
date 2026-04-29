# Predictive Power Guide

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

Focusa predictions are bounded, inspectable records. They guide decisions; they do not mutate state silently and do not override operator steering.

## API

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/predictions \
  -H 'Content-Type: application/json' \
  -d '{"prediction_type":"token_risk","predicted_outcome":"watch","confidence":0.7,"recommended_action":"run token doctor","why":"recent context high","context_refs":["/v1/telemetry/token-budget/status"]}' | jq .

curl -sS http://127.0.0.1:8787/v1/predictions/recent | jq .
curl -sS http://127.0.0.1:8787/v1/predictions/stats | jq .
```

## CLI

```bash
focusa predict record \
  --prediction-type token_risk \
  --predicted-outcome watch \
  --confidence 0.7 \
  --recommended-action "run focusa tokens doctor" \
  --why "recent token records show bloat" \
  --context-refs /v1/telemetry/token-budget/status

focusa predict recent --limit 20
focusa predict evaluate <prediction_id> --actual-outcome "watch" --score 1.0
focusa predict stats
```

## Prediction types

- `next_action_success`
- `tool_choice`
- `release_failure`
- `stale_state`
- `context_relevance`
- `token_risk`
- `cache_hit`
- `drift_risk`
- `workpoint_resume_success`
- `compaction_recovery`

## Stats

`focusa predict stats` reports total predictions, evaluated predictions, global accuracy, and accuracy by prediction type.

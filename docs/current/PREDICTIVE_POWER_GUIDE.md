# Predictive Power Guide

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

Focusa predictions are bounded, inspectable records. They guide decisions; they do not override operator steering. Current predictions carry bounded `trajectory` and `ontology_context`; evaluated outcomes feed metacognition capture memory, and promoted metacognition evaluations record follow-up predictions so prediction ↔ metacognition compounds as a flywheel.

## API

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/predictions \
  -H 'Content-Type: application/json' \
  -d '{"prediction_type":"token_risk","predicted_outcome":"watch","confidence":0.7,"recommended_action":"run token doctor","why":"recent context high","context_refs":["/v1/telemetry/token-budget/status"]}' | jq .

curl -sS -X POST http://127.0.0.1:8787/v1/predictions/capture-outcome \
  -H 'Content-Type: application/json' \
  -d '{"prediction_type":"token_risk","actual_outcome":"watch","score":1.0,"ontology_context":{"object_refs":["TokenBudget"],"tool_refs":["focusa_resource_mode"],"evidence_refs":["/v1/telemetry/token-budget/status"]}}' | jq .

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
focusa predict capture-outcome --prediction-type token_risk --actual-outcome "watch" --score 1.0 --ontology-context '{"object_refs":["TokenBudget"],"tool_refs":["focusa_resource_mode"]}'
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

`focusa predict stats` reports total predictions, evaluated predictions, global accuracy, accuracy by prediction type, and trajectory grouping when active trajectory context exists.

## Flywheel contract

1. Before uncertain/risky/model-choice work: record `focusa_predict_record` with `ontology_context` (`object_refs`, `action_refs`, `tool_refs`, `evidence_refs`, `relation_refs`).
2. After proof/test/CI/evidence: run `focusa_predict_evaluate` or `focusa predict capture-outcome`.
3. Successful prediction outcomes become metacognition captures with `strategy_class=prediction_metacog_flywheel`.
4. Successful metacognition evaluations create follow-up predictions (`prediction_type=metacog_learning_transfer`).
5. Next agent turn should retrieve metacognition before acting, then record the next prediction.

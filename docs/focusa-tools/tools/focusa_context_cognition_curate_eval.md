# `focusa_context_cognition_curate_eval`

**Family:** `trajectory`
**Label:** Context Cognition Curate Eval

## Purpose

**Spec 100 Phase 4 — Eval harness** with CQRS write side. Run a curator eval case: take a list of candidates, run the deterministic curator, and compute precision / recall / F1 versus an operator-supplied `expected_selected_paths`. Append the result as a `CuratorEvalRun` to `data/curator-eval-ledger/{project_root_hash}/eval-runs.jsonl` (append-only, scope-bounded, replay-friendly).

The eval result is the input to the **Phase 5 Cognition Optimizer**: the operator then submits the eval result + a candidate artifact to `focusa_context_cognition_curate_optimize`, which decides `promote | rollback` per Spec 100 §15 promotion rule.

## When to use

- The operator has a curator eval case (target, candidates, expected_selected_paths, score_threshold).
- The operator wants to measure whether the v0 deterministic curator's selection matches expectations.
- The operator wants a durable, append-only record of the eval for the promotion gate.

Do not use for production context selection; use `focusa_context_cognition_curate` instead.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — **required** workstream scope for eval-ledger writes; missing/blank rejects with `failure_class=continuity_id_missing`.
- `case_id` — optional case id; defaults to a generated UUID v7.
- `target` — curator target string. Defaults to the active workpoint's `next_slice` or `mission`.
- `token_budget` — total tokens allowed. Default 2000, max 1,000,000; `0` rejects with `failure_class=token_budget_invalid`.
- `candidates` — list of `{kind, path, body?, evidence_ref?, tokens?}` items.
- `expected_selected_paths` — list of paths the operator expects the curator to keep.
- `score_threshold` — F1 threshold for promotion. Default 0.5; must be finite `0.0..=1.0`.
- `baseline_f1` — baseline F1 to beat. Default 0.0; must be finite `0.0..=1.0`.
- `evidence_refs` — list of evidence refs that boost candidate ranking.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, plus the run fields:

- `run_id` (UUID v7)
- `case_id`
- `selected_paths` (curator's selection)
- `expected_paths` (operator-supplied)
- `precision`, `recall`, `f1` (computed)
- `baseline_f1`, `score_threshold`
- `tokens_used`, `token_budget`
- `promoted` (bool: F1 > baseline_f1 AND F1 >= score_threshold)
- `eval_ref` (ledger handle: `curator-eval:{project_root}:{run_id}`)
- `rehydrate_id` (= `run_id`)

The `eval_ref` is an evidence-citation handle. The result is also returned in `details.evidence_refs` for direct linking via `focusa_evidence_capture`.

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "case_id": "case-001",
  "target": "focusa_context_cognition_curate",
  "token_budget": 200,
  "candidates": [
    {"kind": "file", "path": "crates/focusa-api/src/routes/context_cognition.rs", "body": "curate handler"},
    {"kind": "file", "path": "crates/focusa-cli/src/commands/context_cognition.rs", "body": "unrelated"}
  ],
  "expected_selected_paths": ["crates/focusa-api/src/routes/context_cognition.rs"],
  "score_threshold": 0.5,
  "baseline_f1": 0.0,
  "evidence_refs": []
}
```

```text
focusa_context_cognition_curate_eval ok | context cognition curate eval → f1=0.66 promoted=yes
ids: run_id=019ea... eval_ref=curator-eval:/home/wirebot/focusa:019ea... rehydrate_id=019ea...
fields: f1=0.667 precision=1.000 recall=0.500 baseline_f1=0.000 tokens_used=2 promoted=yes advisory=true
next: focusa_context_cognition_curate_optimize → focusa_metacog_capture → focusa_predict_record
```

## Scope rules

- `project_root` is **required** — eval is scoped to project.
- `continuity_id` is **required** — eval writes are scoped by `project_root + continuity_id`.
- Agent runtime paths (e.g. `/root/pi-mono`, `/home/wirebot/.cargo`) are rejected with `failure_class=scope_mismatch`.
- The eval is **deterministic** for the same input (curator is deterministic; precision/recall/F1 are computed in-route).
- The ledger is **append-only** — existing eval runs are never modified or deleted.

## Notes

- Per Spec 100 §15.1 the curator-eval-ledger is the **CQRS write side** for eval runs. The read side is `GET /v1/context-cognition/curate/eval/runs` (CLI: `focusa context-cognition curate-eval-runs`).
- The eval harness pairs with `focusa_context_cognition_curate_optimize` (Phase 5) for the promotion gate.
- The eval result is suitable for emitting as a `focusa_metacog_capture` lesson (`kind=curator_eval_v0`) and a `focusa_predict_record` prediction (`prediction_type=curator_optimization_v1`).

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `continuity_id_missing` — provide the active continuity id from Workpoint/Trajectory scope.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `token_budget_invalid` — provide a token budget greater than zero.
- `score_out_of_range` — keep `score_threshold` and `baseline_f1` finite and within `0.0..=1.0`.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.
- `storage_unwritable` — inspect daemon logs; the eval is not persisted and the route returns 500.

When `failure_class` is missing, treat the response as a successful eval; verify with `GET /v1/context-cognition/curate/eval/runs`.

## Contract summary

- Family: `trajectory`
- Side effects: `write_curator_eval` (append-only ledger)
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/context-cognition/curate/eval`, `GET /v1/context-cognition/curate/eval/runs`
- CLI commands: `focusa context-cognition curate-eval`, `focusa context-cognition curate-eval-runs`
- Core surface: `Spec100 §15.1 CQRS write side (eval-ledger append-only JSONL)`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition_curate_optimize` — submit the eval result + a candidate artifact for the promotion decision.
- `focusa_metacog_capture` — capture the eval as a lesson (kind=curator_eval_v0).
- `focusa_predict_record` — record a prediction (prediction_type=curator_optimization_v1) for the eval outcome.
- `focusa_evidence_capture` — link the eval run as evidence to the active Workpoint.

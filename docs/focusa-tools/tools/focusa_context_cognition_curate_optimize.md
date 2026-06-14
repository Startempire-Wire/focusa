# `focusa_context_cognition_curate_optimize`

**Family:** `trajectory`
**Label:** Context Cognition Curate Optimize

## Purpose

**Spec 100 Phase 5 — Cognition Optimizer** with CQRS write side. Submit a candidate prompt/module artifact and get the `promote | rollback` decision per Spec 100 §15 promotion rule:

- `promoted=true` when `eval_score > baseline_score` AND `eval_score >= score_threshold`
- `explicit_rollback=true` overrides to `promoted=false`
- otherwise `rollback`

Appends a `CognitionOptimizerArtifact` to `data/cognition-optimizer-artifacts/{project_root_hash}/artifacts.jsonl` (append-only, scope-bounded, replay-friendly). The latest promoted artifact becomes the active policy; rollback is a new entry with `promoted=false` and `rollback_ref` pointing at the previous promoted artifact id.

The `optimization_frame` in the `ContextCognitionPacket` is populated from the latest promoted artifact for the project (Spec 100 Phase 6 integration).

## When to use

- The operator has a `focusa_context_cognition_curate_eval` result (`eval_score`, `baseline_score`, `eval_run_id`).
- The operator has a candidate prompt/module artifact (`prompt_artifact_ref`).
- The operator wants the curator's promotion decision (or explicit rollback).

Do not use for trivial submissions; the optimize call is for the promotion gate.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — **required** workstream scope for optimizer-artifact writes; missing/blank rejects with `failure_class=continuity_id_missing`.
- `module_name` — module name. Default `curator`.
- `prompt_artifact_ref` — path or ref id of the candidate artifact (≤ 4096 chars).
- `eval_score` — candidate artifact's eval F1 score; must be finite `0.0..=1.0`.
- `baseline_score` — baseline F1 to beat. Default 0.0; must be finite `0.0..=1.0`.
- `score_threshold` — F1 threshold for promotion. Default 0.5; must be finite `0.0..=1.0`.
- `eval_run_id` — optional CuratorEvalRun id that produced `eval_score`.
- `rollback` — explicit rollback override. Default `false`.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, plus:

- `artifact_id` (UUID v7)
- `decision` (`promote | rollback`)
- `promoted` (bool)
- `eval_score`, `baseline_score`, `score_threshold`
- `rollback_ref` (id of the previous promoted artifact, or `none`)
- `eval_run_id` (if supplied)
- `rehydrate_id` (= `artifact_id`)

The artifact is persisted in the optimizer-artifacts ledger. The next call to `GET /v1/context-cognition/optimizer/artifacts` returns it in the list.

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "module_name": "curator",
  "prompt_artifact_ref": "prompts/curator-v1.md",
  "eval_score": 0.85,
  "baseline_score": 0.50,
  "score_threshold": 0.6,
  "eval_run_id": "019ea...",
  "rollback": false
}
```

```text
focusa_context_cognition_curate_optimize ok | context cognition curate optimize → decision=promote promoted=yes
ids: artifact_id=019ea... rehydrate_id=019ea... rollback_ref=019e9...
fields: decision=promote eval_score=0.850 baseline_score=0.500 promoted=yes advisory=true
next: focusa_context_cognition_optimizer_artifacts → focusa_predict_record → focusa_metacog_capture
```

## Scope rules

- `project_root` is **required** — optimize is scoped to project.
- `continuity_id` is **required** — optimizer writes are scoped by `project_root + continuity_id`.
- `prompt_artifact_ref` is **required** — empty/missing is rejected with `failure_class=prompt_artifact_ref_missing`.
- `eval_score` is **required** — missing is rejected with `failure_class=eval_score_missing`.
- Agent runtime paths are rejected with `failure_class=scope_mismatch`.
- The artifact ledger is **append-only** — promotion is a new entry with `promoted=true`; rollback is a new entry with `promoted=false`.

## Notes

- Per Spec 100 §15.1 the cognition-optimizer-artifacts ledger is the **CQRS write side** for the promotion gate. The read side is `GET /v1/context-cognition/optimizer/artifacts` (CLI: `focusa context-cognition optimizer artifacts`).
- The optimize call is **deterministic** for the same input (decision rule is a pure function of inputs + the latest promoted artifact).
- The runtime consumption of the promoted artifact happens on the next `focusa_context_cognition_curate` call. The artifact is read fresh from the ledger, not cached.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `prompt_artifact_ref_missing` — supply `prompt_artifact_ref`.
- `eval_score_missing` — supply `eval_score`.
- `continuity_id_missing` — provide the active continuity id from Workpoint/Trajectory scope.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path.
- `score_out_of_range` — keep `eval_score`, `baseline_score`, and `score_threshold` finite and within `0.0..=1.0`.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.
- `storage_unwritable` — inspect daemon logs; the artifact is not persisted and the route returns 500.

When `failure_class` is missing, treat the response as a successful optimize; verify with `GET /v1/context-cognition/optimizer/artifacts`.

## Contract summary

- Family: `trajectory`
- Side effects: `write_cognition_optimizer_artifact` (append-only ledger)
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/context-cognition/curate/optimize`
- CLI commands: `focusa context-cognition curate-optimize`
- Core surface: `Spec100 §15.1 CQRS write side (artifact ledger append + promotion gate)`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition_optimizer_artifacts` — list the artifact ledger (verify the new entry).
- `focusa_predict_record` — record a prediction (prediction_type=curator_optimization_v1) for the promotion outcome.
- `focusa_metacog_capture` — capture the promotion/rollback as a lesson (kind=curator_optimization_v0).
- `focusa_evidence_capture` — link the artifact as evidence to the active Workpoint.

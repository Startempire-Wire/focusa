# `focusa_context_cognition_optimizer_artifacts`

**Family:** `trajectory`
**Label:** Context Cognition Optimizer Artifacts

## Purpose

**Spec 100 Phase 5 — Cognition Optimizer** with CQRS read side. List the versioned `CognitionOptimizerArtifact` JSONL ledger for a project+module. Returns the recent artifact list and the latest promoted artifact (if any).

This is the read companion to `focusa_context_cognition_curate_optimize` (the write side). The artifact ledger is append-only, scope-bounded by `(project_root, module_name)`, and replay-friendly.

## When to use

- The operator wants to see the artifact history for a project+module.
- The operator wants to know which artifact is currently promoted (active policy).
- The operator wants the `rollback_ref` chain to understand the promotion/rollback sequence.

Do not use for trivial artifact lookups; this is a structured read.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `module_name` — module name. Default `curator`.
- `limit` — max artifacts to return. Default 10, max 200.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, plus:

- `count` — number of artifacts returned.
- `artifacts` — list of `{artifact_id, module_name, prompt_artifact_ref, eval_score, baseline_score, promoted, rollback_ref, eval_run_id, created_at, promovido_at}`.
- `latest_promoted` — the latest artifact with `promoted=true`, or `null` if none.
- `rehydrate_id` — the id of the most recent artifact (regardless of promotion).

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "module_name": "curator",
  "limit": 10
}
```

```text
focusa_context_cognition_optimizer_artifacts ok | optimizer artifacts → count=3 module=curator
ids: rehydrate_id=019ea... latest_promoted_id=019ea...
fields: count=3 module_name=curator latest_promoted=019ea...@0.85 advisory=true
next: focusa_context_cognition_curate_optimize
```

## Scope rules

- `project_root` is **required** — read is scoped to project.
- Agent runtime paths are rejected with `failure_class=scope_mismatch`.
- The read is **read-only** — no Workpoint, Trajectory, or HLT mutation.

## Notes

- Per Spec 100 §15.1 the cognition-optimizer-artifacts ledger is the **CQRS write side** for the promotion gate; this tool is the read side.
- The read is **deterministic** for the same ledger state.
- The latest promoted artifact is the active policy; the runtime consumption happens on the next `focusa_context_cognition_curate` call.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful read; verify the returned `count` against the operator's expectation.

## Contract summary

- Family: `trajectory`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/context-cognition/optimizer/artifacts`
- CLI commands: `focusa context-cognition optimizer artifacts`
- Core surface: `Spec100 §15.1 CQRS read side (artifact ledger)`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition_curate_optimize` — submit a new artifact and get the promotion decision.
- `focusa_context_cognition_curate_eval` — run a curator eval case (the input to the promotion gate).
- `focusa_predict_record` — record a prediction (prediction_type=curator_optimization_v1) for the latest promotion.
- `focusa_metacog_capture` — capture the latest promotion as a lesson.

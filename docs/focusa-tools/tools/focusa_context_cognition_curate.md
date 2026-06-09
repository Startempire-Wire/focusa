# `focusa_context_cognition_curate`

**Family:** `trajectory`
**Label:** Context Cognition Curate

## Purpose

**Spec 100 Phase 3 — Context Curator** with token-budgeted context selection. Takes a list of candidates (files, docs, diffs, snippets, codemaps, evidence) and selects the highest-scoring subset under a token budget. Returns `selected_context` (the kept items) and `excluded_context` (the dropped items with reasons: `low_score` or `over_budget`).

The curator preserves `project_root + continuity_id` scope, prefers bounded handles, and avoids transcript-tail authority. v0 ranks candidates by:

- workpoint target keyword overlap (workpoint `next_slice` or `mission`, or operator-supplied `target`),
- evidence_ref overlap with operator-supplied `evidence_refs`,
- tie-breaker on token count (denser items preferred).

v0.5 will replace the word-count tokenizer with a real `tiktoken`-backed estimator and add ontology/workpoint-active-object relevance.

## When to use

- The agent or operator has a candidate list (≥ 1 file, doc, diff, evidence ref) and a token budget.
- The next prompt/CLI/menubar section needs a curated subset, not a raw dump.
- Workpoint has a clear `next_slice` or `mission`; the curator uses it as the default target.

Do not use for trivial single-file selection; just read the file.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — optional workstream filter.
- `target` — curator target string. Defaults to the active workpoint's `next_slice`, then `mission`, then empty.
- `token_budget` — total tokens allowed. Default 2000, max 1,000,000.
- `candidates` — list of `{kind, path, body?, evidence_ref?, tokens?}` items. `kind` is `file | doc | diff | snippet | codemap | evidence`. `tokens` overrides the body-derived estimate.
- `evidence_refs` — list of evidence refs that boost candidates matching them.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, `target`, `token_budget`, `tokens_used`, `tokens_remaining`, `selected_context` (array of `{kind, path, body?, tokens, score}`), `excluded_context` (array of `{kind, path, reason}`), `selected_count`, `excluded_count`, `evidence_refs`, and `rehydrate_id`.

The output is bounded: at most the operator's candidate count, plus bounded per-item fields (≤ ~1KB JSON each). v0 does not mutate Workpoint or Trajectory.

## Example

```json
{
  "project_root": "/home/wirebot/focusa",
  "target": "focusa_context_cognition curate",
  "token_budget": 200,
  "candidates": [
    {"kind": "file", "path": "crates/focusa-api/src/routes/context_cognition.rs", "body": "focusa_context_cognition_curate handler", "evidence_ref": "ev:1"},
    {"kind": "file", "path": "crates/focusa-cli/src/commands/context_cognition.rs", "body": "unrelated CLI commands"}
  ],
  "evidence_refs": ["ev:1", "ev:2"]
}
```

```text
focusa_context_cognition_curate ok | context cognition curate → selected=2 excluded=0
ids: rehydrate_id=ctx_curate:/home/wirebot/focusa:2 target=focusa_context_cognition curate
fields: selected_count=2 excluded_count=0 tokens_used=9 token_budget=200 tokens_remaining=191 advisory=true
next: focusa_context_cognition → focusa_context_cognition_render → focusa_evidence_capture
```

## Scope rules

- `project_root` is **required** — curator is scoped to project.
- Agent runtime paths (e.g. `/root/pi-mono`) are rejected with `failure_class=scope_mismatch`.
- The curator never reads files from disk in v0; the operator must supply `body` (or accept the empty fallback). v0.5 will add bounded file reads.
- `token_budget` must be a positive integer (1–1,000,000).

## Notes

- The curator's score is **deterministic** for the same input (workpoint target + body + evidence_refs). No randomness, no model calls.
- `excluded_context` reasons are bounded strings: `low_score: N.NN < 2.0` or `over_budget: N > remaining M`. Operators can audit exclusions without model calls.
- v0 implements: scoring, exclusion, budget cut, evidence overlap boost. v0.5 will add ontology/workpoint active-object relevance + tiktoken-backed token estimation.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `project_root_unverified` — call `focusa_project_verify` first.
- `scope_mismatch` — the `project_root` is an agent runtime path; pick a real project folder.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful curation; verify with `focusa_context_cognition`.

## Contract summary

- Family: `trajectory`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/context-cognition/curate`
- CLI commands: `focusa context-cognition curate`
- Core surface: `Spec100 §14 Context Curator with token-budgeted selection`
- Spec: `docs/100-context-cognition-spec.md`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_context_cognition` — the full packet JSON.
- `focusa_context_cognition_render` — the compact text render.
- `focusa_evidence_capture` — link the curated selection to the active Workpoint.
- `focusa_project_verify` — verify project identity on `project_root_unverified`.
- `focusa_workpoint_resume` — rehydrate the active Workpoint on discontinuity.

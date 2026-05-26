# `focusa_project_card_outcome`

**Family:** `project_identity`  
**Label:** Project Card Outcome

## Purpose

Attach a verified result to a specific project-card `algorithm_run_id` so future bootstrap and success-sequence planning can learn from the outcome.

## When to use

- After acting on a `focusa_project_card` recommendation.
- After proof/evidence exists for the selected trajectory slice.
- Before final reports when the project-card algorithm should learn from success or failure.

## Parameters

- `algorithm_run_id` — required run id returned by `focusa_project_card`.
- `actual_outcome` — required observed final result.
- `score` — optional score from `0.0` to `1.0`; defaults to `1.0` in the API.
- `evidence_refs` — optional bounded proof handles.
- `project_root` — optional project root associated with the run.
- `notes` — optional bounded result note.

## Expected result

Returns `schema=focusa.project_card_algorithm_outcome.v1`, the persisted `outcome`, storage paths, and flywheel guidance. Side effects: appends `project_card_algorithm_outcomes.jsonl` and updates `project_card_signal_weights.json`.

## Example

```text
focusa_project_card_outcome algorithm_run_id="019e..." actual_outcome="validated endpoint" score=1.0 evidence_refs=["test:pass"] project_root="/home/wirebot/focusa"
```

## Contract summary

- Family: Project Identity.
- Side effects: `write_project_card_outcome`.
- Result envelope: `tool_result_v1` with status, recovery posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/project/card/outcome`.
- CLI commands: `focusa project card-outcome`.
- Core surface: Spec98 project-card learning flywheel outcome attachment.
- Contract source: `docs/current/focusa-tool-contracts.json`.

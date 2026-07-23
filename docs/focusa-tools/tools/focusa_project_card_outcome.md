# `focusa_project_card_outcome`

Attach a final outcome/result to a specific project-card algorithm_run_id and update learned project-card weights. Use it when Attach a verified result to a project-card algorithm_run_id so project-card learning weights and future bootstrap/sequence planning can improve. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Attach a verified result to a project-card algorithm_run_id so project-card learning weights and future bootstrap/sequence planning can improve.
- Capability family: `project_identity`; namespace: `focusa.project_identity`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `algorithm_run_id` (required; string): Project-card algorithm_run_id returned by focusa_project_card.
- `actual_outcome` (required; string): Observed final outcome/result for that algorithm run.
- `score` (optional; number): Optional outcome score from 0.0 to 1.0; defaults to 1.0.
- `evidence_refs` (optional; array): Evidence refs proving the outcome.
- `project_root` (optional; string): Optional project root associated with the run.
- `notes` (optional; string): Optional bounded note about the result.
- `task_timing` (optional; structured): Optional override timing object; Pi auto-populates elapsed task timing when omitted.
- `token_usage` (optional; structured): Optional override token usage object; Pi auto-populates provider/estimated token counts when omitted.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_project_card_outcome`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "algorithm_run_id": "example",
  "actual_outcome": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_project_card_outcome.md

## Anti-examples

- assuming unsafe broad cwd is canonical
- skipping verify after scope mismatch

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_project_card_outcome`, `write_project_card_outcome`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_project_card` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_card`, `focusa_predict_record`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Runbooks: `runbook:project_identity`
- Pi: `focusa_project_card_outcome`; MCP: `focusa.project.card.outcome`; OpenAI: `focusa_project_card_outcome`.
- CLI: `focusa project card-outcome`.
- REST: `POST /v1/project/card/outcome`.
- Specification: contract registry.
- Descriptor digest: `sha256:0ae2780401554a0a537a69eab44dd021aebb242df05c663f5cc27b5618c82eb8`.

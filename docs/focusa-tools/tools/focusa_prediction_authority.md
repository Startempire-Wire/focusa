# `focusa_prediction_authority`

Append or project immutable Spec 138 prediction/outcome/learning/transfer authority in one typed project/workstream scope. Use it when Append or project immutable Spec 138 authority in typed scope. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Append or project immutable Spec 138 authority in typed scope.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (required; string | string): See the strict descriptor schema.
- `event` (optional; structured): ScopedAuthorityEvent when action=append.
- `project_root` (optional; string): Explicit or current verified project root.
- `continuity_id` (optional; string): Explicit or current continuity id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_prediction_authority`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "action": "append"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_prediction_authority.md

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_or_read_prediction_authority`, `write_or_read_prediction_authority`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_predict_recent` (likely_next)
- `focusa_evidence_capture` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_predict_recent`, `focusa_evidence_capture`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_prediction_authority`; MCP: `focusa.prediction.authority`; OpenAI: `focusa_prediction_authority`.
- CLI: `focusa predict authority-append`, `focusa predict authority-projection`.
- REST: `POST /v1/prediction-authority/events`, `POST /v1/prediction-authority/projection`, `GET /v1/prediction-authority/projection`.
- Specification: contract registry.
- Descriptor digest: `sha256:829d6b6b5c03e33c851c6f1bab99dbeb58933a99d5f5c6fee86fb8871051a9bc`.

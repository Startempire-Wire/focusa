# `focusa_predict_evaluate`

Evaluate a prediction inside its exact typed project/workstream scope. Use it when Evaluate a Focusa prediction against an actual outcome and optional score; required before final task completion when relevant predictions exist. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Evaluate a Focusa prediction against an actual outcome and optional score; required before final task completion when relevant predictions exist.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `prediction_id` (required; string): Prediction id to evaluate.
- `actual_outcome` (required; string): Observed actual outcome.
- `score` (optional; number): Score 0.0 to 1.0.
- `learning_signal_ref` (optional; string): Optional scoped learning signal ref.
- `project_root` (optional; string): Explicit or current verified project root.
- `continuity_id` (optional; string): Explicit or current workstream continuity id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_predict_evaluate`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "prediction_id": "example",
  "actual_outcome": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_predict_evaluate.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_prediction_evaluation`, `write_prediction_evaluation`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_capture` (likely_next)
- `focusa_metacog_reflect` (likely_next)
- `focusa_predict_stats` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_predict_stats`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_predict_evaluate`; MCP: `focusa.predict.evaluate`; OpenAI: `focusa_predict_evaluate`.
- CLI: `focusa predict evaluate`.
- REST: `POST /v1/predictions/{prediction_id}/evaluate`.
- Specification: contract registry.
- Descriptor digest: `sha256:9ac6b1b79fee87f2042728ae71ccd9c82cf2e1e0d9bbe621b868a9a32e1e6189`.

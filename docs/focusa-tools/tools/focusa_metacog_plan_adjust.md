# `focusa_metacog_plan_adjust`

Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement. Use it when Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `reflection_id` (required; string): Reflection id.
- `selected_updates` (optional; array): Selected updates.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_plan_adjust`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "reflection_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_plan_adjust.md

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
- Side effects: `write_state`, `write_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_evaluate_outcome` (likely_next)
- `focusa_predict_record` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_evaluate_outcome`, `focusa_predict_record`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_plan_adjust`; MCP: `focusa.metacog.plan.adjust`; OpenAI: `focusa_metacog_plan_adjust`.
- CLI: `focusa metacognition adjust`.
- REST: `POST /v1/metacognition/adjust`.
- Specification: contract registry.
- Descriptor digest: `sha256:b3d4244628e17a3889e0cae106596fa76bc9c2a67e4fff8c02d43071e694d7f8`.

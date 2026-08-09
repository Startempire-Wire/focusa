# `focusa_state_hygiene_apply`

Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update. Use it when Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `approved` (required; boolean): Must be true to apply proposal-safe hygiene.
- `reason` (optional; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_state_hygiene_apply`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "approved": false
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_state_hygiene_apply.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- hiding failures behind null/unknown
- silent deletion or cleanup

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_focus_state_note`, `write_focus_state_note`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_state_hygiene_doctor` (likely_next)
- `focusa_workpoint_resume` (likely_next)
- `focusa_tool_doctor` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_state_hygiene_doctor`, `focusa_workpoint_resume`, `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_state_hygiene_apply`; MCP: `focusa.state.hygiene.apply`; OpenAI: `focusa_state_hygiene_apply`.
- CLI: none.
- REST: `POST /v1/focus/update`.
- Specification: contract registry.
- Descriptor digest: `sha256:80c6186914a0f979d00639405f2a23042f22e8d3986131b9d38afc3bf70a980e`.

# `focusa_tool_doctor`

Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action. Use it when Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `scope` (optional; string): Optional family/surface to diagnose, e.g. workpoint, focus_state, metacog.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tool_doctor`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tool_doctor.md

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
- Side effects: `diagnostic`, `diagnostic`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `true`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_resource_mode` (likely_next)
- `focusa_project_identity` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_resource_mode`, `focusa_project_identity`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_tool_doctor`; MCP: `focusa.tool.doctor`; OpenAI: `focusa_tool_doctor`.
- CLI: none.
- REST: `GET /v1/health`, `GET /v1/workpoint/current`, `GET /v1/work-loop/status?summary_only=true`.
- Specification: contract registry.
- Descriptor digest: `sha256:c98099e6d891580b9d5d4c7e1edb496522738bd9d9f5c3e5199a3cfa737f4790`.

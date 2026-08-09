# `focusa_resource_mode`

Read or control Focusa resource mode, including activating/deactivating LowMem mode when resources are constrained. Use it when Read or control Focusa ResourceMode, including activating or deactivating LowMem mode when resources are constrained. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read or control Focusa ResourceMode, including activating or deactivating LowMem mode when resources are constrained.
- Capability family: `diagnostics_hygiene`; namespace: `focusa.diagnostics_hygiene`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string | string | string | string): Mode action. activate_lowmem enables LowMem; deactivate_lowmem clears the runtime override back to auto.
- `mode` (optional; string | string | string | string | string): Optional target mode when action=set_mode.
- `reason` (optional; string): Why the mode is being read or changed.
- `preflight` (optional; boolean): If true, only read current mode and report intended change.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_resource_mode`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_resource_mode.md

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
- Side effects: `control_state`, `control_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_traverse` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_traverse`, `focusa_trajectory_view`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Runbooks: `runbook:diagnostics_hygiene`
- Pi: `focusa_resource_mode`; MCP: `focusa.resource.mode`; OpenAI: `focusa_resource_mode`.
- CLI: `focusa resource mode`.
- REST: `GET /v1/resource/mode`, `POST /v1/resource/mode`.
- Specification: contract registry.
- Descriptor digest: `sha256:012f5d1c74cedaab865e32ff1d6f32fff9ad7308f0c7a88de1354a79763e5400`.

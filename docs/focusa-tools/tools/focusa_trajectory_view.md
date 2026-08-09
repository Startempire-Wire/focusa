# `focusa_trajectory_view`

Read the per-project Trajectory Intelligence view: project identity, goal/state/gap/evidence/drift, and next Workpoint candidate. Use it when Read the per-project Trajectory Intelligence view before acting: project identity, goal/state/gap/evidence/drift, next Workpoint candidate, and learning-loop context for task closure. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read the per-project Trajectory Intelligence view before acting: project identity, goal/state/gap/evidence/drift, next Workpoint candidate, and learning-loop context for task closure.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Optional expected project root; defaults to Pi session cwd.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to Pi session key.
- `continuity_id` (optional; string): Optional logical continuity id; defaults to Pi continuity id and is part of authority boundary.
- `mode` (optional; string | string): View mode; summary is hot-path bounded.
- `allow_prior_project_trajectory` (optional; boolean): If true, use the prior same-project trajectory as advisory reload fallback when continuity_id changed.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_trajectory_view`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_trajectory_view.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- overriding Workpoint/operator authority
- merging sessions on goal similarity alone

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_temporal_authority` (likely_next)
- `focusa_trajectory_assess` (likely_next)
- `focusa_trajectory_define_goal` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_temporal_authority`, `focusa_trajectory_assess`, `focusa_trajectory_define_goal`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_trajectory_view`; MCP: `focusa.trajectory.view`; OpenAI: `focusa_trajectory_view`.
- CLI: `focusa trajectory view`.
- REST: `GET /v1/trajectory/view`.
- Specification: contract registry.
- Descriptor digest: `sha256:7668a96a8cf8d06c5654270a5b0a3c6eff6a22077ef4935ad30cc01b7c1b6a79`.

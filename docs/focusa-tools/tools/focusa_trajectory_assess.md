# `focusa_trajectory_assess`

Assess current project state against the desired Trajectory end state and return gaps/recommended action. Use it when Assess project current state against desired Trajectory end state and return gaps/recommended action; task-boundary reviews should cross-check predictions and metacog lessons. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Assess project current state against desired Trajectory end state and return gaps/recommended action; task-boundary reviews should cross-check predictions and metacog lessons.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `observed_state` (optional; string): Observed current state override.
- `evidence_refs` (optional; array): Evidence refs supporting observed state.
- `project_root` (optional; string): Optional expected project root; defaults to Pi session cwd.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to Pi session key.
- `continuity_id` (optional; string): Optional logical continuity id; defaults to Pi continuity id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_trajectory_assess`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_trajectory_assess.md

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

- `focusa_trajectory_propose_workpoint` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_propose_workpoint`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_trajectory_assess`; MCP: `focusa.trajectory.assess`; OpenAI: `focusa_trajectory_assess`.
- CLI: `focusa trajectory assess`.
- REST: `POST /v1/trajectory/assess`.
- Specification: contract registry.
- Descriptor digest: `sha256:c8fad58603fae51983c6e476fb795d676b8d3ad898f8c0d5c34b95508b8f5ecc`.

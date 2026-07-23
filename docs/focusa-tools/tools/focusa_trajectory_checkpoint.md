# `focusa_trajectory_checkpoint`

Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation. Use it when Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `summary` (optional; string): Optional bounded Trajectory checkpoint summary.
- `project_root` (optional; string): Optional expected project root; defaults to Pi session cwd.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to Pi session key.
- `continuity_id` (optional; string): Optional logical continuity id; defaults to Pi continuity id.
- `idempotency_key` (optional; string): Optional external idempotency key.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_trajectory_checkpoint`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_trajectory_checkpoint.md

## Anti-examples

- overriding Workpoint/operator authority
- merging sessions on goal similarity alone

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `advisory_checkpoint`, `advisory_checkpoint`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_trajectory_resume` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_checkpoint`, `focusa_trajectory_resume`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_trajectory_checkpoint`; MCP: `focusa.trajectory.checkpoint`; OpenAI: `focusa_trajectory_checkpoint`.
- CLI: `focusa trajectory checkpoint`.
- REST: `POST /v1/trajectory/checkpoint`.
- Specification: contract registry.
- Descriptor digest: `sha256:b0d0f05ea5ea6a848188e3a5de7d86a82111df39d7061604155529037a63a984`.

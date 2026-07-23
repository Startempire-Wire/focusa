# `focusa_work_loop_control`

Control continuous work loop: on, pause, resume, stop. Use it when Control continuous work loop: on, pause, resume, stop. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Control continuous work loop: on, pause, resume, stop.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (required; string | string | string | string): See the strict descriptor schema.
- `reason` (optional; string): Optional operator reason (max 200 chars).
- `preset` (optional; string | string | string | string): See the strict descriptor schema.
- `preflight` (optional; boolean): If true, only report intended route/writer and do not mutate work-loop state.
- `root_work_item_id` (optional; string): Optional root BD/task/item id. If omitted, tool infers from active task or bd ready.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_control`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "action": "on"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_control.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

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

- `focusa_work_loop_writer_status` (likely_next)
- `focusa_work_loop_status` (likely_next)
- `focusa_work_loop_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_writer_status`, `focusa_work_loop_status`, `focusa_work_loop_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_control`; MCP: `focusa.work.loop.control`; OpenAI: `focusa_work_loop_control`.
- CLI: none.
- REST: `POST /v1/work-loop/enable`, `POST /v1/work-loop/pause`, `POST /v1/work-loop/resume`, `POST /v1/work-loop/stop`.
- Specification: contract registry.
- Descriptor digest: `sha256:daafbec36326d8fc18a1ba681421c5847e0f2cfe8db6e19c658d0d7274ced9c2`.

# `focusa_work_loop_select_next`

Ask daemon to defer blocked work and select next ready work item. Use it when Ask daemon to defer blocked work and select next ready work item. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Ask daemon to defer blocked work and select next ready work item.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `parent_work_item_id` (optional; string): Parent work item id. If omitted, use active current_task work_item_id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_select_next`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_select_next.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `select_next_work`, `select_next_work`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_work_loop_context` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_checkpoint`, `focusa_work_loop_context`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_select_next`; MCP: `focusa.work.loop.select.next`; OpenAI: `focusa_work_loop_select_next`.
- CLI: none.
- REST: `POST /v1/work-loop/select-next`.
- Specification: contract registry.
- Descriptor digest: `sha256:e67deac89a3e482f0dcc2544bc5070c1208d0a3f3f77e5cbd46c813c0b04504c`.

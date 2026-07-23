# `focusa_work_loop_status`

Get current continuous work-loop state and budgets. Use it when Get current continuous work-loop state and budgets. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Get current continuous work-loop state and budgets.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- No arguments.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_status`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_status.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_only`, `read_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `true`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_work_loop_writer_status` (likely_next)
- `focusa_work_loop_context` (likely_next)
- `focusa_work_loop_select_next` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_writer_status`, `focusa_work_loop_context`, `focusa_work_loop_select_next`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_status`; MCP: `focusa.work.loop.status`; OpenAI: `focusa_work_loop_status`.
- CLI: none.
- REST: `GET /v1/work-loop/status?summary_only=true`.
- Specification: contract registry.
- Descriptor digest: `sha256:b5d39927dc1a4f5a6d68143d497de3844b6e8b3885a53861c39b46098c711ab9`.

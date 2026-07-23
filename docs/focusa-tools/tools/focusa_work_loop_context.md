# `focusa_work_loop_context`

Update continuation decision context (current ask/scope/steering). Use it when Update continuation decision context (current ask/scope/steering). It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Update continuation decision context (current ask/scope/steering).
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `current_ask` (required; string): Current ask for continuation context (max 240 chars).
- `ask_kind` (optional; string): ask_kind hint (optional).
- `scope_kind` (optional; string): scope_kind hint (optional).
- `carryover_policy` (optional; string): carryover policy hint (optional).
- `excluded_context_reason` (optional; string): Reason for excluding carryover context (optional).
- `excluded_context_labels` (optional; array): See the strict descriptor schema.
- `operator_steering_detected` (optional; boolean): See the strict descriptor schema.
- `source_turn_id` (optional; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_context`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "current_ask": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_context.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_context`, `write_context`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_work_loop_checkpoint` (likely_next)
- `focusa_work_loop_status` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_checkpoint`, `focusa_work_loop_status`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_context`; MCP: `focusa.work.loop.context`; OpenAI: `focusa_work_loop_context`.
- CLI: none.
- REST: `POST /v1/work-loop/context`.
- Specification: contract registry.
- Descriptor digest: `sha256:df9b65d155b08daf54255f7825cad745e8d09dfa556ebc2ff712a0ea4e6f67db`.

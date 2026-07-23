# `focusa_call_stack_design`

Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold (entry → handlers → services → adapters → storage → output) that the operator/agent fills in for the specific feature. Per Spec 103. Use it when Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root for the design. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity id filter.
- `mission` (required; string): Short description of the feature this design covers.
- `entry_surface` (optional; string | string | string): Entry surface kind (default: pi_tool).
- `entry_name` (required; string): Proposed tool/command/route name.
- `workpoint_id` (optional; string): Workpoint to attach the design to (required when attach_to_workpoint=true).
- `attach_to_workpoint` (optional; boolean): When true, the design becomes focusa_evidence linked to the active Workpoint.
- `attach_to_stg` (optional; boolean): When true, the design sets the active STG of the active Trajectory.
- `parent_design_id` (optional; string): Optional parent design id to chain refinements.
- `notes` (optional; string): Optional bounded free-form notes.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_call_stack_design`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "mission": "example",
  "entry_name": "focusa_workpoint_resume"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_call_stack_design.md

## Anti-examples

- broad roots such as /root
- parallel memory outside the active project+continuity scope

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_call_stack_design`, `write_call_stack_design`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_call_stack_verify` (likely_next)
- `focusa_workpoint_link_evidence` (likely_next)
- `focusa_trajectory_assess` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_call_stack_verify`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_call_stack_design`; MCP: `focusa.call.stack.design`; OpenAI: `focusa_call_stack_design`.
- CLI: `focusa call-stack design`.
- REST: `POST /v1/call-stack/design`.
- Specification: contract registry.
- Descriptor digest: `sha256:70f2f0c5060395cdcda7b10d80e21c5cd63e705747a4eccbeb4da82189d2b474`.

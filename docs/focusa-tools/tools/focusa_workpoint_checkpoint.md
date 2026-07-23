# `focusa_workpoint_checkpoint`

Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint. Use it when Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `current_ask` (optional; string): Current operator ask or mission framing.
- `work_item_id` (optional; string): Beads/work item id, e.g. focusa-a2w2.6.
- `continuity_id` (optional; string): Stable logical session/workstream id; defaults to this Pi session continuity id.
- `checkpoint_reason` (optional; string): manual|operator_checkpoint|before_compact|after_compact|context_overflow|session_resume|model_switch|fork
- `mission` (required; string): Current mission/objective to preserve across compaction.
- `target_objects` (optional; array): Ontology/file/component/endpoint refs currently targeted.
- `current_action` (optional; string): Typed action, e.g. patch_component_binding or resume_workpoint.
- `verified_evidence` (optional; array): Short evidence refs/results already verified; use handles, not raw logs.
- `blockers` (optional; array): Open blockers or drift boundaries.
- `next_action` (required; string): Exact bounded next action to resume after compact/retry.
- `do_not_drift` (optional; array): Actions/areas the next agent must not drift into.
- `source_turn_id` (optional; string): Pi/source turn id for provenance.
- `idempotency_key` (optional; string): Optional external idempotency key.
- `canonical` (optional; boolean): False only for degraded fallback packets.
- `project_root` (optional; string): Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_workpoint_checkpoint`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "mission": "example",
  "next_action": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_workpoint_checkpoint.md

## Anti-examples

- broad roots such as /root
- parallel memory outside the active project+continuity scope

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `checkpoint`, `checkpoint`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_resume` (likely_next)
- `focusa_active_object_resolve` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_resume`, `focusa_active_object_resolve`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_workpoint_checkpoint`; MCP: `focusa.workpoint.checkpoint`; OpenAI: `focusa_workpoint_checkpoint`.
- CLI: `focusa workpoint checkpoint`.
- REST: `POST /v1/workpoint/checkpoint`.
- Specification: contract registry.
- Descriptor digest: `sha256:58967fd9bfaf244edd059e56bae48e0af08f5a7f5ddf20a6e6ddf84ff413cf60`.

# `focusa_workpoint_resume`

Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action. Use it when Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `workpoint_id` (optional; string): Specific workpoint id; omit to use active workpoint.
- `continuity_id` (optional; string): Stable logical session/workstream id; defaults to this Pi session continuity id.
- `session_id` (optional; string): Optional temporal Pi session id; defaults to this Pi session key.
- `mode` (optional; string): compact_prompt|full_json|operator_summary
- `project_root` (optional; string): Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe.
- `current_ask` (optional; string): Optional latest operator ask used to compute current-action authority; defaults to Pi current ask.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_workpoint_resume`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_workpoint_resume.md

## Anti-examples

- broad roots such as /root
- parallel memory outside the active project+continuity scope

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_only`, `read_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_trajectory_view` (likely_next)
- `focusa_active_object_resolve` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_trajectory_view`, `focusa_active_object_resolve`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_workpoint_resume`; MCP: `focusa.workpoint.resume`; OpenAI: `focusa_workpoint_resume`.
- CLI: `focusa workpoint resume`.
- REST: `POST /v1/workpoint/resume`.
- Specification: contract registry.
- Descriptor digest: `sha256:ab9c9bbe2ed5cd1be3c2be60126fbc6114de34d72116a188fdd0b734de5b3e68`.

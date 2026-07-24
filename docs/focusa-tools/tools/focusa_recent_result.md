# `focusa_recent_result`

Record a completed result, output, or reference (max 180 chars). Use it when Record a completed result, output, or reference (max 180 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Record a completed result, output, or reference (max 180 chars).
- Capability family: `focus_state`; namespace: `focusa.focus_state`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `result` (required; string): Recent result (max 180 chars).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_recent_result`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "result": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_recent_result.md

## Anti-examples

- raw transcript dumping
- source-of-truth replacement for Workpoint continuation

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_state`, `write_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_evidence_capture` (likely_next)
- `focusa_trajectory_assess` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_evidence_capture`, `focusa_trajectory_assess`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:focus_state`
- Pi: `focusa_recent_result`; MCP: `focusa.recent.result`; OpenAI: `focusa_recent_result`.
- CLI: `focusa focus update --recent-result`.
- REST: `POST /v1/focus/update`.
- Specification: contract registry.
- Descriptor digest: `sha256:d205d47351eaab35aa77a492473d555ec66c57c22b941080217bf54915f36a93`.

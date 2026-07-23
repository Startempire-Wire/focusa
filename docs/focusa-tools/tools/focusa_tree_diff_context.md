# `focusa_tree_diff_context`

Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints. Use it when Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints.
- Capability family: `tree_lineage`; namespace: `focusa.tree_lineage`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `from_snapshot_id` (required; string): Source snapshot id.
- `to_snapshot_id` (required; string): Target snapshot id.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tree_diff_context`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "from_snapshot_id": "example",
  "to_snapshot_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tree_diff_context.md

## Anti-examples

- treating lineage as current project authority
- restore without explicit rollback intent

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

- `focusa_tree_restore_state` (likely_next)
- `focusa_tree_path` (likely_next)
- `focusa_metacog_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tree_restore_state`, `focusa_tree_path`, `focusa_metacog_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:tree_lineage`
- Pi: `focusa_tree_diff_context`; MCP: `focusa.tree.diff.context`; OpenAI: `focusa_tree_diff_context`.
- CLI: `focusa state snapshot diff`.
- REST: `POST /v1/focus/snapshots/diff`.
- Specification: contract registry.
- Descriptor digest: `sha256:49e9a45cbbcbb6b693d4e2125138ee8f968b73d5125716e74c1dab9cbf8d726f`.

# `focusa_tree_head`

Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work. Use it when Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work.
- Capability family: `tree_lineage`; namespace: `focusa.tree_lineage`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `session_id` (optional; string): Optional session id scoping hint.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tree_head`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tree_head.md

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

- `focusa_tree_path` (likely_next)
- `focusa_tree_snapshot_state` (likely_next)
- `focusa_lineage_tree` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tree_path`, `focusa_tree_snapshot_state`, `focusa_lineage_tree`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:tree_lineage`
- Pi: `focusa_tree_head`; MCP: `focusa.tree.head`; OpenAI: `focusa_tree_head`.
- CLI: `focusa lineage head`.
- REST: `GET /v1/lineage/head`.
- Specification: contract registry.
- Descriptor digest: `sha256:957f2a47c229a8038f1bb0a6c281d29a9fc1a8d4a5d2c5dc33b193881293c338`.

# `focusa_tree_recent_snapshots`

Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id. Use it when Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id.
- Capability family: `tree_lineage`; namespace: `focusa.tree_lineage`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `limit` (optional; integer; min=1, max=20): How many recent snapshots to return (default 5).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tree_recent_snapshots`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tree_recent_snapshots.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

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

- `focusa_tree_diff_context` (likely_next)
- `focusa_tree_snapshot_compare_latest` (likely_next)
- `focusa_tree_snapshot_state` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tree_diff_context`, `focusa_tree_snapshot_compare_latest`, `focusa_tree_snapshot_state`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:tree_lineage`
- Pi: `focusa_tree_recent_snapshots`; MCP: `focusa.tree.recent.snapshots`; OpenAI: `focusa_tree_recent_snapshots`.
- CLI: `focusa state snapshot recent`.
- REST: `GET /v1/focus/snapshots/recent`.
- Specification: contract registry.
- Descriptor digest: `sha256:e089a60fa0b98c90f98582b3b3e0642c32519a5348fb73c0b4e037f2c3772e31`.

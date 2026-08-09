# `focusa_tree_snapshot_state`

Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason. Use it when Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason.
- Capability family: `tree_lineage`; namespace: `focusa.tree_lineage`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `clt_node_id` (optional; string): Optional CLT node id. Defaults to current head.
- `snapshot_reason` (optional; string): Reason label for snapshot.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tree_snapshot_state`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tree_snapshot_state.md

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

- `focusa_tree_recent_snapshots` (likely_next)
- `focusa_tree_diff_context` (likely_next)
- `focusa_tree_restore_state` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tree_recent_snapshots`, `focusa_tree_diff_context`, `focusa_tree_restore_state`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:tree_lineage`
- Pi: `focusa_tree_snapshot_state`; MCP: `focusa.tree.snapshot.state`; OpenAI: `focusa_tree_snapshot_state`.
- CLI: `focusa state snapshot create`.
- REST: `POST /v1/focus/snapshots`.
- Specification: contract registry.
- Descriptor digest: `sha256:5ca84115499b7d8da35a7e2b6bec35826d2cd0da485fe57147ae90ea1a25c828`.

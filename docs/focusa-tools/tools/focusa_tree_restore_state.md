# `focusa_tree_restore_state`

Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool. Use it when Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool.
- Capability family: `tree_lineage`; namespace: `focusa.tree_lineage`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `snapshot_id` (required; string): Snapshot id to restore.
- `restore_mode` (optional; string | string): Restore mode: exact|merge

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tree_restore_state`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "snapshot_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tree_restore_state.md

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
- Read-only: `true`; destructive: `true`; idempotent: `true`; open-world: `false`.
- Confirmation required: `true`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_tree_head` (likely_next)
- `focusa_tree_path` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tree_head`, `focusa_tree_path`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:tree_lineage`
- Pi: `focusa_tree_restore_state`; MCP: `focusa.tree.restore.state`; OpenAI: `focusa_tree_restore_state`.
- CLI: `focusa state snapshot restore`.
- REST: `POST /v1/focus/snapshots/restore`.
- Specification: contract registry.
- Descriptor digest: `sha256:731327943438d0ea3138b5595db7c75cedc885863ffcb33d371b087c1c17698d`.

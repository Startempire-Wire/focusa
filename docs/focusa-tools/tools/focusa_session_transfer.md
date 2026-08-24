# `focusa_session_transfer`

Typed save/continue/rollover wrapper for moving long work between Pi sessions without forking or continuity-id fingerprint fallback. Use it when Save, continue, or Spec130-roll over a long Focusa/Pi work session with explicit source_scope/target_scope or target_continuity_id, source/target session ids, checkpoint/packet refs, and rollover action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Save, continue, or Spec130-roll over a long Focusa/Pi work session with explicit source_scope/target_scope or target_continuity_id, source/target session ids, checkpoint/packet refs, and rollover action.
- Capability family: `workpoint`; namespace: `focusa.workpoint`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (required; string): save|continue|status|rollover
- `rollover_action` (optional; string | string | string | string | string | string | string): Spec130 rollover action; required for rotating continuity workflows.
- `source_scope` (optional; object): See the strict descriptor schema.
- `target_scope` (optional; object): See the strict descriptor schema.
- `source_working_subpath_id` (optional; string): Source WorkingSubpath id; defaults to active context or primary.
- `target_working_subpath_id` (optional; string): Explicit target WorkingSubpath id for auditable cross-worktree transfer.
- `target_continuity_id` (optional; string): Explicit target continuity id when target_scope is same root with rotated continuity.
- `source_session_id` (optional; string): Source/native Pi session id.
- `target_session_id` (optional; string): Target/native Pi session id after rollover/transfer.
- `checkpoint_ref` (optional; string): Pre-created checkpoint ref to bind transfer.
- `workpoint_packet_ref` (optional; string): Workpoint/resume packet ref to bind transfer.
- `compaction_packet_ref` (optional; string): Spec130 compaction mission packet ref.
- `project_root` (optional; string): Deprecated convenience source root; prefer source_scope.root_path.
- `current_ask` (optional; string): Current resume/save intent.
- `mission` (optional; string): Optional save mission; defaults to current ask or inferred Workpoint mission.
- `next_action` (optional; string): Optional exact next action for save.
- `continuity_id` (optional; string): Deprecated source continuity id; prefer source_scope.continuity_id.
- `write_preload` (optional; boolean): Request preload write guidance; defaults false and never writes implicitly.
- `preload_target` (optional; string | string | string | string | string | string): Target agent surface; defaults cursor.
- `preload_mode` (optional; string | string | string | string | string): Preload mode; defaults session_transfer.
- `receipt_preview` (optional; boolean): Return a bounded receipt preview; defaults true.
- `receipt_commit` (optional; boolean): Explicitly commit the preload receipt; defaults false.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_session_transfer`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "action": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_session_transfer.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- broad roots such as /root
- parallel memory outside the active project+continuity scope

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"explicit_source_target_scope"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `save_may_checkpoint_workpoint`, `save_may_checkpoint_workpoint`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workpoint_resume` (likely_next)
- `focusa_project_card` (likely_next)
- `focusa_trajectory_view` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workpoint_resume`, `focusa_project_card`, `focusa_trajectory_view`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:workpoint`
- Pi: `focusa_session_transfer`; MCP: `focusa.session.transfer`; OpenAI: `focusa_session_transfer`.
- CLI: `focusa project session-transfer`.
- REST: `POST /v1/project/session-transfer`, `GET /v1/project/card`, `POST /v1/workpoint/checkpoint`, `POST /v1/workpoint/resume`, `GET /v1/trajectory/view`.
- Assignable: `true`; parity: `full`.
- Specification: contract registry.
- Descriptor digest: `sha256:61ff46acb5eb657ca2c3c05511cbe5ed1354872bf0355e6972723f81d3a14b13`.

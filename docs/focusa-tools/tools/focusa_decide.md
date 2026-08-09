# `focusa_decide`

Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists. Use it when Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists.
- Capability family: `focus_state`; namespace: `focusa.focus_state`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `decision` (required; string): ONE crystallized architectural choice — what was decided and why (max 160 chars). NOT a task list or debugging note.
- `rationale` (optional; string): Context: why this decision was made (max 200 chars). Summarize from scratchpad notes.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_decide`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "decision": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_decide.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

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

- `focusa_project_identity` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:focus_state`
- Pi: `focusa_decide`; MCP: `focusa.decide`; OpenAI: `focusa_decide`.
- CLI: `focusa focus update --decision`.
- REST: `POST /v1/focus/update`.
- Specification: contract registry.
- Descriptor digest: `sha256:0d17e612eb2d79ac658d340004e8e8f15fbc93608c7ef68b0c5adb8fa26ed465`.

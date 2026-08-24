# `focusa_work_loop_control`

Control continuous work loop: on, pause, resume, stop. Use it when Control continuous work loop: on, pause, resume, stop. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Control continuous work loop: on, pause, resume, stop.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (required; string | string | string | string): See the strict descriptor schema.
- `reason` (optional; string): Optional operator reason (max 200 chars).
- `preset` (optional; string | string | string | string): See the strict descriptor schema.
- `preflight` (optional; boolean): If true, only report intended route/writer and do not mutate work-loop state.
- `root_work_item_id` (optional; string): Optional root provider WorkItem id. If omitted, infer from the active scoped task.
- `renew_budget` (optional; boolean): Explicitly start a fresh budget epoch when action=resume.
- `max_turns` (optional; number; min=1): See the strict descriptor schema.
- `max_wall_clock_ms` (optional; number; min=1000): See the strict descriptor schema.
- `max_retries` (optional; number; min=0): See the strict descriptor schema.
- `cooldown_ms` (optional; number; min=0): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_work_loop_control`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "action": "on"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_work_loop_control.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `control_state`, `control_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_work_loop_writer_status` (likely_next)
- `focusa_work_loop_status` (likely_next)
- `focusa_work_loop_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_writer_status`, `focusa_work_loop_status`, `focusa_work_loop_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_work_loop_control`; MCP: `focusa.work.loop.control`; OpenAI: `focusa_work_loop_control`.
- CLI: none.
- REST: `POST /v1/work-loop/enable`, `POST /v1/work-loop/pause`, `POST /v1/work-loop/resume`, `POST /v1/work-loop/stop`.
- Assignable: `true`; parity: `domain`.
- Specification: contract registry.
- Descriptor digest: `sha256:1d6731f5ad056deb351fea3f1258efc9c624aca0deda59218874036dc6382fc0`.

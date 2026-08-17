# `focusa_bg_run`

Run a terminal-blocking command in the background as a first-class Focusa job. The daemon records the job durably; on completion the agent's front terminal receives the completion notification with a bounded output tail (no polling). Canonical TBQ dispatch primitive — use instead of raw setsid/nohup shells whenever the Focusa daemon is up. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done.
- Capability family: `background_job`; namespace: `focusa.background_job`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `name` (required; string): Human job name (appears in the completion notification).
- `command` (required; string): The full command line to execute (after -- semantics).
- `cwd` (optional; string): Working directory (defaults to the current session cwd).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_bg_run`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "name": "focusa_workpoint_resume",
  "command": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_bg_run.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- when another narrower tool is explicitly indicated

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `durable_dispatch`, `durable_dispatch`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_bg_status` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_bg_status`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-silent-sessions`
- Runbooks: `runbook:background_job`
- Pi: `focusa_bg_run`; MCP: `focusa.bg.run`; OpenAI: `focusa_bg_run`.
- CLI: none.
- REST: `/v1/background-jobs `.
- Specification: contract registry.
- Descriptor digest: `sha256:eba3886251790b86ae7ae5d6ef010a17a68f985d54838f3710102dcc910e7d13`.

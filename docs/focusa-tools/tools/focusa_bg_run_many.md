# `focusa_bg_run_many`

Dispatch multiple terminal-blocking jobs in parallel as first-class Focusa jobs. Each job completes independently and delivers its completion notification (with bounded output tail) to the agent front terminal via SSE — the orchestration primitive for parallel builds, test shards, and multi-step pipelines. Returns the job ledger immediately; never blocks. Use it when Dispatch or inspect durable background jobs through Focusa's canonical execution surface. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Dispatch or inspect durable background jobs through Focusa's canonical execution surface.
- Capability family: `background_job`; namespace: `focusa.background_job`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `jobs` (required; array): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_bg_run_many`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "jobs": []
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_bg_run_many.md

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
- Pi: `focusa_bg_run_many`; MCP: `focusa.bg.run.many`; OpenAI: `focusa_bg_run_many`.
- CLI: `focusa bg run`.
- REST: Pi-local only.
- Assignable: `true`; parity: `daemon_backed`.
- Specification: contract registry.
- Descriptor digest: `sha256:5dc7c618cde9be335f3bc2f04e8b113361f085aa74cc92d412d611de64fbd20c`.

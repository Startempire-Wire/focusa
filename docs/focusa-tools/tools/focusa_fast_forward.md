# `focusa_fast_forward`

Fast-forward session completion by multiplying parallel workloop-bound silent sessions (2x/4x/6x/8x...). Compiles the deterministic FanoutPlan — round-robin task division across lanes with per-lane policy budgets — then returns the plan; each lane executes as one silent session bound to its work items (docs/168, #312). Use it when Execute governed Silent Session fanout only when the daemon fanout router is live. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Execute governed Silent Session fanout only when the daemon fanout router is live.
- Capability family: `session_fanout`; namespace: `focusa.session_fanout`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `multiplier` (required; number): Speed multiplier: 2, 4, 6, 8...
- `work_items` (required; array): See the strict descriptor schema.
- `policy_max_turns_per_session` (optional; number): Per-lane turn cap (default 12).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_fast_forward`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "multiplier": 0,
  "work_items": []
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_fast_forward.md

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
- Runbooks: `runbook:session_fanout`
- Pi: `focusa_fast_forward`; MCP: `focusa.fast.forward`; OpenAI: `focusa_fast_forward`.
- CLI: none.
- REST: Pi-local only.
- Assignable: `false`; parity: `unavailable_unregistered_route`.
- This capability is unavailable because its daemon router is not registered. Declared unavailable routes: `POST /v1/silent-sessions/fanout`.
- Specification: contract registry.
- Descriptor digest: `sha256:9fa23cad505904b83811b1d99e805fce78c2f3494af57274579527a4acda98f6`.

# `focusa_cockpit_projection`

Read the whole flywheel in one bounded payload: workset summaries, open CallGraph run frontiers, direction steers, and the background-job board with ETAs. Read-only, ledger-backed; the hand-in-glove operator view. Use it when Read the bounded daemon-owned cockpit projection. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read the bounded daemon-owned cockpit projection.
- Capability family: `cockpit`; namespace: `focusa.cockpit`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root scope (defaults to the session cwd).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_cockpit_projection`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_cockpit_projection.md

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
- Side effects: `read_projection`, `read_projection`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_workset_projection` (likely_next)
- `focusa_bg_status` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_workset_projection`, `focusa_bg_status`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:cockpit`
- Pi: `focusa_cockpit_projection`; MCP: `focusa.cockpit.projection`; OpenAI: `focusa_cockpit_projection`.
- CLI: none.
- REST: Pi-local only.
- Assignable: `false`; parity: `unavailable_unregistered_route`.
- This capability is unavailable because its daemon router is not registered. Declared unavailable routes: `GET /v1/cockpit/projection`.
- Specification: contract registry.
- Descriptor digest: `sha256:e9b298548af2309fd00616fd083875b10e9716ba448ff21aaf118b05d0033b94`.

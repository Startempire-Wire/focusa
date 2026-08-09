# `focusa_agent_prompt`

Read canonical Pi guidance; prefer focusa_* tools over raw daemon calls. Use it when Retrieve the Pi-aware daemon reminder and canonical tool-layer guidance to prevent raw curl/fetch drift. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Retrieve the Pi-aware daemon reminder and canonical tool-layer guidance to prevent raw curl/fetch drift.
- Capability family: `focus_state`; namespace: `focusa.focus_state`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- No arguments.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_agent_prompt`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_agent_prompt.md

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
- Side effects: `read_only`, `read_only`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_tool_doctor` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_project_identity` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tool_doctor`, `focusa_trajectory_view`, `focusa_project_identity`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Runbooks: `runbook:focus_state`
- Pi: `focusa_agent_prompt`; MCP: `focusa.agent.prompt`; OpenAI: `focusa_agent_prompt`.
- CLI: none.
- REST: `GET /v1/agent/prompt`.
- Specification: contract registry.
- Descriptor digest: `sha256:2a07462ccd20927f58d2fb56f34ce380e17a64c1cca64c0ac18e7d51c1ddd5ec`.

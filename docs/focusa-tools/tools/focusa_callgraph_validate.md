# `focusa_callgraph_validate`

Validate a CallGraph definition against the Spec 155 structural rules (identity, endpoints, entries, joins, compensation, per-cycle policy). Pure + deterministic. Use it when Validate or observe a daemon-owned CallGraph when its router is live. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Validate or observe a daemon-owned CallGraph when its router is live.
- Capability family: `callgraph`; namespace: `focusa.callgraph`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `graph` (required; object): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_callgraph_validate`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "graph": {}
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_callgraph_validate.md

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
- Side effects: `read_validation`, `read_validation`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_callgraph_observe` (likely_next)
- `focusa_tool_describe` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_callgraph_observe`, `focusa_tool_describe`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Runbooks: `runbook:callgraph`
- Pi: `focusa_callgraph_validate`; MCP: `focusa.callgraph.validate`; OpenAI: `focusa_callgraph_validate`.
- CLI: none.
- REST: Pi-local only.
- Assignable: `false`; parity: `unavailable_unregistered_route`.
- This capability is unavailable because its daemon router is not registered. Declared unavailable routes: `POST /v1/callgraphs/validate`.
- Specification: contract registry.
- Descriptor digest: `sha256:1eb1fed567949457fbcf47becdc970a8ac410607cfe9d3fbd4354f129849a2d2`.

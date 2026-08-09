# `focusa_tool_describe`

Cold-load one complete runtime Focusa tool definition after search. Returns strict input/output schemas, operational guidance, authority, side effects, failures, recovery, dependencies, skills, docs, and protocol bindings without loading unrelated tools. Use it when Cold-load one complete capability contract including strict schemas and recovery. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Cold-load one complete capability contract including strict schemas and recovery.
- Capability family: `traversal`; namespace: `focusa.traversal`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `name` (required; string): Exact Focusa Pi tool name returned by focusa_tool_search.
- `include_schemas` (optional; boolean; default=true): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tool_describe`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "name": "focusa_workpoint_resume"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tool_describe.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- full payloads by default
- unbounded history/tree/ontology reads

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"agent"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_tool_graph` (likely_next)
- `focusa_agent_card` (likely_next)
- `focusa_tool_search` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tool_graph`, `focusa_agent_card`, `focusa_tool_search`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Runbooks: `runbook:traversal`
- Pi: `focusa_tool_describe`; MCP: `focusa.tool.describe`; OpenAI: `focusa_tool_describe`.
- CLI: `focusa help all --json`.
- REST: `GET /v1/agent/tools/{name}`.
- Specification: `docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md`.
- Descriptor digest: `sha256:772ace59faf3a056e70a8302a4ae4f762b9d5d37ad12965fbfbb084ea0bff47b`.

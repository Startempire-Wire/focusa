# `focusa_tool_bundle`

Load a bounded family bundle of capability metadata and optionally strict schemas. Use after search or graph traversal when one workflow needs several related tools; avoid broad all-tool prompt injection. Use it when Load one bounded capability family with schemas deferred by default. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Load one bounded capability family with schemas deferred by default.
- Capability family: `traversal`; namespace: `focusa.traversal`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `family` (required; string): Exact Focusa tool family.
- `include_schemas` (optional; boolean; default=false): See the strict descriptor schema.
- `limit` (optional; integer; min=1, max=50, default=25): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_tool_bundle`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "family": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_tool_bundle.md

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

- `focusa_tool_describe` (likely_next)
- `focusa_tool_graph` (likely_next)
- `focusa_tool_search` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_tool_describe`, `focusa_tool_graph`, `focusa_tool_search`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Runbooks: `runbook:traversal`
- Pi: `focusa_tool_bundle`; MCP: `focusa.tool.bundle`; OpenAI: `focusa_tool_bundle`.
- CLI: `focusa help all --json`.
- REST: `GET /v1/agent/tool-bundles`.
- Assignable: `true`; parity: `full`.
- Specification: `docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md`.
- Descriptor digest: `sha256:af5c766df4c742da2b76313aa6baed111241164a4786f7b2af4718d3ff70166a`.

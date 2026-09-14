# `focusa_traverse`

Read-only surgical traversal across large Focusa surfaces. Use for bounded lineage, ontology, evidence, telemetry, Workpoint, and registry slices instead of full payloads. Use it when Read-only surgical traversal across large Focusa surfaces using bounded selectors, cursors, field projection, tags, and cold full-payload guards. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read-only surgical traversal across large Focusa surfaces using bounded selectors, cursors, field projection, tags, and cold full-payload guards.
- Capability family: `traversal`; namespace: `focusa.traversal`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.
- Evidence, ECS references, and trajectory projections can be inspected in HLT/STG-aligned bounded slices without requesting full payloads.

## Parameters and strict input schema

- `surface` (required; string): Surface: lineage|ontology|focus_stack|workpoints|evidence|telemetry|tool_registry etc.
- `selector` (optional; string): Selector: window|head|path|children|neighborhood|summaries|search|recent|tags_verify.
- `anchor` (optional; string): Optional anchor id/tag/ref.
- `query` (optional; string): Optional search/filter query.
- `cursor` (optional; string): Optional cursor/offset token.
- `limit` (optional; integer; min=1, max=200): Bounded result limit.
- `depth` (optional; integer; min=1, max=64): Traversal depth cap.
- `radius` (optional; integer; min=1, max=8): Neighborhood radius cap.
- `fields` (optional; array): Optional projected fields.
- `tags` (optional; array): Optional traversal tags to verify as strings or TraverseTagRef-style objects.
- `tag_mode` (optional; string | string | string | string | string): Traversal tag mode; defaults mixed.
- `include_payload` (optional; boolean): Spec96 alias for explicit cold opt-in larger payload; defaults false.
- `include_full_payload` (optional; boolean): Compatibility alias for explicit cold opt-in larger payload; defaults false.
- `include_rehydrate_refs` (optional; boolean): Include rehydrate refs for omitted/cold slices.
- `budget_tokens` (optional; integer; min=1, max=20000): Optional token budget hint.
- `session_identity` (optional; structured): Optional FocusaSessionIdentity envelope for scoped traversal.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_traverse`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "surface": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_traverse.md

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

- Scope: `{"kind":"read","route_family":"auto"}`
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

- `focusa_active_object_resolve` (likely_next)
- `focusa_evidence_capture` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_active_object_resolve`, `focusa_evidence_capture`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Runbooks: `runbook:traversal`
- Pi: `focusa_traverse`; MCP: `focusa.traverse`; OpenAI: `focusa_traverse`.
- CLI: none.
- REST: `POST /v1/traverse`, `POST /v1/traverse/verify-tags`.
- Specification: contract registry.
- Descriptor digest: `sha256:cd7132926507e5eaed7dc20843d4a9f713a2d318c281995fb453422fc3b88c61`.

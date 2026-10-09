# `focusa_reflex_primitives`

List bounded Spec97 Reflex Primitive summaries by family/query; read-only routing metadata, never mutation authority. Use it when Read bounded Spec97 Reflex Primitive summaries by family/query from the read-only registry; advisory routing metadata only, never mutation authority. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Read bounded Spec97 Reflex Primitive summaries by family/query from the read-only registry; advisory routing metadata only, never mutation authority.
- Capability family: `traversal`; namespace: `focusa.traversal`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `family` (optional; string): Optional primitive family filter, e.g. recovery, evidence, resource.
- `query` (optional; string): Optional risk/object/action search text.
- `limit` (optional; integer; min=1, max=50): Bounded result limit.
- `include_payload` (optional; boolean): Cold opt-in for full primitive payloads; default false.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_reflex_primitives`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_reflex_primitives.md

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
- Advisory source hints (not dispatch/replay guarantees): read-only `true`; destructive `false`; idempotent `true`; open-world `false`. Verify selected action/schema/policy separately.
- Confirmation required: `unknown (null)`; preview supported: `unknown (null)`. Inspect the selected action, strict installed schema and policy; unknown is not permission.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_traverse` (likely_next)
- `focusa_tool_doctor` (likely_next)
- `focusa_workpoint_resume` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_traverse`, `focusa_tool_doctor`, `focusa_workpoint_resume`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Runbooks: `runbook:traversal`
- Pi: `focusa_reflex_primitives`; MCP: `focusa.reflex.primitives`; OpenAI: `focusa_reflex_primitives`.
- CLI: none.
- REST: `GET /v1/reflex/primitives`.
- Specification: contract registry.
- Descriptor digest: `sha256:110a7de5c130c4c6ccaded1425414bd199e41aecfc34e5f5d53c101ab152f00c`.

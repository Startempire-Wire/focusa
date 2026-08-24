# `focusa_context_cognition`

Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Returns a typed packet describing scope, authority, freshness, selected context, ontology frame, evidence frame, reasoning frame, optimization frame, and route frame. Never mutates state. Use it when Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Never mutates state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Never mutates state.
- Capability family: `trajectory`; namespace: `focusa.trajectory`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (optional; string): Project root for the packet. Defaults to Pi session cwd.
- `continuity_id` (optional; string): Optional continuity id filter.
- `session_id` (optional; string): Optional session id filter.
- `include_rehydrate_refs` (optional; boolean): When true, return rehydrate_refs for each surface.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_context_cognition`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_context_cognition.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- overriding Workpoint/operator authority
- merging sessions on goal similarity alone

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
- `focusa_workpoint_checkpoint` (likely_next)
- `focusa_evidence_capture` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:trajectory`
- Pi: `focusa_context_cognition`; MCP: `focusa.context.cognition`; OpenAI: `focusa_context_cognition`.
- CLI: `focusa context-cognition view`.
- REST: `GET /v1/context-cognition`.
- Assignable: `true`; parity: `full`.
- Specification: contract registry.
- Descriptor digest: `sha256:e23a13087abfb98c11c9c8ecfcc756b3ae10623bd9b8df7188622fe779b68c7c`.

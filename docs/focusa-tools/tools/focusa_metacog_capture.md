# `focusa_metacog_capture`

Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson. Use it when Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson.
- Capability family: `metacognition`; namespace: `focusa.metacognition`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.
- Captures retain HLT/MLG/STG alignment within the active project_root + continuity_id scope for trajectory-context retrieval.

## Parameters and strict input schema

- `kind` (required; string): Signal kind.
- `content` (required; string): Signal content.
- `rationale` (optional; string): Optional rationale.
- `evidence_refs` (optional; array): Evidence refs supporting this learning signal.
- `confidence` (optional; number; min=0, max=1): Optional confidence 0..1
- `strategy_class` (optional; string): Optional strategy class.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_metacog_capture`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "kind": "example",
  "content": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_metacog_capture.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- journaling raw logs
- unverified lessons without evidence

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_state`, `write_state`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_metacog_retrieve` (likely_next)
- `focusa_metacog_reflect` (likely_next)
- `focusa_workpoint_checkpoint` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_metacog_retrieve`, `focusa_metacog_reflect`, `focusa_workpoint_checkpoint`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Runbooks: `runbook:metacognition`
- Pi: `focusa_metacog_capture`; MCP: `focusa.metacog.capture`; OpenAI: `focusa_metacog_capture`.
- CLI: `focusa metacognition capture`.
- REST: `POST /v1/metacognition/capture`.
- Specification: contract registry.
- Descriptor digest: `sha256:c0ed8ccce0810df2acc78317908307a153059fc27ed1fdc4420ef1b5365da05e`.

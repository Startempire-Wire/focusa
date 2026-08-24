# `focusa_agent_artifact_delivery`

Commit verified agent artifacts with explicit operator confirmation and a durable Receipt reference. Use it when Operate the Spec 140 agent artifact delivery surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Operate the Spec 140 agent artifact delivery surface with typed scope and evidence.
- Capability family: `agent_runtime`; namespace: `focusa.agent_runtime`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `request` (required; structured): Typed delivery request.
- `confirmed` (required; boolean): Explicit operator confirmation.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_agent_artifact_delivery`.

## Output

Result envelope: `focusa.tool_result.v1`.
Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.

## Example

```json
{
  "request": {},
  "confirmed": false
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_agent_artifact_delivery.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- unverified prompt sources
- silent prompt replacement
- artifact delivery without a Receipt

## Authority, permissions, and side effects

- Scope: `{"kind":"write","route_family":"agent-runtime"}`
- Authority: `{"kind":"canonical","path":"/v1/agent-runtime/delivery/commit"}`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_agent_artifact_verify` (likely_next)
- `focusa_instruction_integrity_status` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_agent_artifact_verify`, `focusa_instruction_integrity_status`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Runbooks: `runbook:agent_runtime`
- Pi: `focusa_agent_artifact_delivery`; MCP: `focusa.agent.artifact.delivery`; OpenAI: `focusa_agent_artifact_delivery`.
- CLI: `focusa agent-runtime artifacts apply`.
- REST: `POST /v1/agent-runtime/delivery/commit`.
- Assignable: `true`; parity: `full`.
- Specification: `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`.
- Descriptor digest: `sha256:2931a5c4a5da8bea32785b126af276f32d755dd86137264c1e81269c3994358c`.

# `focusa_canonical_instruction_amendment_propose`

Record an operator-originated canonical instruction amendment proposal without activating it. Use it when Operate the Spec 140 canonical instruction amendment propose surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Operate the Spec 140 canonical instruction amendment propose surface with typed scope and evidence.
- Capability family: `agent_runtime`; namespace: `focusa.agent_runtime`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `request` (required; structured): Typed amendment proposal envelope.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_canonical_instruction_amendment_propose`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "request": {}
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_canonical_instruction_amendment_propose.md

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

- `focusa_canonical_instruction_amendment_activate` (likely_next)
- `focusa_instruction_integrity_evaluate` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_canonical_instruction_amendment_activate`, `focusa_instruction_integrity_evaluate`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:agent_runtime`
- Pi: `focusa_canonical_instruction_amendment_propose`; MCP: `focusa.canonical.instruction.amendment.propose`; OpenAI: `focusa_canonical_instruction_amendment_propose`.
- CLI: `focusa agent-runtime amendment-propose`.
- REST: `POST /v1/agent-runtime/amendments/propose`.
- Specification: `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`.
- Descriptor digest: `sha256:f053e2a4edd8fff8a16c9ee22ca2375cd2ece38cf094348eb8928f16c82671df`.

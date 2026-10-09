# `focusa_instruction_sources`

Discover bounded, registered project instruction sources with trust and authority metadata. Use it when Operate the Spec 140 instruction sources surface with typed scope and evidence. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Operate the Spec 140 instruction sources surface with typed scope and evidence.
- Capability family: `agent_runtime`; namespace: `focusa.agent_runtime`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `project_root` (required; string): See the strict descriptor schema.
- `max_source_bytes` (optional; number; min=1): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_instruction_sources`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "project_root": "/tmp/focusa-project"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_instruction_sources.md

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

- Scope: `{"kind":"read","route_family":"agent-runtime"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
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

- `focusa_instruction_conflicts` (likely_next)
- `focusa_instruction_explain` (likely_next)
- `focusa_agent_runtime_effective` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_instruction_conflicts`, `focusa_instruction_explain`, `focusa_agent_runtime_effective`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Runbooks: `runbook:agent_runtime`
- Pi: `focusa_instruction_sources`; MCP: `focusa.instruction.sources`; OpenAI: `focusa_instruction_sources`.
- CLI: `focusa agent-runtime sources`.
- REST: `GET /v1/agent-runtime/instructions/sources`.
- Specification: `docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md`.
- Descriptor digest: `sha256:d7ee2f7b8b62713efcca0611d91165c8406b00c0f3e5ba3e9ce4260a38288030`.

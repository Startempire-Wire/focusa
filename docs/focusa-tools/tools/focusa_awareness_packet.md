# `focusa_awareness_packet`

Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools, including Spec 111 preload status surfaces. Use it when Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools. Use on reload, post-compaction, tool guidance, warning, or UIAI bridge surfaces. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools. Use on reload, post-compaction, tool guidance, warning, or UIAI bridge surfaces.
- Capability family: `awareness`; namespace: `focusa.awareness`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `surface` (optional; string | string | string | string | string | string | string | string | string): Awareness surface (default: reload).
- `mode` (optional; string | string | string | string): Awareness rendering mode (default: standard).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_awareness_packet`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_awareness_packet.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- treating awareness as canonical authority
- ignoring suppressed lines when debugging degraded awareness

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"awareness:read"}`
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

- `focusa_workpoint_resume` (likely_next)
- `focusa_trajectory_view` (likely_next)
- `focusa_tool_doctor` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_tool_doctor`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Runbooks: `runbook:awareness`
- Pi: `focusa_awareness_packet`; MCP: `focusa.awareness.packet`; OpenAI: `focusa_awareness_packet`.
- CLI: none.
- REST: `GET /v1/awareness/packet`, `GET /v1/awareness/packet/{surface}`.
- Specification: contract registry.
- Descriptor digest: `sha256:d17e3cbb384807bc9a10692eb7290fbc806f692b63538fc39067fcad9e3284a2`.

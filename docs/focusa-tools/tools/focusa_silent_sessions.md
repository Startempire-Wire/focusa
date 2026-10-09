# `focusa_silent_sessions`

Daemon-native Spec133 Silent Session client for status, observation, steering, controls, config, receipts, capabilities, and legacy action compatibility. Creation, start, and resume require exact daemon North Star admission; pause, interrupt, cancel, and observation remain available for recovery. Process-control failures return failure_class=process_control_failed with receipt-backed recovery. Use it when Thin daemon-native Spec133 API client for exact session/run status, bounded observation, steering, controls, config, receipts, capabilities, and legacy action compatibility; process-control failures return failure_class=process_control_failed with receipt-backed recovery. Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Thin daemon-native Spec133 API client for exact session/run status, bounded observation, steering, controls, config, receipts, capabilities, and legacy action compatibility; process-control failures return failure_class=process_control_failed with receipt-backed recovery.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string | string | string | string | string | string | string | string | string | string | string | string | string): See the strict descriptor schema.
- `session_id` (optional; string): Exact durable Silent Session id.
- `session_name` (optional; string): Legacy alias for exact session_id; no legacy name normalization.
- `run_id` (optional; string): Exact current run id.
- `generation` (optional; integer; min=1): Exact current run generation.
- `approval_id` (optional; string): Durable daemon approval id for mutations.
- `idempotency_key` (optional; string): Mutation replay key.
- `text` (optional; string): Input or steering text.
- `command` (optional; string): Legacy alias for text; never executed as a shell command.
- `cursor` (optional; string): Opaque event/output cursor.
- `channel` (optional; string): Output channel; defaults to stdout.
- `config` (optional; structured): Typed preflight/config request body.
- `approved` (optional; boolean): Legacy compatibility hint only; never grants authority.
- `force` (optional; boolean): Legacy compatibility hint only; daemon policy decides force.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_silent_sessions`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_silent_sessions.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"canonical","path":"daemon:/v1/silent-sessions"}`
- Side effects: `daemon_api_control`, `daemon_api_control`
- Advisory source hints (not dispatch/replay guarantees): read-only `false`; destructive `false`; idempotent `true`; open-world `false`. Verify selected action/schema/policy separately.
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

- `focusa_work_loop_status` (likely_next)
- `focusa_work_loop_checkpoint` (likely_next)
- `focusa_resource_mode` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_work_loop_status`, `focusa_work_loop_checkpoint`, `focusa_resource_mode`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`, `skill:focusa-silent-sessions`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_silent_sessions`; MCP: `focusa.silent.sessions`; OpenAI: `focusa_silent_sessions`.
- CLI: `focusa silent`.
- REST: `GET /v1/silent-sessions`, `POST /v1/silent-sessions/preflight`, `GET /v1/silent-sessions/{session_id}`, `GET /v1/silent-sessions/{session_id}/output`, `POST /v1/silent-sessions/{session_id}/input`, `POST /v1/silent-sessions/{session_id}/start`, `POST /v1/silent-sessions/{session_id}/pause`, `POST /v1/silent-sessions/{session_id}/resume`, `POST /v1/silent-sessions/{session_id}/interrupt`, `POST /v1/silent-sessions/{session_id}/cancel`, `POST /v1/silent-sessions/{session_id}/restart`, `POST /v1/silent-sessions/{session_id}/config/preview`, `GET /v1/silent-sessions/{session_id}/receipts`, `GET /v1/silent-sessions/capabilities`.
- Specification: contract registry.
- Descriptor digest: `sha256:94bf1ba7457810732e7bc32cd2f2e2547b0ed2dd673ebdd69550d04c99647b91`.

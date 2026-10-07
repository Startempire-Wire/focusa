# `focusa_device_pair_qr`

Mac menubar OAuth-like device pairing with QR handoff (Spec focusa-ui0y, Mode B). Calls /v1/device/pair/start and returns pair_url + pair_url_qr_payload prominently so the Mac menubar can render a QR the operator's phone can scan. Use it when Mac menubar OAuth-like device pairing with QR handoff (focusa-ui0y, Mode B). Same as pair_start but surfaces pair_url for QR rendering (Telegram/Discord-style). Source capability metadata is not installed support or action admission; discover the exact active schema, scope and selected operation. Likely-next capabilities are advisory, not an execution sequence. Shared journey: docs/agent/02-focusa-cohesive-project-flow.md.

## When to use

- Mac menubar OAuth-like device pairing with QR handoff (focusa-ui0y, Mode B). Same as pair_start but surfaces pair_url for QR rendering (Telegram/Discord-style).
- Capability family: `session_transfer`; namespace: `focusa.session_transfer`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `device_name` (optional; string): Human-readable device name (e.g. 'operator-macbook-pro'). Defaults to 'operator-device'.
- `platform` (optional; string): Platform string. Default: 'macos'.
- `daemon_base_url` (optional; string): Daemon base URL the device will reconnect to. Default: '<http://127.0.0.1:8787>'.
- `scopes` (optional; array): OAuth-like scopes. Default: ['read', 'write'].

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_device_pair_qr`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_device_pair_qr.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- raw localStorage as canonical
- raw URL paste without a saved pair

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_device_pair`, `write_device_pair`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `true`.
- Confirmation required: `null`; preview supported: `null`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

Complete journey: `docs/agent/02-focusa-cohesive-project-flow.md` (Bootstrap/Genesis → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance). Reuse valid state; select capabilities by condition rather than running a tool list as a script.
A rejected operation calls for exact-cause diagnosis and supported scoped recovery, not automatic mission abandonment; preserve genuine authority boundaries and resume only after verification.
Shared boundaries and conditional crosswalk: [AUTHORITY_MODEL.md](../../current/AUTHORITY_MODEL.md), [GOLDEN_WORKFLOW.md](../../current/GOLDEN_WORKFLOW.md), and [AGENT_ADAPTER_CONTRACT.md](../../current/AGENT_ADAPTER_CONTRACT.md).

- `focusa_device_pair_status` (likely_next)
- `focusa_device_pair_list` (likely_next)

Prerequisites: resolve exact ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey through installed bindings; project_root/continuity_id are lookup inputs; verify current instruction, lifecycle/operation/frontier and required grants before effects.
Likely next: `focusa_device_pair_status`, `focusa_device_pair_list`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:session_transfer`
- Pi: `focusa_device_pair_qr`; MCP: `focusa.device.pair.qr`; OpenAI: `focusa_device_pair_qr`.
- CLI: `focusa device pair-qr`.
- REST: `POST /v1/device/pair/start`.
- Specification: `docs/53-focusa-device-pairing-spec.md`.
- Descriptor digest: `sha256:bf92344ac751319d1f6c6552c66792300aca171e45ee6bc79ee02b6590fba544`.

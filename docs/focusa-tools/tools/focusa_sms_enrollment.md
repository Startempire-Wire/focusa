# `focusa_sms_enrollment`

Read value-free customer-owned connector enrollment status. Use it when Read value-free customer-owned connector enrollment status. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read value-free customer-owned connector enrollment status.
- Capability family: `communications`; namespace: `focusa.communications`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- No arguments.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_sms_enrollment`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_sms_enrollment.md

## Operator alignment

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## Anti-examples

- assuming OTP authority grants inbox access
- placing OTP/message/credential values in logs, argv, receipts, or model context

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"sms:enrollment"}`
- Authority: `{"kind":"canonical","path":"/v1/sms/enrollment"}`
- Side effects: `read_value_free_enrollment`, `read_value_free_enrollment`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_sms_health` (likely_next)
- `focusa_sms_threads` (likely_next)
- `focusa_sms_events` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_sms_health`, `focusa_sms_threads`, `focusa_sms_events`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:communications`
- Pi: `focusa_sms_enrollment`; MCP: `focusa.sms.enrollment`; OpenAI: `focusa_sms_enrollment`.
- CLI: `focusa sms enrollment`.
- REST: `GET /v1/sms/enrollment`.
- Specification: `docs/156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md`.
- Descriptor digest: `sha256:9eb11001e22713cfae9bce4eb3b4eb719138a10bc729c0fe0b6bfedd88f191c0`.

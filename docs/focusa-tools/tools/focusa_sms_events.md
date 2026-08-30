# `focusa_sms_events`

Read bounded value-free broker audit events. Use it when Read bounded value-free communications audit events. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Read bounded value-free communications audit events.
- Capability family: `communications`; namespace: `focusa.communications`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `grant_id` (required; string): See the strict descriptor schema.
- `consumer_ref` (required; string): See the strict descriptor schema.
- `since` (optional; string): See the strict descriptor schema.
- `limit` (optional; integer; min=1, max=500, default=100): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_sms_events`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "grant_id": "example",
  "consumer_ref": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_sms_events.md

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

- Scope: `{"kind":"read","route_family":"sms:events"}`
- Authority: `{"kind":"canonical","path":"/v1/sms/events"}`
- Side effects: `read_value_free_audit_events`, `read_value_free_audit_events`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_sms_health` (likely_next)
- `focusa_sms_checkpoint` (likely_next)
- `focusa_sms_revoke` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_sms_health`, `focusa_sms_checkpoint`, `focusa_sms_revoke`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:communications`
- Pi: `focusa_sms_events`; MCP: `focusa.sms.events`; OpenAI: `focusa_sms_events`.
- CLI: `focusa sms events`.
- REST: `GET /v1/sms/events`.
- Specification: `docs/156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md`.
- Descriptor digest: `sha256:7f3809c27906e4a722f00bbd982180d1f7517ba260e40cf7076a426a1b7ca159`.

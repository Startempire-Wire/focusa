# `focusa_sms_otp_challenge`

Register an exact provider/target challenge before requesting OTP delivery. Returns a handle, never an OTP. Use it when Register an exact provider and target challenge before OTP delivery. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Register an exact provider and target challenge before OTP delivery.
- Capability family: `communications`; namespace: `focusa.communications`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `provider` (required; string): See the strict descriptor schema.
- `target_handle` (required; string): See the strict descriptor schema.
- `consumer_ref` (required; string): See the strict descriptor schema.
- `grant_id` (required; string): See the strict descriptor schema.
- `ttl_seconds` (optional; integer; min=30, max=600, default=300): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_sms_otp_challenge`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "provider": "example",
  "target_handle": "example",
  "consumer_ref": "example",
  "grant_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_sms_otp_challenge.md

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

- Scope: `{"kind":"write","route_family":"sms:otp_challenge"}`
- Authority: `{"kind":"canonical","path":"/v1/sms/otp/challenges"}`
- Side effects: `bounded_challenge_registration`, `bounded_challenge_registration`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_sms_otp_inject` (likely_next)
- `focusa_sms_events` (likely_next)
- `focusa_sms_health` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_sms_otp_inject`, `focusa_sms_events`, `focusa_sms_health`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:communications`
- Pi: `focusa_sms_otp_challenge`; MCP: `focusa.sms.otp.challenge`; OpenAI: `focusa_sms_otp_challenge`.
- CLI: `focusa sms otp-challenge`.
- REST: `POST /v1/sms/otp/challenges`.
- Specification: `docs/156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md`.
- Descriptor digest: `sha256:98d64356a5183ce2eb7f24cbdc4ce40e41224da62bfb3ca7ff747f80fbbe6ddd`.

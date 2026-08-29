# `focusa_sms_otp_inject`

Inject one eligible OTP into its exact bound target. The OTP value never enters model context or tool output. Use it when Inject one eligible OTP into its exact bound target without exposing the value to model context. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Inject one eligible OTP into its exact bound target without exposing the value to model context.
- Capability family: `communications`; namespace: `focusa.communications`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `challenge_handle` (required; string): See the strict descriptor schema.
- `target_handle` (required; string): See the strict descriptor schema.
- `consumer_ref` (required; string): See the strict descriptor schema.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_sms_otp_inject`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "challenge_handle": "example",
  "target_handle": "example",
  "consumer_ref": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_sms_otp_inject.md

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

- Scope: `{"kind":"control","route_family":"sms:inject_otp"}`
- Authority: `{"kind":"canonical","path":"/v1/sms/otp/inject"}`
- Side effects: `single_use_secret_injection`, `single_use_secret_injection`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_sms_events` (likely_next)
- `focusa_sms_health` (likely_next)
- `focusa_sms_revoke` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_sms_events`, `focusa_sms_health`, `focusa_sms_revoke`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:communications`
- Pi: `focusa_sms_otp_inject`; MCP: `focusa.sms.otp.inject`; OpenAI: `focusa_sms_otp_inject`.
- CLI: `focusa sms otp-inject`.
- REST: `POST /v1/sms/otp/inject`.
- Specification: `docs/156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md`.
- Descriptor digest: `sha256:80bdc3da5111da2511ec75d5731e2ec3f5158045dcd574e76416f230d38a27ab`.

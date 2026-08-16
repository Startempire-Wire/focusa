---
name: focusa-security-auth-licensing
description: "Use for permissions, mutation confirmation, bearer/device pairing, revocation, license activation, secrets, and operator approval boundaries."
---

# Focusa Security Auth Licensing

Use for permissions, mutation confirmation, bearer/device pairing, revocation, license activation, secrets, and operator approval boundaries.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-security-auth-licensing-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- auth failure
- device pairing
- permission scope
- license
- secret-bearing operation

## Non-trigger examples

- exposing secrets in transcript
- trusting remote annotations as authority

## Required sequence

1. `focusa_device_pair_start`
2. `focusa_device_pair_status`
3. `focusa_device_pair_list`
4. `focusa_device_pair_revoke`
5. `focusa_tool_describe`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_tool_doctor`
- `focusa_device_pair_status`
- `focusa_project_verify`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

Least-privilege scope, explicit approval, revocation/recovery, and secret-safe evidence are verified.

Stable evidence or receipt refs must support any completion claim.

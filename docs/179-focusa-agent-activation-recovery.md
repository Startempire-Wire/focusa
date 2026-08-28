# Focusa Agent Activation Recovery

## Purpose

An already-paid Focusa customer must be able to recover activation without
being sent back through email verification, checkout, or an opaque
`--resume` handle. The agent still remains fail-closed for a genuinely new
machine: it may not invent an email, OTP, payment confirmation, license, or
entitlement.

## Operator command

For an existing paid license key, the agent runs:

```text
focusa license activate-flow --agent --license-key <existing-key> --json
```

`--license-key` selects the authority's existing-key fast path. It sends the
key and this machine's persisted node identity to the canonical authority,
then verifies and atomically persists the returned signed lease. The key is
not printed by default and is not placed in generic agent transcripts.

The agent must not use `--email` for this path. Email starts a new pending
registration and can legitimately require human verification and payment.

## Recovery behavior

1. Load or create the local `focusa.node_identity.v1` identity.
2. Submit the existing key through `/activation/redeem`.
3. Authority verifies EDD ownership, product, node allocation, and idempotency.
4. Client verifies the authority key-set and device-bound lease.
5. Client writes the signed authority state atomically.
6. Re-running the command returns the same effective entitlement without
   consuming another seat or rewriting historical signed leases.

If the client has no key and no recoverable local registration, the agent
returns a typed human-action/error envelope. This is intentional: silent key
invention would violate Spec 152E security requirements.

## Implementation note

`run_agent_activation_command` dispatches `--license-key` to the existing
`run_redeem_fast_path` before the new-registration branch. This preserves the
single authority path and avoids a second activation implementation.

## Acceptance evidence

- Static CLI contract test verifies agent fast-path dispatch.
- Existing authority E2E verifies raw UUID binding, idempotent retry, active
  lifetime bundle entitlement, and zero synthetic fixture residue.
- Barry's public HTTPS poll verified raw UUID lease sequence 18; historical
  signed lease rows remained byte-identical.
- Release acceptance still requires canonical v0.9.185 artifacts with embedded
  production trust roots before customer-side closure.

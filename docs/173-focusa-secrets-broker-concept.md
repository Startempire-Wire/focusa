# Focusa Secrets Broker — Architecture Concept (#173)

**Status:** concept (brainstorm crystallized 2026-08-23) · **Owner vision thread:** Sir V3
**Related:** #174 (Agent Workforce Extension), #175 (Capabilities & Owner Topology)

## Purpose

A first-class Focusa subsystem that syncs with human password managers and gives
agents seamless, governed access to accounts humans use — background, invisible,
audited. Agents act as client representatives without ever handling vault
software, master passwords, or interactive 2FA.

## Design rules (operator directives)

- Zero friction: no interactive unlocks, ever. Token-driven connectors only.
- Client-local custody: runs inside the customer's Focusa install.
- Injection over distribution: prefer filling credentials into UIAI's browser
  over handing raw values to agent context. Values never enter transcripts.
- Every access = one audit ledger entry (which agent, which secret, when, why).
- Governance happens at onboarding (scope grants); not mid-task friction.

## Architecture

```
Client vault (Bitwarden / 1Password / KeePassXC / import)
      │ connector token (API key / Secrets Manager token / service account)
      ▼
crates/focusa-secrets        ← core library
  • Connector trait (token-driven pull)
  • Sync engine (timer + on-demand; freshness metadata)
  • Encrypted local cache
  • Policy engine (folder/collection → agent scopes)
  • Audit ledger (shared entry schema with #175)
      ▼
Daemon module                ← local API over existing loopback + SSE events
      ▼
Surfaces: UIAI Engine browser injection · Cockpit console · menubar/TUI status ·
          Pi extension `focusa_secrets_resolve` · Workforce role scopes (#174)
```

## Failure semantics (decided 2026-08-23)

Vault unreachable mid-task → **always queue and retry**, notify the owner,
append to the audit log. Agents never hard-fail on vault availability; work
pauses cleanly with a visible pending-secret state.

## Connection/auth pattern

Reuse the menubar desktop bridge precedent: pairing token + device id +
nonce-callback handshake with explicit protocol version string. The broker's
vault connection reuses the same pairing UX family so all Focusa pairings feel
identical across surfaces.

## Security posture

- Cache encrypted at rest; keys bound to install identity.
- Blast radius limited to customer's own install (no shared cloud).
- Ledger exportable — client-facing trust artifact.
- Anti-hijack alignment: secret values flow through memory channels only,
  consistent with pi hot-path guardrails already in turns.ts.

## Open questions

- Which password managers in v1 connector set beyond Bitwarden?
- TOTP handling policy (serve codes? display-only?).
- Freshness SLA per connector (sync interval guarantees).

## Elon-rule trim pass (2026-08-23)

- V1 broker = policy + audit + resolve, proxying live vault calls through the
  connector. The dedicated encrypted cache layer is DELETED: connector CLIs
  (bw/rbw) already own sync and local caching; a second cache duplicates that
  responsibility. Reintroduce only if measured latency/offline need proves it.
- Failure semantics unchanged: queue-and-retry + notify + audit.

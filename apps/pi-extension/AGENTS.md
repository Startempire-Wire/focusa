# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Agent-KB API Default Reference

Inherit the workspace rule: use `agent-kb-api` first for KH/OVH/operator policy, verify freshness, and use local Agent KB files only as a read-only fallback.

## Agent communications + GitHub 2FA adapter contract

- The Pi extension is a thin client to the daemon-owned communications/credential broker. Its release-critical use case is completing an active `github.com` login with a renewable SMS OTP; it must never expose ambient messages, browser cookies, paired-profile state, or Google/Apple credentials.
- GitHub MFA defaults to broker-side SMS `inject_otp`. Connector failure must surface and route to private repair/re-pairing; the extension must not silently switch to GitHub Mobile, passkey, authenticator app, or any other MFA method. Alternate renewable methods require explicit Sir V3 direction.
- Tool calls must require scoped challenge/provider fields and return canonical `tool_result_v1` envelopes. Prefer broker-side one-time OTP injection; plaintext reveal requires an explicit grant. Redact values from model context, logs, receipts, screenshots, and persisted extension state.
- GitHub OTP is the first bounded tool slice. Preserve future separately granted tools for thread listing, bounded reads, sends, and events; never let an OTP capability imply general SMS access.
- Keep tools connector-neutral and capability-based so Android/Google Messages and the urgent first-class iPhone/iOS connector use identical public contracts. No Android-only fields in shared tool schemas; no assumptions about private Apple messaging APIs. Recovery-code access is never a tool capability.
- Consumer acceptance includes revocation, expiry, replay rejection, rate limiting, audit attribution, degraded/offline handling, and parity tests against real Android and iPhone connector paths.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Connected workflow and delivery

Inherit the repository `AGENTS.md` and `docs/agent/02-focusa-cohesive-project-flow.md`: exact binding, conditional initialization, linked goals/spec/tasks, current action, recovery and verified advancement. Catalog links are not executable dependency graphs. Do not inject a static product goal or treat a saved-scope packet as current-action permission.

## Landing the Plane (Session Completion)

Follow the repository's outcome/delivery contract instead of duplicating a universal Git checklist here. Apply an approved development/native reload promptly when active testing is requested, verify the actual consumer and retain evidence. Git publication is conditional on the configured delivery mechanism or explicit request; production artifacts retain the canonical signed release/install path.

Preserve other agents' changes and stashes. Do not treat a commit, push, source/schema generation or a different process's reload as installed or current-process behavioral proof. Continue ready authorized work; record exact unresolved dependencies rather than claim completeness.

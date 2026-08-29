# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Agent-KB API Default Reference

Inherit the workspace rule: use `agent-kb-api` first for KH/OVH/operator policy, verify freshness, and use local Agent KB files only as a read-only fallback.

## Agent communications + GitHub 2FA adapter contract

- The Pi extension is a thin client to the daemon-owned communications/credential broker. Its release-critical use case is completing an active `github.com` login with a renewable SMS OTP; it must never expose ambient messages, browser cookies, paired-profile state, or Google/Apple credentials.
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

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

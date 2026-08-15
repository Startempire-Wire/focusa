# Agent Instructions

## Agent-KB API Default Reference

For KH/OVH/operator policy, inherit `/root/AGENTS.md`: query `agent-kb-api` first, verify freshness, use exact document lookup after empty searches, and treat local Agent KB files as read-only fallback.

## Focusa agent docs entry point

Before broad Focusa code changes or after context loss, read `docs/agent/01-focusa-agent-docs-index.md`. It is the bounded, public-safe architecture/commands/API/Workpoint/Trajectory/private-boundary guide for agents.

## Current agent-readiness fast path

1. Verify `project_root + continuity_id` with `focusa_project_identity`/`focusa_project_verify`; a Git worktree is a typed working subpath under that authority.
2. Resume Trajectory and the canonical Workpoint before acting; transcript tails, cached aliases, and predictions do not grant authority.
3. Discover capabilities progressively: `focusa_agent_card` → `focusa_tool_search` → `focusa_tool_describe`/`focusa_tool_graph`.
4. All Focusa Pi tools must remain one-to-one across runtime registration, `docs/contracts/spec141/generated-capability-v2/pi-tools.json`, capability descriptors, and `docs/focusa-tools/tools/`.
5. Load the matching `.pi/skills/<name>/SKILL.md`, then its numbered runbook only when the workflow requires detail. Packaged copies live under `apps/pi-extension/skills/` and must be byte-identical.
6. For durable background execution, use daemon-native Silent Sessions with exact session/run/generation and approval/idempotency fields—never raw tmux or shell aliases.
7. For context pressure, preserve canonical Workpoint/Trajectory state and governed auto-rollover; do not treat transcript compaction as authority.
8. Customer lifecycle changes must prove install or repair/rerun, trusted OTA/update rollback, and uninstall with user data preserved unless purge is explicit.

Current surfaces: Mission Canvas/Work Rail and generated UI (`docs/135-series-current-manifest.md`), Silent Sessions (`docs/133-silent-sessions-final-release-proof.md`), all-tool/skill machine contracts (`docs/contracts/spec141/generated-capability-v2/`), and public onboarding (`README.md`, `docs/current/FOCUSA_FRIENDLY_ONBOARDING.md`).

## Pre-work rule: always check remote first (mandatory)

Before any durable state change (commit, push, branch switch, merge, rebase, tag), or before resuming work after a session reload, agent context drift, or gap in continuity:

1. `git fetch origin` to discover remote commits you do not yet have locally.
2. `git status` to see local uncommitted work and any rebase-incompatibility risk.
3. If you have unstaged changes and the remote has moved, **stash first**, then `git pull --rebase`, then `git stash pop`. Resolve any conflicts before continuing.
4. Only then proceed to the canonical build/deploy chain below.

Why: shipping from a stale local head duplicates or reverts remote work, and creates
phantom commits in the operator's log. The discipline is: **see the world before you change it.**

## Canonical build/deploy rule (mandatory)

**Build and deploy ONLY through the full live GitHub release pipeline.**

- Canonical command: `scripts/create-dev-release-tag.sh --base 0.9 --push`
- Required chain: `CI` → `Release` → `Deploy Live Daemon` → audit/self-heal/watchdog.
- Do **not** build release artifacts locally with `cargo build --release`.
- Do **not** deploy from `target/release` or call `install-daemon.sh --binary target/release/...`.
- Do **not** run only a partial deploy workflow as a shortcut.
- If the pipeline fails, fix the pipeline/system and push; Auto Heal + Watchdog must recover future failures.

See `docs/canonical-live-release-pipeline.md` before any build/deploy work.

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Commit message policy

Run `scripts/dev.sh hooks` after cloning or after any Beads hook reinstall.
Commit subjects must remain meaningful Conventional Commit descriptions because
GitHub changelogs and tagged release summaries use the first line. Bead IDs may
appear only below the subject as a `Beads:` body trailer; ID-only subjects are
rejected by local hooks, CI, and the release-tag gate.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, or vendor/license registry work.

Agents must never commit `.focusa-private/`, raw transcripts, runtime objects, local host paths, admin URLs, customer data, or license data.

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

## RELEASE STRATEGY & VERSIONING

- Canonical policy: `docs/release-strategy.md` — read it before any release work.
- Three lanes: **0.9.x = patch** (security/critical only), **0.10.x = minor**
  (batched features, on cadence), **>= 1.0 = major** (breaking + migration notes).
- Pre-1.0 rule: breaking changes bump MINOR (`0.10.0`), never a `0.9.x` patch.
- Before tagging, classify the range: `python3 scripts/next-version.py`
  (used by CI in `.github/workflows/release-version-policy.yml`).
- Never tag outside the canonical pipeline
  (`scripts/create-dev-release-tag.sh --base <MAJOR.MINOR> --push`).
- Issue triage: `security` + `lane:patch` for security/critical;
  `lane:minor` for features/non-critical; `lane:major` for breaking plans.

# Agent Instructions

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


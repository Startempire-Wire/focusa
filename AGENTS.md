# Agent Instructions

## P0 architecture transition — mandatory first read

Before broad Focusa changes, after context loss, or when resuming Mission Canvas/Pi/core/daemon/Desktop work, read in order:

1. `docs/agent/00-p0-transition-bootstrap.md`
2. `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
3. `docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
4. `docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`
5. `docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`
6. `docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml`

The active P0 foundation is:

- Workstream is the durable cognitive workspace;
- Thread is legacy terminology;
- Continuity is lineage inside a Workstream, not Workstream identity;
- no canonical cognitive object exists outside exact Scope + Workstream;
- the daemon-global cognitive singleton must be removed;
- Focusa Desktop becomes the primary rich Mission Canvas environment;
- Pi remains a standalone/embedded Work Surface and bounded terminal compatibility projection;
- GUI, CLI and agent tools share one semantic command graph.

Do not add new daemon-global cognitive selectors. Do not use `project_root + continuity_id` as complete permanent canonical identity. Do not continue expanding the full rich Mission Canvas inside Pi before completing the transition preservation report.

## Mission Canvas/Desktop MacBook exception

The agent refactoring the current Mission Canvas worktree on the MacBook follows a narrow exception to the normal “push before stopping” rule until operator approval:

- local commits and a preservation checkpoint are mandatory;
- do not commit or push directly to `origin/main`;
- do not push onto the existing shared Mission Canvas branches;
- do not push tags or create releases from the MacBook;
- do not force-push;
- publish only an explicitly approved dedicated review branch or patch set.

This exception prevents the transition agent from mutating shared upstream authority before preservation, review and milestone proof. It does not permit uncommitted work.

## Desktop preview/build/release rule

- Focusa Desktop is the primary application, not a side dashboard.
- Use one pinned local Rust toolchain; do not install multiple toolchains or repeatedly bootstrap Rust.
- Preview the shared SvelteKit application continuously in a browser.
- Use UIAI Engine for browser interaction, screenshots, responsive checks, console/network diagnostics and Evidence.
- Do not add Playwright or another browser authority.
- Build and open the full Tauri shell at 5%, 25%, 50%, 75% and 100% milestones.
- Do not create shipping artifacts with local `cargo build --release`.
- At 75%, after operator approval, connect from the MacBook to the approved KnownHost release host through the private approved Tailscale or direct SSH path and initiate the canonical release pipeline there.
- Do not commit private hostnames, IPs, credentials or SSH details to this public repository.

## Agent-KB API Default Reference

For KH/OVH/operator policy, inherit `/root/AGENTS.md`: query `agent-kb-api` first, verify freshness, use exact document lookup after empty searches, and treat local Agent KB files as read-only fallback.

## Focusa agent docs entry point

Before broad Focusa code changes or after context loss, read `docs/agent/00-p0-transition-bootstrap.md`, then `docs/agent/01-focusa-agent-docs-index.md`.

## Current agent-readiness fast path

1. Resolve exact `ScopeRef + WorkstreamId`; verify ProjectRootKey and exact Attachment where runtime mutation matters. A Git worktree is a typed working subpath, not authority by itself.
2. Resume the Workstream-owned tactical Trajectory and canonical Workpoint before acting; transcript tails, cached aliases, predictions, CWD and UI selection do not grant authority.
3. Discover capabilities progressively: `focusa_agent_card` → `focusa_tool_search` → `focusa_tool_describe`/`focusa_tool_graph`.
4. All Focusa Pi tools must remain one-to-one across runtime registration, `docs/contracts/spec141/generated-capability-v2/pi-tools.json`, capability descriptors, and `docs/focusa-tools/tools/`.
5. Load the matching `.pi/skills/<name>/SKILL.md`, then its numbered runbook only when the workflow requires detail. Packaged copies live under `apps/pi-extension/skills/` and must be byte-identical.
6. For durable background execution, use daemon-native Silent Sessions with exact Workstream, Attachment, session/run/generation and approval/idempotency fields—never raw tmux or shell aliases.
7. For context pressure, preserve Workstream-owned Workpoint/Trajectory state and governed auto-rollover; do not treat transcript compaction as authority.
8. Customer lifecycle changes must prove install or repair/rerun, trusted OTA/update rollback, and uninstall with user data preserved unless purge is explicit.

Current surfaces: Workstream-rooted reducer migration (`docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`), Mission Canvas/Desktop transition (`docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`), Silent Sessions (`docs/133-silent-sessions-final-release-proof.md`), all-tool/skill machine contracts (`docs/contracts/spec141/generated-capability-v2/`), and public onboarding (`README.md`, `docs/current/FOCUSA_FRIENDLY_ONBOARDING.md`).

## Pre-work rule: always check remote first (mandatory)

Before any durable state change (commit, push, branch switch, merge, rebase, tag), or before resuming work after a session reload, agent context drift, or gap in continuity:

1. `git fetch origin` to discover remote commits you do not yet have locally.
2. `git status` to see local uncommitted work and any rebase-incompatibility risk.
3. If you have unstaged changes and the remote has moved, preserve first. Do not blindly stash/rebase a divergent Mission Canvas worktree; follow the preservation checkpoint and migration-ledger process in FOCUSA-TRANSITION-001.
4. Only then proceed to the canonical build/deploy chain below.

Why: shipping from a stale local head duplicates or reverts remote work, and creates phantom commits in the operator's log. The discipline is: **see and preserve the world before you change it.**

## Canonical build/deploy rule (mandatory)

**Build and deploy ONLY through the full live GitHub release pipeline.**

- Canonical command: `scripts/create-dev-release-tag.sh --base 0.9 --push`
- Required chain: `CI` → `Release` → `Deploy Live Daemon` → audit/self-heal/watchdog.
- For the Desktop transition, initiate this command from the approved KnownHost release host at the 75% milestone after operator approval.
- Do **not** build release artifacts locally with `cargo build --release`.
- Do **not** deploy from `target/release` or call `install-daemon.sh --binary target/release/...`.
- Do **not** run only a partial deploy workflow as a shortcut.
- If the pipeline fails, fix the pipeline/system through the reviewed branch and rerun from the approved host.

See `docs/canonical-live-release-pipeline.md` before any build/deploy work.

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Commit message policy

Run `scripts/dev.sh hooks` after cloning or after any Beads hook reinstall.
Commit subjects must remain meaningful Conventional Commit descriptions because GitHub changelogs and tagged release summaries use the first line. Bead IDs may appear only below the subject as a `Beads:` body trailer; ID-only subjects are rejected by local hooks, CI, and the release-tag gate.

## Quick Reference

```bash
bd ready
bd show <id>
bd update <id> --status in_progress
bd close <id>
bd sync
```

## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, vendor/license registry work or private release-host details.

Agents must never commit `.focusa-private/`, raw transcripts, runtime objects, local host paths, admin URLs, customer data, license data, private hostnames, IP addresses or credentials.

## Landing the Plane (Session Completion)

For ordinary work, work is not complete until the approved branch is pushed. For the MacBook Mission Canvas/Desktop transition exception, local commits and checkpointing are required, but upstream publication waits for explicit operator approval.

1. File issues for remaining work.
2. Run quality gates if code changed.
3. Update issue/task status, milestone Evidence and migration ledger.
4. Follow the publication policy for the active workstream:
   - ordinary approved branch: pull/rebase, `bd sync`, push and verify;
   - MacBook transition branch before approval: keep all work locally committed and report exact commit/checkpoint; do not push shared upstream refs.
5. Clean up only safe temporary state; never delete preservation checkpoints or migration evidence prematurely.
6. Verify all intended changes are committed.
7. Hand off exact Workstream, task-graph node, milestone, Evidence, risks and next safe action.

**CRITICAL RULES:**

- Never leave intentional work uncommitted.
- Never push directly to `main` or shared Mission Canvas branches during the transition.
- Never publish tags/releases from the MacBook.
- If an approved push or release pipeline fails, resolve and retry through the governed path.

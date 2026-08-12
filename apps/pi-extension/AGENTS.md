# Agent Instructions

## P0 Mission Canvas transition — stop before implementation

Before changing `apps/pi-extension/`, Mission Canvas, Work Rail, project binding, scoped state, rehydration, compaction, session identity or Desktop handoff, read:

1. `../../docs/agent/00-p0-transition-bootstrap.md`
2. `../../docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
3. `../../docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
4. `../../docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`
5. `../../docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`
6. `../../docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml`
7. `../../docs/transitions/FOCUSA-TRANSITION-001-macbook-agent-handoff-prompt.md`

The old primary route—building the complete rich Mission Canvas inside Pi TUI—is frozen.

The required route is:

- preserve all existing work and create a local checkpoint;
- inventory unique local/branch changes and current tests;
- correct identity to exact Scope + WorkstreamId;
- extract reusable semantic logic from Pi rendering;
- keep Pi as standalone/embedded Work Surface and terminal fallback;
- build Focusa Desktop as the primary application;
- expose Desktop through the shared GUI/CLI/agent command graph;
- continuously preview the shared app in a browser and prove it through UIAI Engine;
- build the full Tauri shell at 5%, 25%, 50%, 75% and 100% gates.

Do not use `project_root + continuity_id`, CWD, current tab, latest verified project, Session alone, Thread or daemon-global active/current state as complete canonical authority.

Do not rebase, delete, mass-rename, format broadly or regenerate lockfiles before the preservation report and migration ledger exist.

## MacBook branch and release restriction

For the current Mission Canvas/Desktop refactor:

- commit locally on a dedicated transition/refactor branch;
- do not push directly to `origin/main`;
- do not push onto existing shared Mission Canvas branches;
- do not push tags or create releases from the MacBook;
- do not force-push;
- publish only an explicitly approved review branch or patch set.

Use one pinned local Rust toolchain. Do not install multiple toolchains or repeatedly bootstrap Rust. Use browser-first SvelteKit/Vite preview for ordinary iteration and UIAI Engine for browser proof. Do not use local `cargo build --release` for shipping artifacts.

At the 75% Desktop gate, after explicit operator approval, connect to the approved KnownHost release host through the private approved Tailscale or direct SSH path and initiate the canonical release cycle there. Never commit private host details to the public repository.

The first response from an agent resuming the old Mission Canvas worktree must contain the A–M handoff report and proposed 5% milestone defined in FOCUSA-TRANSITION-001.

## Agent-KB API Default Reference

Inherit the workspace rule: use `agent-kb-api` first for KH/OVH/operator policy, verify freshness, and use local Agent KB files only as a read-only fallback.

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready
bd show <id>
bd update <id> --status in_progress
bd close <id>
bd sync
```

## Landing the Plane (Session Completion)

For this transition worktree, intentional work must be locally committed and checkpointed. Upstream publication waits for explicit operator approval.

1. File issues/tasks for remaining work.
2. Run focused tests and current UIAI Engine proof.
3. Update migration ledger, task node and milestone Evidence.
4. Commit locally with a meaningful message.
5. Do not push `main`, shared Mission Canvas branches, tags or releases.
6. Keep preservation checkpoints and migration evidence until explicit retirement.
7. Hand off exact Workstream identity, local commit/checkpoint, milestone, tests, risks and next safe action.

**CRITICAL RULES:**

- Never leave intentional work uncommitted.
- Never push directly to shared upstream authority during the transition.
- Never publish a release from the MacBook.
- At an approved publication/release gate, follow the reviewed branch and KnownHost canonical pipeline requirements.

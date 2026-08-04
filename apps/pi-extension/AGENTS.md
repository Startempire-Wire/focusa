# Agent Instructions

## P0 Mission Canvas transition — stop before implementation

Before changing `apps/pi-extension/`, Mission Canvas, Work Rail, project binding, scoped state, rehydration, compaction, session identity or Desktop handoff, read:

1. `../../docs/agent/00-p0-transition-bootstrap.md`
2. `../../docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
3. `../../docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
4. `../../docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`

The old primary route—building the complete rich Mission Canvas inside Pi TUI—is frozen.

The required route is:

- preserve all existing work and create a checkpoint;
- inventory unique local/branch changes and current tests;
- correct identity to exact Scope + WorkstreamId;
- extract reusable semantic logic from Pi rendering;
- keep Pi as standalone/embedded Work Surface and terminal fallback;
- move the primary rich experience to Focusa Desktop;
- expose Desktop through the shared GUI/CLI/agent command graph.

Do not use `project_root + continuity_id`, CWD, current tab, latest verified project, Session alone, Thread or daemon-global active/current state as complete canonical authority.

Do not rebase, delete, mass-rename, format broadly or regenerate lockfiles before the preservation report and migration ledger exist.

The first response from an agent resuming the old Mission Canvas worktree must contain the A–M handoff report defined in FOCUSA-TRANSITION-001.

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

**When ending a work session**, complete all steps below. Work is not complete until `git push` succeeds.

1. File issues for remaining work.
2. Run quality gates if code changed.
3. Update issue/task status and the migration ledger.
4. Push to remote:

   ```bash
   git pull --rebase
   bd sync
   git push
   git status
   ```

5. Keep preservation checkpoints and migration evidence until explicit retirement.
6. Verify all changes are committed and pushed.
7. Hand off exact Workstream identity, task-graph node, tests, risks and next safe action.

**CRITICAL RULES:**

- Work is not complete until `git push` succeeds.
- Never stop before pushing.
- Never say “ready to push when you are.”
- If push fails, resolve and retry.

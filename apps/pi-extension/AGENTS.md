# Agent Instructions

## P0 transition notice — do not continue the old Mission Canvas route

This branch predates the Spec 158 Workstream foundation correction and the Focusa Desktop pivot.

Before changing this worktree:

```bash
git fetch origin --prune
git show origin/transition/spec158-desktop-pivot:docs/agent/00-p0-transition-bootstrap.md
git show origin/transition/spec158-desktop-pivot:docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-macbook-agent-handoff-prompt.md
```

Required immediate action:

1. preserve and locally commit the exact MacBook worktree state;
2. create the checkpoint required by FOCUSA-TRANSITION-001;
3. inventory unique commits/files/tests and produce the migration ledger;
4. stop adding rich desktop-class Pi TUI panels;
5. correct identity to exact Scope + WorkstreamId before shared extraction;
6. propose the 5% Focusa Desktop primary-app milestone.

## MacBook/upstream restriction

- local commits are required;
- do not push directly to `origin/main`;
- do not push new implementation onto this shared legacy branch;
- do not create tags/releases from the MacBook;
- do not force-push;
- publish only an explicitly approved dedicated review branch or patch set.

Use one pinned Rust toolchain. Preview the real shared SvelteKit app continuously in browser and prove it through UIAI Engine. Build/open the complete Tauri shell at 5%, 25%, 50%, 75% and 100% milestones. At 75%, after operator approval, initiate the canonical release from the approved KnownHost release host through the private approved connection path.

Never commit private hostnames, IPs, credentials or SSH details.

## Canonical correction

Workstream is the durable cognitive workspace. Thread is legacy terminology. Continuity is lineage inside a Workstream. Session/Instance are runtime metadata. Work Surface is presentation only. UI focus, CWD, last/current/latest daemon state and `project_root + continuity_id` alone are not canonical authority.

## Beads quick reference

```bash
bd onboard
bd ready
bd show <id>
bd update <id> --status in_progress
bd close <id>
bd sync
```

## Session completion for this transition

Commit all intentional work locally and report the exact checkpoint/commit, tests, migration ledger, task nodes, milestone status, blockers and next safe action. Do not publish shared upstream refs without explicit operator approval.

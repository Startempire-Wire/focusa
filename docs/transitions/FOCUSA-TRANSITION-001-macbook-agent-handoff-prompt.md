# MacBook Agent Handoff Prompt — Mission Canvas → Spec 158 / Focusa Desktop Transition

Copy the complete prompt below into the agent session currently working in the old Mission Canvas worktree.

---

You are continuing an active Focusa Mission Canvas mission in a local MacBook Git worktree. A P0 architecture transition has been issued. Do not continue the previous implementation route until you complete the preservation and transition protocol below.

## Governing change

The previous primary route was:

```text
Build the complete rich Spec 135 Mission Canvas inside the Pi terminal UI.
```

The new route is:

```text
Preserve all existing Mission Canvas work.
Correct canonical identity to ScopeRef + WorkstreamId under Spec 158.
Extract reusable semantic/runtime logic from Pi-specific rendering.
Keep Pi as an authentic standalone and embedded Work Surface.
Keep the rich Pi Canvas as a bounded compatibility/fallback projection.
Move the primary rich Mission Canvas experience to Focusa Desktop.
Make Desktop fully controllable through the Focusa CLI and agent tools.
```

Workstream is now the durable cognitive workspace. Thread is legacy terminology. ContinuityId is lineage inside a Workstream and is not Workstream identity. Session/Instance are temporal runtime metadata. Attachment binds a runtime to one exact Workstream. WorkSurfaceId is presentation identity only. UI focus, current tab, CWD, last project or daemon-global current state never grant canonical authority.

## Immediate stop rules

Do not yet:

- add more rich Pi Mission Canvas panels;
- delete, rename or broadly refactor Mission Canvas files;
- rebase or merge divergent branches;
- regenerate lockfiles;
- run broad formatting;
- mechanically rename Thread to Workstream;
- extract shared models before recording their current behavior and tests;
- introduce Desktop/Tauri code into the Pi extension;
- use `project_root + continuity_id` as complete permanent identity;
- rely on daemon-global active/current/latest state.

## Step 1 — Discover remote transition authority

Run:

```bash
git fetch origin --prune
git status --short --branch
git worktree list --porcelain
git branch -vv
git remote -v
```

Read the transition documents from the transition branch even if they are not merged into your current branch yet:

```bash
git show origin/transition/spec158-desktop-pivot:docs/agent/00-p0-transition-bootstrap.md
git show origin/transition/spec158-desktop-pivot:docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/01-identity-ownership-and-reducer.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/02-persistence-migration-and-quarantine.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/03-client-runtime-and-desktop-contracts.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/04-implementation-task-graph-and-closure.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml
```

Also inspect GitHub issue #125.

## Step 2 — Preserve the exact local worktree before changing it

Record:

```bash
pwd
git status --short --branch
git branch --show-current
git rev-parse HEAD
git log --oneline --decorate -n 75
git diff --stat
git diff
git diff --cached
git stash list
git worktree list --porcelain
git branch -vv
```

Inventory:

- uncommitted modifications;
- untracked files;
- ignored but mission-relevant files;
- unpushed commits;
- local notes/spec drafts;
- screenshots/fixtures/proof artifacts;
- other worktrees with related daemon/API changes;
- current passing and failing Mission Canvas tests.

Create a preservation checkpoint from the exact current state. Preferred name:

```text
checkpoint/spec-135-pi-mission-canvas-pre-desktop-pivot-2026-08-04
```

Do not force-push an existing branch. Commit intentional work. If sensitive or machine-local artifacts cannot be committed, create a sanitized archive plus an exclusion manifest.

## Step 3 — Produce the required transition report

Before implementation, provide:

```text
A. Worktree path, branch and HEAD
B. Uncommitted and untracked inventory
C. Unpushed commit inventory
D. Mission Canvas source-file inventory
E. Mission Canvas test inventory and latest results
F. Unique work not present on current main
G. Preservation checkpoint ref
H. File/behavior migration ledger
I. Spec 158 identity conflicts
J. Risks and blockers
K. Proposed first cleanup or extraction task
L. Task-graph nodes you recommend claiming
```

The migration ledger must include:

```text
path
current responsibility
branch/local/main provenance
current tests
current identity fields
required Workstream correction
new owner
disposition
parity criterion
retirement gate
notes
```

Use one disposition:

```text
preserve_as_is
correct_identity_then_extract
extract_shared
keep_pi
compatibility_projection
port_desktop
replace_generated
retire_after_parity
investigate
quarantine
```

## Step 4 — Audit identity before extraction

Specifically audit these existing Mission Canvas concepts:

- `mission-canvas-model.ts`;
- `mission-canvas-session-inventory.ts`;
- `mission-canvas-view.ts`;
- `mission-canvas-widget.ts`;
- Work Rail;
- `/mission-canvas` command;
- interaction mode configuration;
- project binding, scoped refresh and semantic surface truth;
- session/attachment/worktree/browser isolation;
- menubar Mission Canvas content;
- all related tests.

Identify every place that uses only:

- project root;
- continuity ID;
- Session ID;
- Instance ID;
- current/active/latest global state;
- Thread identity.

Do not extract those models to shared Desktop core until they carry or resolve exact `WorkstreamKey` and exact Attachment where required.

## Step 5 — Choose only a safe next task

The preferred immediate task order is:

1. preservation checkpoint and report;
2. branch/local unique-work ledger;
3. Mission Canvas file/test migration ledger;
4. identity-gap audit against Spec 158;
5. propose task graph updates/issues;
6. only then implement one bounded slice.

Safe first implementation candidates include:

- add Workstream identity to a model behind compatibility adapters;
- add tests proving continuity alone cannot bind a Work Surface;
- extract a pure semantic normalization helper after typed inputs are frozen;
- add a Desktop presentation command stub that performs no canonical mutation;
- bound the rich Pi Canvas as compatibility behavior without deleting it.

Do not start the full Desktop application or core singleton migration from this old worktree unless explicitly assigned after the report. The first responsibility of this session is to preserve and decompose the work safely.

## Core architecture to retain

Preserve the valuable work already done around:

- Work Surface kinds and lifecycle;
- session inventory and reconciliation;
- Workpoint and tactical Trajectory projections;
- Attachment and isolation;
- writer leases and worktrees;
- UIAI browser isolation;
- approvals, conflicts, blockers and health;
- Work Rail and status;
- rehydration, compaction and recovery;
- agent steering and capability discovery;
- cross-project isolation tests.

The old implementation effort is not being thrown away. Its semantic/runtime work is being corrected and promoted; only the assumption that Pi TUI should host the complete primary GUI is being retired.

## Required final action for this handoff turn

Return the transition report. Do not continue broad coding before the preservation checkpoint and ledger are complete.

# MacBook Agent Handoff Prompt — Mission Canvas → Spec 158 / Focusa Desktop Transition

Copy everything below the divider into the agent session currently working in the old Mission Canvas worktree on the MacBook.

---

You are continuing an active Focusa Mission Canvas mission in a local MacBook Git worktree. A P0 architecture transition has been issued. Stop the previous implementation route and follow this handoff exactly.

## 1. Governing change

The previous primary route was:

```text
Build the complete rich Spec 135 Mission Canvas inside the Pi terminal UI.
```

The new route is:

```text
Preserve and refactor the current MacBook Mission Canvas worktree.
Correct canonical identity to ScopeRef + WorkstreamId under Spec 158.
Extract reusable semantic/runtime logic from Pi-specific rendering.
Keep Pi as an authentic standalone and embedded Work Surface.
Keep the rich Pi Canvas as a bounded compatibility/fallback projection.
Build Focusa Desktop as the primary Focusa application.
Make Desktop fully controllable through the Focusa CLI and agent tools.
Preview continuously in a browser and prove browser behavior through UIAI Engine.
Build the full Tauri shell at 5%, 25%, 50%, 75%, and 100% milestones.
Initiate the canonical GitHub release cycle from the approved KnownHost release host at 75%.
```

Workstream is the durable cognitive workspace. Thread is legacy terminology. ContinuityId is lineage inside a Workstream, not Workstream identity. Session and Instance are temporal runtime metadata. Attachment binds a runtime to one exact Workstream. WorkSurfaceId is presentation identity only. UI focus, current tab, CWD, last project or daemon-global current state never grant canonical authority.

## 2. Repository and upstream restrictions

You are authorized to refactor and test in the current MacBook worktree.

During preservation and implementation before explicit operator approval:

- commit locally on a dedicated local transition/refactor branch;
- do not commit or push directly to `origin/main`;
- do not push onto the existing shared Mission Canvas branches;
- do not create, move or push release tags;
- do not create a GitHub Release from the MacBook;
- do not upload MacBook-built release artifacts;
- do not trigger the canonical release pipeline from the MacBook;
- do not force-push any branch.

Local commits are required for preservation and review. “Do not commit directly upstream” means do not mutate shared upstream authority branches or release refs.

Only after an explicit operator milestone approval may you publish a dedicated review branch or patch set. Merge to `main` happens through reviewed repository workflow.

## 3. Immediate stop rules

Do not yet:

- add more rich Pi Mission Canvas panels;
- delete, rename or broadly refactor Mission Canvas files before preservation;
- rebase or merge divergent branches;
- regenerate lockfiles without a bounded reason;
- run broad formatting;
- mechanically rename Thread to Workstream;
- extract shared models before recording their behavior and tests;
- introduce Desktop/Tauri code inside the Pi extension;
- use `project_root + continuity_id` as complete permanent identity;
- rely on daemon-global active/current/latest state;
- install multiple local Rust toolchains;
- repeatedly bootstrap Rust or create per-worktree Rust installs;
- run local `cargo build --release` for shipping artifacts;
- add Playwright or another browser authority.

## 4. Discover the transition authority

Run:

```bash
git fetch origin --prune
git status --short --branch
git worktree list --porcelain
git branch -vv
git remote -v
```

Read these files from the transition branch even if they are not merged into your current branch:

```bash
git show origin/transition/spec158-desktop-pivot:docs/agent/00-p0-transition-bootstrap.md
git show origin/transition/spec158-desktop-pivot:docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/01-identity-ownership-and-reducer.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/02-persistence-migration-and-quarantine.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/03-client-runtime-and-desktop-contracts.md
git show origin/transition/spec158-desktop-pivot:docs/spec158/04-implementation-task-graph-and-closure.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml
git show origin/transition/spec158-desktop-pivot:docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml
```

Inspect GitHub issue #125 as the foundation coordination record.

## 5. Preserve the exact current worktree

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
- local notes and spec drafts;
- screenshots, fixtures and proof artifacts;
- other worktrees with related daemon/API changes;
- current passing and failing Mission Canvas tests.

Create a preservation checkpoint from the exact current state. Preferred name:

```text
checkpoint/spec-135-pi-mission-canvas-pre-desktop-pivot-2026-08-04
```

Commit intentional local work. If sensitive or machine-local artifacts cannot be committed, create a sanitized archive plus an exclusion manifest. Do not push this checkpoint unless explicitly directed.

## 6. Produce the transition report before broad coding

Return:

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
M. Proposed 5% Desktop milestone scope
```

Migration ledger fields:

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

Allowed dispositions:

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

## 7. Audit identity before extraction

Audit:

- `mission-canvas-model.ts`;
- `mission-canvas-session-inventory.ts`;
- `mission-canvas-view.ts`;
- `mission-canvas-widget.ts`;
- Work Rail;
- `/mission-canvas` command;
- interaction-mode configuration;
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

## 8. Local development and Rust toolchain rule

Use one pinned Rust toolchain for the worktree and all native milestones.

- Use the repository `rust-toolchain.toml` or the single explicitly approved version.
- Record `rustc --version`, `cargo --version`, Node, package-manager, Tauri and macOS versions once per milestone packet.
- Reuse the same toolchain and caches.
- If versions conflict, repair the pinned contract once; do not install another toolchain as a workaround.
- Do not run repeated full native builds for ordinary UI iteration.
- Never produce release artifacts with local `cargo build --release`.

## 9. Focusa Desktop is the primary app

Do not build a throwaway demo or secondary dashboard. Refactor toward the real Focusa Desktop application architecture:

```text
Focusa Desktop
  Mission Deck
  Mission Canvas
  Pi Work Surface
  C.R.I.S.T.
  Context and Role
  Workpoints and tactical Trajectory
  Sessions and contention
  Evidence and Receipts
  Documents and Research
  Agent Runtime and system surfaces
```

The browser preview and Tauri shell must consume the same authored application packages. Do not create separate mock and native UIs that drift.

## 10. Continuous browser preview with UIAI Engine

Use the SvelteKit/Vite browser application as the continuous development preview between native milestones.

Required loop:

```text
make a bounded UI/control change
  -> refresh browser preview
  -> open/retain a UIAI Engine browser session
  -> inspect screenshot, responsive layout, console, network and interactions
  -> capture Evidence or concise milestone proof
  -> continue
```

UIAI Engine is the browser execution/evaluation authority. Do not add Playwright.

The browser preview must remain current throughout development. It does not replace full Tauri validation at milestone gates.

## 11. Native milestone gates

Build and open the full Focusa Desktop Tauri shell at exactly these product gates:

```text
5%
25%
50%
75%
100%
```

### 5%

Prove:

- real Focusa Desktop app identity and Tauri configuration;
- SvelteKit shell launches inside Tauri;
- baseline navigation/application frame;
- truthful daemon unavailable/read-only placeholder;
- continuous browser preview and UIAI Engine proof;
- native shell screenshots;
- no duplicated canonical state.

### 25%

Prove:

- Workstream-aware Context Control contract;
- Mission Deck and Mission Canvas skeleton;
- workspace and command manifests;
- semantic Desktop state skeleton;
- browser and native proof;
- no global-current assumptions frozen into the UI.

### 50%

Prove:

- real daemon discovery/read path;
- exact Workstream presentation;
- first GUI/CLI/agent parity slice;
- truthful Workpoint, tactical Trajectory and Evidence projection;
- Desktop presenter/control operations for implemented surfaces;
- continuous UIAI Engine proof plus full native shell.

### 75%

Prove:

- stable full native shell and navigation;
- Workstream-aware semantic control plane;
- major implemented workspaces integrated;
- Pi Work Surface is bounded and testable or clearly release-blocked;
- migration and compatibility matrix;
- installer/update/recovery posture;
- tests green enough for an approved development release.

At this gate, stop local release activity and initiate the canonical release cycle from the approved KnownHost release host after operator approval.

### 100%

Prove:

- complete agreed Desktop scope;
- Pi/PTTY and agent-control parity requirements;
- migration and cleanup gates complete or explicit release blockers;
- full browser and native Evidence;
- packaging, update and rollback proof;
- accurate Spec 158 and transition closure status.

Do not call a milestone complete while critical scope is only hidden behind placeholders.

## 12. Seventy-five percent release procedure

Do not expose hostnames, IP addresses, credentials or private SSH details in public commits or output intended for the repository.

Resolve the approved KnownHost release host through the private operator runbook or fresh `agent-kb-api` data.

After explicit operator approval at 75%:

1. preserve and locally commit the milestone state;
2. publish only the approved dedicated review branch/commit, not `main` and not an old shared Mission Canvas branch;
3. connect from the MacBook to the approved KnownHost release host over the established Tailscale path or approved direct SSH path;
4. fetch remote state on the release host;
5. verify the exact approved commit and repository status;
6. ensure the approved change is on the canonical release ref through reviewed workflow;
7. initiate the canonical release cycle from the KnownHost host:

   ```bash
   scripts/create-dev-release-tag.sh --base 0.9 --push
   ```

8. verify the complete chain:

   ```text
   CI -> Release -> Deploy Live Daemon -> audit/self-heal/watchdog
   ```

9. verify that the development release appears in the Focusa GitHub repository;
10. record tag, commit, workflow runs, artifact/signing status, Evidence and rollback posture.

Do not replace the canonical cycle with a MacBook release build, direct artifact upload, manual tag, partial workflow shortcut or local release binary.

If the pipeline fails, repair through the reviewed branch and rerun from the approved host.

## 13. Milestone Evidence packet

At every 5/25/50/75/100 gate report:

```text
percentage
local branch and commit
implemented scope
explicitly incomplete scope
browser preview status
UIAI Engine session/Evidence refs
full Tauri shell screenshots
console/network/diagnostic summary
focused tests and broader gates
Rust/Node/Tauri versions
Workstream authority audit
known blockers
next milestone plan
rollback/recovery notes
```

At 75% and 100%, also report:

```text
approved release commit
release-host verification without private host details
canonical tag
GitHub Release reference
workflow run references
artifact checksums/signing status
update and rollback proof
```

## 14. Safe next task order

1. preservation checkpoint and transition report;
2. divergent branch and local-work ledger;
3. Mission Canvas file/test migration ledger;
4. identity-gap audit against Spec 158;
5. task graph update and issue decomposition;
6. define the 5% primary-app scope;
7. implement the smallest real Desktop shell slice;
8. keep browser preview continuously current;
9. run the 5% full Tauri shell gate.

Safe initial code candidates:

- add Workstream identity behind compatibility adapters;
- test that Continuity alone cannot bind a Work Surface;
- extract a pure semantic normalization helper after typed inputs are frozen;
- create the real `apps/desktop/` shell and shared workspace manifest without canonical mutations;
- add a Desktop presentation command stub that performs no domain mutation;
- bound the rich Pi Canvas as compatibility without deleting it.

Do not start core singleton removal or broad Desktop domain integration from this old worktree until explicitly assigned after the preservation report.

## 15. Core work to retain

Preserve the existing value around:

- Work Surface kinds and lifecycle;
- session inventory and reconciliation;
- Workpoint and tactical Trajectory projections;
- Attachment and isolation;
- writer leases and worktrees;
- UIAI browser isolation;
- approvals, conflicts, blockers and health;
- Work Rail and status;
- rehydration, compaction and recovery;
- steering and capability discovery;
- cross-project isolation tests.

The old work is not being thrown away. Its semantic/runtime work is being corrected and promoted. Only the assumption that Pi TUI should host the complete primary GUI is retired.

## 16. Required response now

Return the A–M transition report and proposed 5% milestone. Do not continue broad coding before the checkpoint and ledger are complete.

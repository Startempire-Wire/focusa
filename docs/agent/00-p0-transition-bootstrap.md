# P0 Agent Bootstrap — Spec 158 and Mission Canvas Desktop Transition

**Status:** mandatory before broad Focusa work  
**Effective:** 2026-08-04  
**Coordination:** GitHub issue #125  
**Normative sources:**

1. `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
2. `docs/spec158/01-identity-ownership-and-reducer.md`
3. `docs/spec158/02-persistence-migration-and-quarantine.md`
4. `docs/spec158/03-client-runtime-and-desktop-contracts.md`
5. `docs/spec158/04-implementation-task-graph-and-closure.md`
6. `docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
7. `docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`
8. `docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`
9. `docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml`
10. `docs/135-series-current-manifest.md`, as amended by the transition above

---

## Stop line

Before continuing Mission Canvas, Pi-extension, reducer, daemon, Workpoint, Trajectory, Work Loop, Silent Session, Context, Evidence, Desktop, menubar, Focusa.work, or UIAI integration work:

1. fetch remote state;
2. preserve uncommitted and unpushed work;
3. read the documents above;
4. identify the exact `ScopeRef + WorkstreamId` affected;
5. classify the work against the transition task graph;
6. do not continue the obsolete full-rich-GUI-inside-Pi workroute.

The pivot is:

```text
OLD
  complete rich Mission Canvas hosted inside Pi terminal UI

NEW
  Workstream-rooted Focusa reducer and daemon
  + Focusa Desktop as the primary rich Focusa application
  + Pi as authentic embedded/standalone Work Surface
  + bounded terminal compatibility projection
  + GUI/CLI/agent parity through one semantic command graph
  + continuous browser preview proven through UIAI Engine
  + full Tauri shell gates at 5/25/50/75/100 percent
```

---

## MacBook worktree and upstream rule

The current Mission Canvas agent refactors and tests from the existing MacBook worktree.

Before explicit operator approval:

- local commits on a dedicated transition/refactor branch are required;
- no direct push or commit to `origin/main`;
- no push to existing shared Mission Canvas branches;
- no tag or GitHub Release creation;
- no MacBook release build or artifact upload;
- no force push.

Only an approved review branch/commit may leave the laptop. At the 75% Desktop gate, the canonical release cycle is initiated from the approved KnownHost release host, reached through the private approved Tailscale or direct SSH path. Private host details never enter the public repository.

---

## Correct canonical identity

The durable cognitive owner is **Workstream**.

```text
ScopeRef / ProjectRootKey
  -> WorkstreamId
    -> ContinuityId
      -> AttachmentKey
        -> SessionId / InstanceId
          -> runtime object
            -> WorkSurfaceId
```

Rules:

- `WorkstreamId` is durable cognitive workspace identity.
- `ContinuityId` is continuation lineage inside a Workstream; it is not Workstream identity.
- Session and Instance are temporal runtime metadata; they do not own cognition.
- Attachment binds runtime identities to one exact Workstream.
- Work Surface is presentation identity and does not grant mutation authority.
- Thread is legacy/historical terminology only.
- CWD, current tab, active pane, last project, latest trajectory, remembered daemon state, or `project_root + continuity_id` alone must not select canonical cognition.

Any older agent documentation that says `project_root + continuity_id` is the complete canonical authority is superseded by Spec 158.

---

## Core reducer requirement

Focusa keeps one canonical reduction law, but canonical state is partitioned by Workstream.

```text
Models, tools, agents, UIs and adapters propose.
The Focusa reducer canonizes meaning within one exact Workstream partition.
```

The daemon may serve many Projects and Workstreams. It must not own one daemon-global cognitive singleton containing a global Focus Stack, Workpoint, Trajectory, Work Loop, current Thread, current Instance, or global current project.

Daemon-global state is permitted only for infrastructure and explicitly non-cognitive aggregates.

---

## Mission Canvas transition requirement

Do not discard existing Pi Mission Canvas work.

Preserve and migrate:

- Work Surface identity and lifecycle;
- session inventory and reconciliation;
- Project/Workstream/Workpoint binding;
- Attachment and isolation semantics;
- writer leases, worktree identity and browser isolation;
- approvals, conflicts, blockers and health;
- Work Rail and status projections;
- rehydration, compaction, steering and recovery;
- tests proving no cross-project or cross-Workstream contamination.

Change:

- add durable `WorkstreamId` wherever current models use only project root or continuity;
- extract semantic projection logic from Pi TUI classes;
- stop expanding desktop-class panels inside Pi;
- retain Pi Mission Canvas as a bounded compatibility/fallback projection;
- move the primary rich experience to Focusa Desktop.

---

## Desktop development and proof rule

- Focusa Desktop is the primary app, not a side dashboard.
- Use one pinned local Rust toolchain; do not install multiple toolchains or repeatedly bootstrap Rust.
- Use the SvelteKit/Vite browser application for continuous preview between native gates.
- Use UIAI Engine for browser screenshots, responsive checks, console/network diagnostics, interactions and Evidence.
- Do not add Playwright or another browser authority.
- Build and open the complete Tauri shell at 5%, 25%, 50%, 75% and 100% milestones.
- Do not produce local release artifacts with `cargo build --release`.
- At 75%, after explicit approval, initiate `scripts/create-dev-release-tag.sh --base 0.9 --push` from the approved KnownHost release host and verify the full canonical workflow chain.

---

## Focusa Desktop invariant

Focusa Desktop is a projection and command client over Workstream-partitioned daemon authority.

It must be fully controllable through the Focusa CLI and agent tools using stable identifiers:

```text
workspace_id
subsection_id
view_id
object_ref
work_surface_id
command_id
layout_id
operation_id
receipt_ref
```

The agent must not need coordinate clicks, selectors, OCR, or button-label matching.

Presentation actions and domain actions remain separate. Visual focus never changes canonical Workstream authority.

---

## Required first response from an agent resuming old Mission Canvas work

Before implementation, report:

```text
A. worktree path, branch and HEAD
B. uncommitted and untracked inventory
C. unpushed commit inventory
D. current Mission Canvas file/test inventory
E. unique work not present on main
F. preservation checkpoint ref
G. migration ledger: preserve/extract/keep-Pi/compatibility/retire/investigate
H. Spec 158 identity conflicts
I. proposed first extraction or cleanup task
J. task-graph nodes to claim
K. proposed 5% Desktop milestone scope
```

No broad rebase, merge, deletion, rename, lockfile regeneration, formatting sweep, upstream push or release action is allowed before this report and checkpoint exist.

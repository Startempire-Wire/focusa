# FOCUSA-TRANSITION-001 — Version 2
## Spec 135 Mission Canvas → Focusa Desktop Pivot, Spec 158 Foundation Alignment, and Worktree Handoff

**Status:** immediate transition directive and architecture freeze  
**Version:** 2.0  
**Date:** 2026-08-04  
**Audience:** the agent/operator currently implementing Spec 135 Mission Canvas in the Focusa Pi-extension worktree, plus agents continuing Focusa Desktop, Focusa.work, core reducer, Pi integration, generated UI, menubar or UIAI interoperability  
**Upstream foundational authority:** Spec 158 — Workstream-Rooted Cognitive Runtime, Canonical State Partitioning, and Foundation Migration  
**Coordination issue:** `Startempire-Wire/focusa#125`  
**Primary repositories:** `Startempire-Wire/focusa`, `WPUIAI/uiai-engine`

---

# 0. Stop line: read before continuing the active mission

This is a mid-mission architecture pivot plus a foundational authority correction.

It is not permission to discard existing Mission Canvas work, rewrite it from memory, merge divergent branches blindly, or build Focusa Desktop on top of the daemon-global cognition model that Spec 158 is replacing.

The active workroute changes from:

```text
Build the complete Spec 135 Mission Canvas as a rich Pi terminal GUI
```

to:

```text
Preserve the existing Mission Canvas work.
Correct every authority-bearing model to use exact Workstream identity.
Extract reusable semantic/runtime logic from Pi-specific rendering.
Retain Pi as a first-class interactive Work Surface and terminal fallback.
Build Focusa Desktop as the primary rich local Mission Canvas environment.
Make Desktop fully controllable through the Focusa CLI and agent tools.
Carry the portable workspace architecture into Focusa.work.
```

No existing Mission Canvas file, test, branch, local commit, uncommitted change or worktree-only artifact may be deleted or overwritten until the preservation procedure and migration ledger in this document are complete.

---

# 1. Authority precedence

Implementation applies these decisions in order:

```text
Spec 158 Workstream-rooted core foundation
  -> Spec 135 Mission Canvas and Work Surface semantics
    -> FOCUSA-TRANSITION-001 v2 Desktop pivot and worktree migration
      -> Desktop implementation specs and contracts
```

This transition does not replace Spec 158.

Focusa Desktop work may proceed in preservation, shell extraction, contract design and non-authoritative presentation layers while Spec 158 is being implemented. It may not cement the obsolete global cognitive aggregate into new Desktop contracts.

---

# 2. Frozen product decisions

## 2.1 Focusa Desktop

Focusa SHALL gain a full Tauri/Svelte Desktop application.

```text
Focusa Desktop
  = primary rich local Focusa environment
  = Mission Deck + Mission Canvas + Pi + C.R.I.S.T. + Evidence + Work Surfaces
```

Focusa Desktop SHALL use:

- Tauri v2;
- SvelteKit 2 and Svelte 5;
- generated Focusa API clients and schemas;
- A2UI 0.9.1 Lit for generated surfaces;
- Focusa Svelte Custom Elements for approved domain controls;
- a genuine PTY-backed interactive Pi Work Surface;
- a typed Desktop presentation/control protocol;
- Workstream-partitioned reducer authority in the Focusa daemon.

Focusa Desktop owns presentation, windows, navigation, layout, local process presentation and native integration. It does not own canonical Mission, Workpoint, Trajectory, Evidence, Context, Attachment or Workstream state.

## 2.2 Pi

Pi remains strategically central.

Pi SHALL continue to provide:

- authentic interactive coding and conversation;
- standalone terminal and SSH operation;
- Focusa tools and commands;
- exact Workstream Attachment binding;
- continuity, rehydration and compaction;
- compact Work Rail and operator status;
- steering and follow-up;
- stable-reference handoff to Desktop;
- an embedded Desktop Work Surface.

Pi SHALL NOT remain the primary renderer for the complete rich Mission Canvas.

The rich Pi Mission Canvas becomes a bounded compatibility, recovery and terminal-first projection. New desktop-only layouts, generated forms, browser canvases, document editors and complex split-pane behavior do not belong in Pi TUI code.

## 2.3 Menubar

The Focusa menubar remains a compact quick-entry, lifecycle and status surface.

It SHALL NOT become a second full Mission Canvas implementation. Accidental rich Mission Canvas/dashboard content is removed only after useful projection logic and tests have been inventoried and moved to a shared package or Desktop workspace.

## 2.4 Focusa.work

Focusa Desktop and Focusa.work are deployment variants of the same portable Mission Canvas architecture:

```text
Shared Mission Canvas/workspace packages
        ├── Focusa Desktop: Tauri + local daemon + local PTY
        └── Focusa.work: web/PWA + hosted or bridged execution
```

Focusa.work supports:

1. hosted Workstream runtime;
2. connected-local mode through a secure outbound Desktop/daemon bridge;
3. self-hosted Workstream runtime.

It is not a separate conceptual rewrite.

## 2.5 UIAI Engine Cockpit

UIAI Engine Cockpit remains a separate product and authority. Its SvelteKit/Tauri implementation is the reference proving the shell quality and interaction architecture.

Focusa and UIAI retain distinct:

- product identity and licensing;
- canonical authorities;
- app bundle IDs and release channels;
- daemon/runtime ownership;
- security boundaries.

Product-neutral shell primitives should be extracted or versioned for reuse rather than silently forked.

## 2.6 Required Spec 135 topology amendment

Existing Spec 135A wording that makes UIAI Engine Cockpit the single primary rich desktop operator environment conflicts with a distinct Focusa Desktop distribution.

The corrected topology is:

```text
Focusa Desktop
  primary rich Focusa cognition, Mission Canvas and Pi environment

UIAI Engine Cockpit
  primary rich UIAI browser execution, FPV and Test Lab environment

Focusa menubar
  compact quick-entry and lifecycle surface

Pi
  standalone and embedded coding/conversation Work Surface

Focusa.work
  hosted/web projection of Focusa workspaces
```

Spec 135A and the 135 current manifest must receive explicit integration notes. Do not silently reinterpret the conflict in code.

---

# 3. Correct canonical identity

Every new or migrated model follows:

```text
ScopeRef / ProjectRootKey
  -> WorkstreamId
    -> ContinuityId
      -> AttachmentKey
        -> SessionId / InstanceId
          -> runtime object identity
            -> WorkSurfaceId
```

Rules:

- Workstream is the durable cognitive workspace.
- Thread is legacy/historical terminology only.
- Continuity is lineage inside a Workstream, not Workstream identity.
- Session and Instance are temporal runtime metadata.
- Attachment binds runtime identity to exactly one Workstream.
- Work Surface is presentation identity and does not grant mutation authority.
- visual focus, current tab, current window, CWD, last project, latest trajectory or daemon remembered state cannot select canonical cognition.

The existing Pi Mission Canvas model currently contains valuable Project, Continuity, Session, Attachment, lifecycle, health, lease, worktree and browser-isolation semantics. It must gain durable Workstream identity before extraction into shared core.

---

# 4. Correct daemon/reducer boundary

Focusa keeps one canonical reduction law, routed to one exact Workstream partition.

```text
Models, tools, agents, UIs and adapters propose.
The Focusa reducer canonizes meaning within one exact Workstream.
```

The daemon may serve many Projects and Workstreams. It must not own one daemon-global cognitive singleton containing global Focus Stack, Workpoint, tactical Trajectory, Work Loop, current Thread, current Instance or global current project selectors.

Daemon-global state is permitted only for infrastructure and explicitly non-cognitive aggregates.

Focusa Desktop must not be built as a polished client over the obsolete mixed global aggregate. Every canonical-capable operation must carry `WorkstreamKey` or an exact Attachment/Object reference that resolves uniquely to it.

---

# 5. GUI/CLI/agent parity

Focusa Desktop is both human-operable and agent-operable through one semantic command graph.

```text
Workspace/Command Manifest
        ├── Desktop navigation
        ├── Desktop command palette
        ├── `focusa desktop ...` CLI
        ├── Pi extension tools and commands
        ├── Focusa agent tool registry
        └── Focusa.work navigation/control
```

Every implemented destination or operation uses stable IDs:

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

The Focusa agent must be able to:

- launch or focus Desktop;
- inspect Desktop version and protocol compatibility;
- navigate to any registered workspace, subsection or saved view;
- open a Workstream, Workpoint, session, document, research packet, Evidence object or Receipt;
- open, focus, close, move, split, pin or detach Work Surfaces where permitted;
- invoke every registered command for which it has authority;
- observe pending approvals, recovery, contention, entitlement and lease states;
- subscribe to Desktop state transitions;
- verify semantic arrival at the requested destination;
- request bounded visual Evidence when appropriate.

The agent must not need coordinate clicking, selectors, OCR or button-label matching.

Full semantic reach does not bypass governance. Scope, capability, entitlement, approvals, writer leases, destructive-action policy, consent, Evidence and security rules apply identically to GUI and CLI.

---

# 6. Presentation commands versus domain commands

Presentation commands affect Desktop state only:

- launch/focus Desktop;
- navigate to workspace/subsection/view;
- open/focus stable object reference;
- change layout;
- split/move/detach a Work Surface;
- reveal inspector.

Domain commands execute through the owning Focusa reducer/service:

- resume Workpoint;
- attach Session;
- approve proposal;
- steer Session;
- link Evidence;
- create Silent Session;
- change authority;
- close or cancel canonical work.

A Svelte state change never proves a domain mutation succeeded. Domain operations return typed results and Receipts, then Desktop updates from canonical state.

---

# 7. Desktop control contract

The exact route names may be refined, but the capability surface is frozen.

```text
GET  /v1/desktop/manifest
GET  /v1/desktop/status
GET  /v1/desktop/state
GET  /v1/desktop/events

POST /v1/desktop/launch
POST /v1/desktop/present
POST /v1/desktop/navigate
POST /v1/desktop/layout/apply

POST /v1/desktop/surfaces/open
POST /v1/desktop/surfaces/focus
POST /v1/desktop/surfaces/close
POST /v1/desktop/surfaces/move
POST /v1/desktop/surfaces/split
POST /v1/desktop/surfaces/detach

POST /v1/desktop/commands/{command_id}/invoke
GET  /v1/desktop/operations/{operation_id}
```

A local socket or named pipe may optimize transport, but it must implement the same typed contract.

A present request includes exact Workstream identity:

```json
{
  "schema": "focusa.desktop_present_request.v1",
  "workspace_id": "mission_canvas",
  "subsection_id": "sessions",
  "view_id": null,
  "object_ref": "workpoint:wp_...",
  "work_surface_id": null,
  "window_mode": "focus_existing",
  "workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "continuity_id": "cont_...",
  "requested_by": {"client_type": "pi|cli|agent|menubar|desktop|web", "client_id": "..."},
  "idempotency_key": "..."
}
```

The receipt echoes the resolved WorkstreamKey and never infers success from visual focus.

Desktop semantic state includes active presentation plus exact authority context:

```json
{
  "schema": "focusa.desktop_state.v1",
  "connected": true,
  "protocol_version": "1.0",
  "active_window_id": "window_main",
  "active_workspace_id": "mission_canvas",
  "active_subsection_id": "sessions",
  "active_object_ref": "session:...",
  "focused_work_surface_id": "surface_pi_01",
  "resolved_workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "open_work_surfaces": [],
  "visible_commands": [],
  "pending_approvals": [],
  "blocked_states": [],
  "dialogs": [],
  "layout": {},
  "updated_at": "..."
}
```

This semantic state, not a screenshot, is the primary agent verification surface.

---

# 8. Focusa CLI surface

Functional parity is required:

```text
focusa desktop manifest --json
focusa desktop status --json
focusa desktop state --json
focusa desktop launch
focusa desktop focus

focusa desktop present \
  --project-root-key prk_... \
  --workstream-id ws_... \
  --workspace mission_canvas \
  --subsection sessions \
  --object-ref session:...

focusa desktop navigate mission_canvas/context \
  --project-root-key prk_... \
  --workstream-id ws_...

focusa desktop open workpoint:wp_... --json
focusa desktop wait --workspace mission_canvas --timeout 30s
focusa desktop events --jsonl

focusa desktop surface list --json
focusa desktop surface open --kind pi_session --workstream-id ws_...
focusa desktop surface focus surface_pi_01
focusa desktop surface split surface_pi_01 --with evidence:...
focusa desktop surface move surface_pi_01 --group right
focusa desktop surface close surface_pi_01

focusa desktop command list --workspace mission_canvas --json
focusa desktop command invoke focusa.workpoint.resume --input request.json
```

CLI rules:

- stable complete JSON mode;
- operation IDs and Receipts;
- idempotency where practical;
- distinct blocked/recovery exit codes;
- headless usefulness when Desktop is closed;
- exact Workstream resolution;
- no local selection convenience treated as canonical authority.

---

# 9. Desktop workspace architecture

Initial taxonomy:

```text
Work
  Overview / Mission Deck
  Mission Canvas
  Pi
  Sessions
  Documents
  Research

Govern
  C.R.I.S.T.
  Context
  Role
  Trajectory
  Approvals
  Contention

Prove
  Evidence
  Receipts
  History
  Reports

System
  Projects & Workstreams
  Nodes & Services
  Capabilities
  Agent Runtime
  Settings
```

Do not create empty duplicate workspaces merely because the taxonomy names them. Each enabled workspace needs a bounded responsibility, typed owner and command manifest.

Mission Deck answers where the operator is, what can resume, what needs attention, what happened and the next safe action.

Mission Canvas is the interactive projection for one exact Workstream. It composes mission, Workpoints, tactical Trajectory, Work Surfaces, Sessions, contention, Evidence, research, documents, C.R.I.S.T., Context, Role, history and approvals.

Aggregate views across Workstreams are advisory read models. Selecting a card does not change daemon authority until an explicit attach/navigate/command request resolves one Workstream.

---

# 10. Existing Pi Mission Canvas preservation classification

## 10.1 Preserve and correct identity before extraction

High-value semantic assets include:

- Work Surface kinds and projection;
- session inventory normalization, de-duplication and ordering;
- Project/Workstream/Workpoint binding;
- Attachment and isolation semantics;
- lifecycle, health, approvals, conflicts and blockers;
- writer leases and worktree identity;
- UIAI browser isolation;
- semantic surface truth and scoped refresh;
- rehydration, North Star and tactical Trajectory;
- Workpoint resume and Evidence linkage;
- operator steering and capability discovery;
- cross-project isolation tests.

Current models that use project root and Continuity without durable WorkstreamId must be corrected before becoming shared core.

## 10.2 Keep in Pi

- tool and command registration;
- session lifecycle hooks;
- exact Attachment binding;
- Work Rail/widget;
- compact operator status;
- terminal/headless compatibility;
- rehydration and continuity prompts;
- steering and follow-up;
- Desktop present/focus command;
- terminal-specific accessibility and width behavior.

## 10.3 Compatibility projection

`mission-canvas-view.ts` and rich terminal panel flow become a bounded compatibility/fallback projection for SSH, recovery, diagnostics and no-Desktop environments.

Do not continue expanding it with desktop-only panes, drag-and-drop, generated forms, browser canvases, documents or rich split layouts.

## 10.4 Command transition

The `/mission-canvas` command should eventually:

```text
when Desktop is available:
  present Mission Canvas for the exact Workstream

when Desktop is unavailable or terminal is explicitly requested:
  open bounded Pi compatibility projection
```

Possible explicit forms:

```text
/mission-canvas
/mission-canvas desktop
/mission-canvas terminal
/focusa-desktop
```

## 10.5 Menubar cleanup

Remove accidental full Mission Canvas content only after inventory and replacement ownership are recorded. Keep compact status, quick resume, pairing, lifecycle and handoff surfaces.

---

# 11. Pi Work Surface architecture

The Desktop Pi Work Surface hosts an authentic interactive Pi process through a cross-platform PTY.

Required behavior:

- Pi remains alive while hidden;
- resize propagates to PTY;
- terminal streams remain authentic;
- exact Workstream/Continuity/Attachment identity is visible;
- Work Surface can be focused, split, detached and restored;
- agent addresses it by stable ID through CLI;
- closing a view does not silently destroy canonical work;
- restart restores presentation safely.

An ordinary child-process pipe is insufficient for a TUI. A real PTY is required.

Interactive Pi and headless Pi RPC remain separate modes:

```text
Interactive Pi
  operator-owned, PTY-backed, visible Work Surface

Headless Pi RPC
  daemon-owned AgentExecutionAdapter, JSONL/RPC, governed background execution
```

They may attach to the same Workstream through distinct Attachment identities.

---

# 12. Shared package topology

Target responsibility shape:

```text
apps/
  desktop/
  menubar/
  pi-extension/

packages/
  desktop-contracts/
  desktop-shell/
  workspace-registry/
  mission-canvas-core/
  mission-canvas-ui/
  pi-work-surface/
  a2ui-renderer/
  focusa-elements/
  generated-client/
```

Exact names may follow repository convention. Responsibility separation may not be weakened.

Generated OpenAPI types own transport DTOs. Bounded projection packages transform them into presentation models. Svelte components do not manually interpret arbitrary daemon `any` payloads.

SvelteKit owns shell, navigation and authored composition. A2UI Lit owns generated surfaces. Focusa Custom Elements own approved generated domain controls. Do not build a competing Svelte A2UI renderer.

---

# 13. Focusa.work portability

Domain components must not import Tauri throughout their implementation.

Use environment adapters:

```text
DesktopEnvironment
  local daemon, local PTY, filesystem, keychain, updater, notifications

WebEnvironment
  hosted API, remote PTY stream, browser storage, web deployment, web notifications

LocalBridgeEnvironment
  Focusa.work UI plus secure outbound connection to local Desktop/daemon
```

Stable workspace IDs, subsection IDs, object refs, command IDs, Workstream identity and semantic state carry across Desktop and web.

---

# 14. Packaging and runtime compatibility

The primary evaluation path should become:

```text
Install Focusa Desktop
Activate evaluation/license
Select Project and Workstream
Connect provider/model
Open Mission Deck
Resume or create Workpoint
Open embedded Pi when needed
```

The integrated experience must not depend on arbitrary user-managed Node/Pi state.

Choose one governed Pi distribution strategy:

```text
preferred:
  approved standalone Pi binary as signed sidecar

alternative:
  bundled Node runtime meeting the pinned Pi minimum
  plus pinned Pi package and Focusa extension
```

Current Focusa Pi package and upstream Pi runtime requirements must be reconciled explicitly.

Publish and test an atomic compatibility matrix for Desktop, daemon/API, CLI, Pi, Pi extension, generated contracts, PTY protocol, Desktop-control protocol and UIAI interoperability.

---

# 15. Worktree preservation protocol

Before rebase, merge, deletion, rename, broad formatting or lockfile regeneration, record:

```text
pwd
git status --short --branch
git branch --show-current
git rev-parse HEAD
git log --oneline --decorate -n 50
git diff --stat
git diff
git diff --cached
git worktree list --porcelain
git branch -vv
git remote -v
git stash list
```

Inventory:

- untracked files;
- ignored mission-relevant generated artifacts;
- local notes and spec drafts;
- screenshots and fixtures;
- unpushed commits;
- other worktrees containing related daemon/API changes.

Create a preservation checkpoint such as:

```text
checkpoint/spec-135-pi-mission-canvas-pre-desktop-pivot-2026-08-04
```

Commit all intentional work. If local-only or sensitive artifacts cannot be committed, create a sanitized archive and exclusion manifest.

Do not force-push existing branches. Do not blindly rebase the substantially diverged Mission Canvas branches. Review unique commits and migrate deliberately.

---

# 16. Required migration ledger

One row per relevant file or behavior:

```text
path
current responsibility
unique branch/local changes
tests
current identity fields
required Workstream correction
new owner
disposition
migration task
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

No file is deleted without a ledger row and replacement/retirement proof.

---

# 17. Transition workroute

## Phase 0 — Stop expansion and preserve

- stop new rich Pi panels;
- capture exact worktree state;
- create checkpoint;
- record tests and known failures;
- produce migration ledger;
- inventory branch-only and local-only work.

## Phase 1 — Freeze Spec 158 foundation

- inventory global cognitive selectors;
- reserve stable Workstream identity;
- establish WorkstreamContext and ScopeRouter contracts;
- prohibit new global cognitive authority;
- decompose Spec 158 task graph.

## Phase 2 — Workstream identity and reducer routing

- add WorkstreamId/WorkstreamKey;
- separate Continuity;
- add AttachmentKey/workspace binding;
- route canonical events to exact Workstream;
- fail closed on ambiguity.

## Phase 3 — Core singleton cutover

- partition Workpoint and tactical Trajectory;
- partition Focus Stack and Focus State;
- partition Work Loop, writer leases and Silent Sessions;
- partition Context, memory, ontology, Evidence and claims;
- remove global writes, reads and selectors after parity.

## Phase 4 — Extract Mission Canvas semantic core

- correct identity in Work Surface models;
- move session reconciliation and semantic projections out of Pi TUI;
- replace `any` payload interpretation with generated DTOs;
- preserve tests before changing behavior;
- keep Pi adapter consuming shared core.

## Phase 5 — Desktop shell vertical slice

- Tauri single-instance shell;
- shared/proven sidebar and command palette;
- Context Control with exact Workstream;
- Mission Deck and Mission Canvas overview;
- Work Rail and truthful Evidence projection;
- daemon discovery, entitlement and updater;
- semantic Desktop state.

## Phase 6 — Agent/CLI control plane

- Desktop-control contracts and presenter routes;
- `focusa desktop` CLI;
- machine-readable workspace/command manifests;
- semantic state and event stream;
- Receipts, idempotency and parity tests;
- Pi/agent tools wrapping the same contracts.

## Phase 7 — Embedded Pi Work Surface

- cross-platform PTY;
- pinned/package Pi runtime;
- automatic Focusa extension;
- exact Attachment binding;
- process survival, split view and restoration;
- separate headless RPC adapter.

## Phase 8 — Full capability migration

In dependency order:

1. current work and Workpoints;
2. tactical Trajectory;
3. Work Surfaces and Sessions;
4. C.R.I.S.T. generated UI;
5. Context and Role;
6. Evidence and Receipts;
7. research and documents;
8. contention and approvals;
9. history and recovery;
10. Silent Sessions and UIAI surfaces.

## Phase 9 — Focusa.work portability

- hosted runtime;
- connected-local bridge;
- remote PTY;
- mobile/responsive surfaces;
- same semantic command graph.

## Phase 10 — Compatibility retirement

- prove terminal-only, Desktop-offline and CLI/headless use;
- mark obsolete presentation tests;
- remove only code with replacement parity;
- publish migration and rollback notes.

---

# 18. Required regression gates

## Preservation

- all local commits reachable from checkpoint;
- untracked work archived or committed;
- ledger covers all Mission Canvas files/tests;
- no force-update;
- no deletion without disposition.

## Spec 158 foundation

- no core daemon-global cognitive aggregate;
- no canonical current Thread/current Instance selector;
- one-active semantics per Workstream;
- deterministic per-Workstream replay;
- ambiguous records quarantined;
- no permanent dual canonical writes.

## Semantic isolation

- Work Surface carries WorkstreamId;
- no cross-Workstream Workpoint, Focus, Trajectory or Work Loop mutation;
- lease/worktree/browser isolation remains visible;
- no invented canonical state;
- stable Evidence/Receipt refs preserved.

## GUI/CLI/agent parity

For every implemented workspace and command:

- stable manifest entry;
- GUI present/invoke path;
- CLI path;
- agent tool path;
- identical capability/entitlement/scope behavior;
- typed operation result;
- semantic verification;
- blocked/recovery test.

## Desktop/Pi

- genuine PTY;
- Pi survives hidden views;
- closing view does not kill canonical work silently;
- restart restoration;
- terminal fallback;
- Pi can request Desktop presentation;
- CLI can focus Pi Work Surface.

## No duplication

- no handwritten duplicate transport DTOs;
- no separate workspace taxonomy per client;
- no separate command registry per client;
- no Mission state owned by Svelte stores;
- no full Mission Canvas in menubar;
- no second browser authority;
- no competing A2UI renderer.

---

# 19. Immediate instructions to the current Mission Canvas agent

1. Do not continue expanding the rich Pi Mission Canvas UI.
2. Finish only work needed to leave the current worktree coherent and testable.
3. Fetch remote and read `docs/agent/00-p0-transition-bootstrap.md`.
4. Preserve/checkpoint all current state before rebase or cleanup.
5. Produce the migration ledger.
6. Identify unique local/branch work not on main.
7. Do not rebase until the unique-work report is reviewed.
8. Treat Work Surface, Session, Attachment and isolation logic as high-value semantic assets.
9. Correct Workstream identity before shared extraction.
10. Treat Pi rich rendering as compatibility unless reassigned.
11. Add no Tauri/Svelte implementation inside the Pi extension.
12. Add no Focusa Desktop domain state to UIAI Engine.
13. Design every new Desktop action for GUI/CLI/agent parity.
14. Use stable IDs and typed operation Receipts from the first slice.
15. Report spec conflicts rather than silently choosing one.

---

# 20. Required first handoff response

The implementing agent must respond with:

```text
A. Current worktree identity
B. Current HEAD and branch
C. Uncommitted/untracked inventory
D. Unpushed commit inventory
E. Mission Canvas file inventory
F. Test inventory and latest results
G. Unique work not on main
H. Preservation checkpoint ref
I. Migration ledger
J. Spec 158 identity conflicts
K. Risks/blockers
L. Proposed first extraction or cleanup task
M. Task-graph nodes to claim
```

No broad implementation continuation is accepted without this preservation report.

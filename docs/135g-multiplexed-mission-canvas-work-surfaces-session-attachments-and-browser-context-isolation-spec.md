# Spec 135G — Multiplexed Mission Canvas, Work Surfaces, Session Attachments, and Browser Context Isolation

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135G.  
**Scope:** multiplexed interaction, Mission Canvas naming, Work Surfaces, Instances, Sessions, Attachments, concurrent Pi/harness sessions, Silent Sessions, UIAI browser sessions, isolated browser contexts, browser targets, tab and split-pane UX, per-surface steering, aggregate views, contention, proposal resolution, writer isolation, rehydration, events, APIs, client parity, migration, and proof.

---

## 0. One-line definition

Focusa Mission Canvas is the multiplexed interactive work surface that lets an operator observe, steer, compare, suspend, resume, and organize many concurrent project/workstream attachments—Pi sessions, UIAI browser contexts, Silent Sessions, research, documents, and provider work—without restoring singleton authority or confusing visual focus with canonical project state.

---

## 1. Naming constitution

### 1.1 Reserved product name

The term **Cockpit** is reserved exclusively for **UIAI Engine Cockpit**.

Focusa documentation, schemas, components, tests, and UI labels must not use:

- Focusa Cockpit;
- professional cockpit;
- mission cockpit;
- Pi cockpit;
- runtime cockpit;
- Tauri cockpit;
- cockpit view;
- cockpit mode;

unless the reference is explicitly to **UIAI Engine Cockpit**.

### 1.2 Focusa surface names

```text
Focusa Mission Canvas
  The interactive Focusa workspace projection presented inside Pi,
  Mission Deck, native TUI, menubar expansions, or UIAI Engine Cockpit.

Work Surface
  One user-visible tab, pane, split, or detached window within a Mission Canvas.
  It projects one primary Attachment and may show related supporting attachments.

Mission Deck
  Focusa’s standalone guided PWA/experience for project identity, C.R.I.S.T.,
  mission, proof, Workpoints, Receipts, and next-safe-action guidance.

Spec Workbench
  Spec 120’s adversarial specification-authoring and approval environment.

UIAI Engine Cockpit
  The companion rich desktop product shell that hosts browser, FPV, Test Lab,
  Documents, Research, artifacts, automations, and Focusa Mission Canvas projections.
```

### 1.3 Why Mission Canvas

`Mission Canvas` is selected because it:

- avoids collision with UIAI Engine Cockpit;
- avoids collision with Spec Workbench;
- preserves Mission Deck and living-field language;
- describes a composable surface rather than one fixed dashboard;
- naturally supports tabs, split panes, overlays, rails, and multiple simultaneous work objects;
- remains valid in terminal, PWA, Tauri, and embedded contexts.

---

## 2. Normative basis

This specification extends and operationalizes:

- [Spec 38](38-thread-thesis-spec.md) — cognitive workspaces and thread thesis;
- [Spec 39](39-thread-lifecycle-spec.md) — thread lifecycle;
- [Spec 40](40-instance-session-attachment-spec.md) — Instances, Sessions, Attachments, and multiplexing engineers;
- [Spec 41](41-proposal-resolution-engine.md) — asynchronous concurrent proposals and deterministic resolution;
- [Spec 43](43-multi-device-sync.md) — local-first multi-device synchronization;
- [Spec 98](98-project-root-crdt-reconciliation-foundation-spec.md) — ProjectRootKey, WorkstreamKey, AttachmentKey, and CRDT-grade reconciliation;
- [Spec 104](104-typed-scoped-runtime-and-singleton-elimination-spec.md) — typed scope and removal of authority-bearing singletons;
- [Spec 133](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md) — durable autonomous execution sessions, runs, writer leases, and worktree isolation;
- [Spec 135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) — workspace projection and Mission Canvas visual composition;
- [Spec 135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) — UIAI artifacts, FPV, browser sessions, and live refresh;
- [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) — complete implementation graph and no-deferral discipline;
- [Spec 135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) — semantic registry, domain packs, candidate/canonical graph separation, and reactive context.

If any client-local interpretation reintroduces one global active project, workstream, session, browser context, Workpoint, or task as canonical authority, Specs 98, 104, and 135G win.

---

## 3. Product intent

Removing singleton authority was not only a bleed-prevention exercise. It was intended to enable high-integrity multiplexing:

```text
many projects
+ many workstreams
+ many interactive sessions
+ many autonomous sessions
+ many browser contexts
+ many devices and harnesses
+ one deterministic governed Focusa substrate
```

The user should be able to keep several kinds of work alive at once:

```text
[Project Overview]
[Pi · implement task 23]
[UIAI · authenticated admin verification]
[UIAI · isolated legal research]
[Silent Session · integration tests]
[Documents · contract redline]
```

Each Work Surface must remain independently attributable, steerable, recoverable, and scope-safe.

---

## 4. Canonical authority hierarchy

```text
ProjectRootKey
  Verified project source of truth.

WorkstreamKey
  ProjectRootKey + continuity_id.

AttachmentKey
  WorkstreamKey + instance_id + session_id + attachment_id.

Runtime-specific identities
  silent_session_id + run_id
  uiai_session_id + browser_context_id + browser_target_id
  harness-native session reference
```

A Mission Canvas or Work Surface is a projection over these identities. It does not create canonical authority.

### 4.1 Visual focus is not canonical activity

```text
focused_work_surface_id
≠
global active session
≠
global active Workpoint
≠
global current project
```

A client may have one keyboard-focused Work Surface while many sessions continue running.

### 4.2 Aggregate views are labeled projections

A Project Overview, All Sessions, or Global Activity view may aggregate multiple projects/workstreams. It must identify itself as an aggregate and may not be used as an implicit mutation target.

---

## 5. Runtime entities

### 5.1 Existing entities preserved

```text
Instance
  Where cognition or execution is invoked.

Session
  A temporal execution window within an Instance.

Attachment
  A binding between Instance/Session and one Focusa workstream/thread.
```

Attachment roles remain:

- active;
- assistant;
- observer;
- background.

Attachments grant at most proposal/read posture. Canonical mutation still follows reducer and authority paths.

### 5.2 Work Surface

A Work Surface is a durable, rehydratable client projection describing how one primary Attachment is presented.

```yaml
schema: focusa.work_surface.v1

work_surface_id: uuidv7
display_name:
kind: project_overview | pi_session | uiai_browser | silent_session | document | research | provider_item | evidence | custom

scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workpoint_id:
  work_item_ref:

primary_attachment:
  instance_id:
  session_id:
  attachment_id:
  role:

runtime_refs:
  harness_session_ref:
  silent_session_id:
  silent_run_id:
  uiai_session_id:
  browser_context_id:
  primary_browser_target_id:
  document_session_id:

supporting_attachment_refs: []

presentation:
  workspace_profile_ref:
  renderer_id:
  title:
  icon:
  pinned: false
  group_id:
  split_group_id:
  position:
  inspector_state_ref:

activity:
  lifecycle_state:
  semantic_activity:
  health:
  unread_event_count: 0
  pending_approval_count: 0
  conflict_count: 0
  blocker_count: 0
  last_activity_at:

queues:
  steering_queue_ref:
  follow_up_queue_ref:

isolation:
  writer_lease_ref:
  worktree_ref:
  browser_isolation_class:

created_at:
updated_at:
closed_at:
```

### 5.3 Mission Canvas State

Mission Canvas state is split into project-shared semantic defaults and user/device presentation state.

```yaml
schema: focusa.mission_canvas_state.v1

canvas_id:
client_instance_id:
user_id:
device_id:

open_work_surface_ids: []
focused_work_surface_id:
secondary_focused_surface_id:
split_layout_ref:
group_order: []

aggregate_filters:
  project_roots: []
  continuity_ids: []
  kinds: []
  states: []

restoration:
  last_persisted_at:
  layout_revision:
  session_projection_revision:
```

Mission Canvas presentation state is not canonical project truth. Attachment/session identity and project/workstream bindings remain canonical Focusa records.

---

## 6. Browser multiplexing and isolation

### 6.1 Required hierarchy

```text
UIAI Browser Session
└── Browser Context / Container
    ├── Browser Target / Tab A
    ├── Browser Target / Tab B
    └── Worker/Popup targets where supported
```

The terms must not be collapsed:

- a **browser target** is a page/tab/worker target;
- a **browser context** is an isolated cookie/storage/permission container;
- a **UIAI session** is the supervised browser execution object;
- a **Focusa Attachment** binds the UIAI session/context to a project/workstream;
- a **Work Surface** presents that attachment to the operator.

### 6.2 Browser isolation classes

```text
shared_authenticated
  Targets intentionally share one authenticated browser context.

isolated_authenticated
  Dedicated persistent context with its own cookies/storage/permissions.

ephemeral_isolated
  Dedicated disposable context destroyed according to retention policy.

read_only_observer
  Observation/capture context with mutation controls disabled.

capture_worker
  Bounded background context for screenshots, extraction, or verification.
```

### 6.3 Isolation laws

1. Two Work Surfaces must not share a browser context accidentally.
2. Shared context requires an explicit context reference and visible shared-authentication badge.
3. Isolated contexts have separate cookies, local storage, session storage, service workers, permissions, downloads, and browser cache where the browser backend supports them.
4. Browser targets inherit context isolation but maintain separate target IDs and navigation histories.
5. Closing a browser target does not close the context unless it is the last target and policy allows cleanup.
6. Closing a Work Surface does not terminate a UIAI session/context unless the operator chooses a close-and-terminate action.
7. Cross-project context reuse is forbidden by default and requires a separately approved shared-resource policy.
8. Context credentials and storage are never copied into Focusa prompts or event payloads.

### 6.4 Visible browser identity

Every browser Work Surface shows:

- UIAI session;
- browser context/container;
- target/tab count;
- current target URL/title;
- isolation class;
- authentication-sharing posture;
- project/workstream/Workpoint binding;
- FPV state;
- diagnostics;
- evidence and artifact count;
- retention/cleanup posture.

---

## 7. Mission Canvas interaction model

### 7.1 Required controls

The Mission Canvas supports:

- open Work Surface;
- focus;
- pin/unpin;
- group by project/workstream/kind;
- reorder;
- split horizontally/vertically;
- compare side by side;
- move between windows where the host supports it;
- suspend projection without stopping runtime work;
- resume/rehydrate;
- close projection;
- terminate underlying session through a separate governed action;
- duplicate a browser target into the same or a new isolated context;
- move a browser target to a new isolated context where supported;
- open aggregate Project Overview;
- inspect contention/proposals;
- inspect authority, writer leases, worktrees, and isolation.

### 7.2 Required tab/pane indicators

Each Work Surface tab or pane displays bounded indicators for:

- project/workstream;
- session kind;
- running/waiting/blocked/paused/completing state;
- health;
- unread events;
- pending operator input;
- approvals;
- conflicts/proposals;
- writer lease;
- browser isolation;
- evidence/proof readiness.

### 7.3 Aggregate versus local rails

The Work Rail has two explicit modes:

```text
Surface-local
  Work items, Workpoint, evidence, and queues for the focused Work Surface.

Project aggregate
  All relevant work items and sessions under one verified ProjectRootKey.

Cross-project advisory
  Labeled read-only aggregation. No implicit canonical mutation target.
```

### 7.4 Steering and follow-up routing

Steering and follow-up instructions must target an explicit Attachment, Silent Session, UIAI session/context, or selected set.

Required UI:

```text
Send to:
● Focused Pi session
○ Silent Session · tests
○ UIAI context · admin verification
○ All selected sessions [requires preview]
```

Broadcast steering requires a preview showing recipients, project/workstream scope, roles, and authority. Accidental implicit broadcast is forbidden.

---

## 8. Concurrent work and conflict governance

### 8.1 Observations

Append-only observations may arrive concurrently from many sessions:

- browser captures;
- references;
- diagnostics;
- test results;
- telemetry;
- evidence candidates;
- context artifacts.

They remain source-attributed by Instance, Session, Attachment, and runtime-specific IDs.

### 8.2 Decisions

Competing decisional proposals use Spec 41 PRE and reducer authority.

The Mission Canvas must show:

- pending proposals per project/workstream/target;
- originating Work Surface, Instance, Session, and Attachment;
- evidence and confidence;
- resolution window/status;
- accepted/rejected/superseded outcomes;
- reasons and citations;
- fork/compare actions where allowed.

### 8.3 Writer isolation

Two mutation-capable sessions must not write the same dirty workspace unless a shared-writer policy is explicitly approved.

Default behavior:

```text
foreground interactive writer
  current worktree or approved workspace lease

background/silent writer
  isolated worktree

browser-only/research observer
  no filesystem writer lease
```

The Mission Canvas visibly displays writer lease and worktree ownership.

---

## 9. Lifecycle and restoration

### 9.1 Work Surface lifecycle

```text
created
→ opening
→ open
→ focused / background_visible
→ suspended
→ rehydrating
→ open
→ closing
→ closed
→ archived
```

Work Surface lifecycle is independent of underlying runtime lifecycle.

### 9.2 Closing semantics

The close menu must distinguish:

```text
Close view
  Remove the Work Surface while leaving the session/context running.

Pause session
  Governed pause of the underlying execution where supported.

Terminate session
  Governed stop/cancel with impact preview.

Close browser target
  Close one target only.

Close browser context
  Close all targets in that isolated context after preview.
```

### 9.3 Rehydration

After client restart, the Mission Canvas restores:

- open Work Surface references;
- focused/pinned/group/split state;
- associated Attachment/runtime identities;
- latest bounded read models;
- unread cursor;
- health/degraded indicators;
- missing/ended-session recovery actions.

It must not manufacture a new canonical session or adopt a different project because an old runtime reference is unavailable.

---

## 10. Artifact and event identity

Every session-originated artifact/event should carry, when applicable:

```yaml
origin:
  instance_id:
  focusa_session_id:
  attachment_id:
  work_surface_id:
  harness_session_ref:
  silent_session_id:
  silent_run_id:
  uiai_session_id:
  browser_context_id:
  browser_target_id:
```

Project/workstream scope remains separately required:

```yaml
scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workpoint_id:
  work_item_ref:
```

Origin identity explains where an observation came from. Scope defines which project/workstream it belongs to. Neither may substitute for the other.

---

## 11. Event taxonomy

Required projection events include:

```text
mission_canvas_surface_created
mission_canvas_surface_updated
mission_canvas_surface_focused
mission_canvas_surface_suspended
mission_canvas_surface_rehydrated
mission_canvas_surface_closed
mission_canvas_layout_changed

attachment_added
attachment_role_changed
attachment_detached
session_started
session_state_changed
session_ended

browser_context_created
browser_context_isolation_changed
browser_context_closed
browser_target_opened
browser_target_navigated
browser_target_moved
browser_target_closed

surface_unread_changed
surface_approval_required
surface_conflict_changed
surface_writer_lease_changed
```

Events contain scoped IDs, versions, cursors, and invalidation keys rather than full transcripts, page bodies, images, or browser storage.

---

## 12. Bounded read models

Required read models:

```text
mission_canvas.summary
mission_canvas.open_surfaces
mission_canvas.surface_detail
mission_canvas.session_inventory
mission_canvas.project_activity
mission_canvas.contention
mission_canvas.layout

uiai.session_inventory
uiai.browser_context_inventory
uiai.browser_target_inventory

silent_sessions.inventory
attachments.inventory
```

Large event histories and artifact lists use pagination/virtualization.

---

## 13. API families

Conceptual typed APIs:

```text
GET  /v1/mission-canvas
GET  /v1/mission-canvas/surfaces
POST /v1/mission-canvas/surfaces/preview
POST /v1/mission-canvas/surfaces
GET  /v1/mission-canvas/surfaces/:id
POST /v1/mission-canvas/surfaces/:id/focus
POST /v1/mission-canvas/surfaces/:id/suspend
POST /v1/mission-canvas/surfaces/:id/close/preview
POST /v1/mission-canvas/surfaces/:id/close
POST /v1/mission-canvas/layout/preview
POST /v1/mission-canvas/layout

GET  /v1/runtime/instances
GET  /v1/runtime/sessions
GET  /v1/runtime/attachments
GET  /v1/runtime/contention

GET  /v1/uiai/sessions
GET  /v1/uiai/browser-contexts
GET  /v1/uiai/browser-contexts/:id/targets
POST /v1/uiai/browser-targets/:id/move-context/preview
POST /v1/uiai/browser-targets/:id/move-context
```

Canonical runtime mutation routes remain owned by their existing subsystems. Mission Canvas routes mainly create/update projections and invoke governed underlying actions through typed refs.

---

## 14. Client requirements

### 14.1 UIAI Engine Cockpit

The primary rich implementation provides:

- persistent Work Surface tab strip;
- grouped tabs;
- split panes;
- detached windows where supported;
- project overview and all-session inventory;
- browser-context and target management;
- FPV pane per UIAI session/context;
- contention/proposal center;
- Work Rail aggregate/local toggle;
- explicit routing for steering and approvals.

### 14.2 Focusa-enhanced Pi

Pi provides:

- one keyboard-focused Work Surface at a time;
- fast session/workstream switcher;
- persistent right Mission Canvas sidebar;
- lower Work Rail;
- session/activity badges;
- drawers/overlays for inventory and contention;
- explicit target selection for steering/follow-up.

Multiple sessions may continue running while one is focused.

### 14.3 Mission Deck PWA

Mission Deck provides guided project and session overview, session restoration, approvals, and mobile-friendly steering. It need not render every terminal transcript simultaneously but must expose all active sessions truthfully.

### 14.4 Native TUI

The native TUI supports tabs or a session switcher, aggregate Project Overview, session-local detail, and truthful limited-width fallbacks.

### 14.5 Menubar

The menubar shows bounded counts and urgent peeks:

- active sessions;
- waiting-for-operator;
- blocked;
- approvals;
- conflicts;
- unhealthy contexts;
- open Mission Canvas/UIAI Engine Cockpit action.

It does not become the full multiplexed UI.

---

## 15. Migration

### 15.1 Singleton UI state

Any client field named `currentSession`, `activeProject`, `activeWorkpoint`, `currentBrowser`, or equivalent must be classified as either:

- keyboard-focused client projection;
- scoped current value inside a WorkstreamKey;
- legacy singleton requiring migration.

A client-focused value must be renamed or documented so it cannot be mistaken for daemon-global authority.

### 15.2 Existing UIAI sessions

Existing UIAI sessions map to Work Surfaces using available project/workstream and Focusa scope metadata. Missing browser-context identity produces an explicit legacy/shared-context posture until upgraded; it must not be silently labeled isolated.

### 15.3 Existing Pi sessions

Existing Pi attachments map to Work Surfaces keyed by verified project root, continuity, instance, session, and attachment. Session-local UI state must not remain in one process-global object.

### 15.4 Existing generic “cockpit” names

Within the Spec 135 series and new implementation surfaces:

- generic Focusa/Pi workspace uses become **Mission Canvas**;
- individual tabs/panes become **Work Surfaces**;
- only the actual **UIAI Engine Cockpit** retains Cockpit naming.

Legacy code/test names outside the Spec 135 implementation path should be migrated through their owning spec/task with compatibility aliases where externally observable.

---

## 16. Acceptance criteria

Spec 135G is accepted when:

1. Specs 38–41, 43, 98, 104, and 133 are explicit dependencies of the Spec 135 series.
2. Mission Canvas and Work Surface schemas are versioned and generated for all clients.
3. Multiple projects can remain open without state bleed.
4. Multiple workstreams under one project can remain active simultaneously.
5. Multiple Pi/harness sessions can attach to the same or different workstreams.
6. Multiple Silent Sessions can run, pause, resume, and complete independently.
7. Multiple UIAI sessions and browser contexts are visible and independently steerable.
8. One browser context can contain multiple targets/tabs.
9. Separate browser contexts prove cookie/storage/permission isolation.
10. Shared browser context use requires explicit visible selection.
11. Closing a Work Surface does not implicitly terminate its runtime.
12. Closing/terminating actions show distinct previews and effects.
13. Work Rail switches truthfully among surface-local, project aggregate, and cross-project advisory modes.
14. Steering/follow-up routes to explicit attachments and broadcast requires preview.
15. Concurrent observations remain source-attributed and append-only.
16. Concurrent conflicting decisions enter PRE/reducer resolution without silent overwrite.
17. Writer lease and worktree isolation are visible and enforced.
18. Mission Canvas restores tabs, splits, attachment refs, unread cursors, and degraded states after restart.
19. Artifacts/events carry both project/workstream scope and session-origin identity.
20. UIAI Engine Cockpit, enhanced Pi, Mission Deck, TUI, menubar, API, and CLI expose compatible session inventories.
21. The phrase Cockpit is used only for UIAI Engine Cockpit in the Spec 135 series and new implementation labels.
22. Actual multi-session, multi-project, browser-isolation, conflict, recovery, and visual proof is captured.

---

## 17. Required proof scenarios

### 17.1 Two-project Mission Canvas

- Open project A and project B.
- Open Pi and UIAI Work Surfaces in both.
- Mutate project A through an approved action.
- Prove no project B Workpoint, context, artifact, queue, or authority is adopted.

### 17.2 Same-project concurrent sessions

- Open two Pi sessions and one UIAI session under the same verified root.
- Bind them to two workstreams.
- Prove independent Workpoints, queues, histories, and targeted steering.

### 17.3 Same-workstream contention

- Attach two sessions to one WorkstreamKey.
- Submit compatible observations and conflicting decisional proposals.
- Prove append-only merge for observations and PRE resolution for decisions.

### 17.4 Browser container isolation

- Create two isolated authenticated contexts under one project.
- Authenticate differently.
- Prove cookies/storage/permissions do not cross.
- Open multiple targets in each and prove target identity and context membership.

### 17.5 Shared-context warning

- Open two Work Surfaces intentionally sharing one browser context.
- Prove visible shared-authentication indicators and explicit operator action.

### 17.6 Work Surface close semantics

- Close a view while its Silent Session and UIAI context remain active.
- Reopen and rehydrate.
- Separately terminate the runtime and prove impact preview and Receipt/event trail.

### 17.7 Client restart

- Persist multiple open Work Surfaces and splits.
- Restart the UIAI Engine Cockpit or Pi client.
- Prove deterministic restoration without manufacturing or adopting a new canonical session.

### 17.8 Concurrent writers

- Launch foreground and background writing sessions.
- Prove writer-lease conflict handling or isolated worktree assignment.
- Prove no silent shared dirty-worktree mutation.

---

## 18. Closure blockers

This spec cannot close while:

- the UI assumes one global active session;
- Work Surface focus is treated as canonical project/workstream authority;
- session-origin IDs are absent from artifacts/events;
- browser target and browser context are conflated;
- two supposedly isolated contexts share storage;
- Work Surface close terminates runtime implicitly;
- steering can broadcast without recipient preview;
- Work Rail has no aggregate/local distinction;
- concurrent decisions silently overwrite;
- writer isolation is absent or invisible;
- restoration relies on transcript tail or process-global state;
- generic Focusa/Pi surfaces are named Cockpit;
- any required multiplexing proof is surrogate or missing.

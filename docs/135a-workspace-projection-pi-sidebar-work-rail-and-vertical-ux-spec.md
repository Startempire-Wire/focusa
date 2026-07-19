# Spec 135A — Workspace Projection, Mission Canvas, Pi Sidebar, Work Rail, Themes, and Vertical UX

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135A.  
**Scope:** Workspace View Profiles, Focusa Mission Canvas, multiplexed Work Surfaces, domain-pack bindings, dynamic panel/layout/rendering registries, Pi integration and enhanced distribution, Work Rail, steering/follow-up composition, themes, professional verticals, artifact viewers, session inventory, contention, history, accessibility, responsive behavior, and client parity.

---

## 0. One-line definition

Focusa should project one canonical project runtime into a dynamic, project-selectable **Mission Canvas** that stays visible and live inside Pi and other Focusa clients, supports many concurrent Work Surfaces without restoring singleton authority, and makes each vertical visually striking through layout, artifact views, terminology, geometry, density, iconography, and theme—not through color alone.

---

## 1. Normative basis

This spec instantiates:

- [Spec 40](40-instance-session-attachment-spec.md) — Instances, Sessions, Attachments, and multiplexing engineers;
- [Spec 41](41-proposal-resolution-engine.md) — concurrent proposals and resolution visibility;
- [Projection and View Semantics](75-projection-and-view-semantics.md);
- [Spec 98](98-project-root-crdt-reconciliation-foundation-spec.md) — ProjectRootKey, WorkstreamKey, AttachmentKey, and multi-session reconciliation;
- [Spec 104](104-typed-scoped-runtime-and-singleton-elimination-spec.md) — strict anti-singleton scoped runtime;
- [Visual/UI Ontology Core](58-visual-ui-ontology-core.md);
- [Visual/UI Evidence and Workflow](62-visual-ui-evidence-and-workflow.md);
- [Visual/UI Focusa Integration](65-visual-ui-focusa-integration.md);
- [Spec 117](117-mission-deck-onboarding-recall-pwa-spec.md) and [117A](117a-living-mission-field-pwa-spec.md);
- [Spec 119](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md);
- [Spec 121](121-menubar-rearchitecture-spec.md) and [121A](121a-menubar-discipline-and-living-field-spec.md);
- [Spec 124](124-focusa-cli-redesign-project-dashboard-project-creation-scoped-authority-first-mission-command-hierarchy-and-launch-hardening-spec.md);
- [Spec 133](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md);
- [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md);
- [Spec 135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md);
- [Spec 135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md).

The workspace is a projection over canonical state. It is not a new authority or memory store. The focused Work Surface is client presentation state, not a daemon-global active project, session, Workpoint, task, or browser context.

---

## 2. Current reality

The current Focusa native TUI:

- uses Ratatui/Crossterm;
- polls and renders live daemon data;
- has a fixed `Tab` enum;
- uses hard-coded render dispatch;
- has one static palette;
- has Mission Control grid/stack behavior;
- supports modals, scrolling, help, and command concepts.

The current Pi extension:

- registers tools, commands, events, shortcuts, status, and message renderers;
- supports project-scoped settings;
- can open custom TUI components and widgets;
- does not currently own a first-class persistent right-sidebar slot;
- must coexist with steering/follow-up extensions;
- is pinned to a different Pi package line than UIAI’s extension;
- has scoped-state foundations but does not yet expose a complete multi-session Mission Canvas, Work Surface inventory, split-pane model, or browser-context isolation UI.

The implementation must therefore add a shared dynamic UI substrate, converge the Pi extension package/runtime, and carry Specs 40/41/98/104/133 into the visible user experience before the Mission Canvas is considered operational.

---

## 3. Experience architecture

### 3.1 Preferred Mission Canvas composition

```text
┌ WORK SURFACES ──────────────────────────────────────────────────────────────┐
│ [Overview] [Pi · task 23] [UIAI · admin] [Silent · tests] [Research]      │
├──────────────────────── FOCUSED WORK SURFACE ──────────────┬──── FOCUSA ──┤
│ Conversation, tools, browser, document, or result         │ Project       │
│                                                           │ Workspace     │
│                                                           │ Session       │
│                                                           │ Current work  │
│                                                           │ Next work     │
│                                                           │ Proof         │
│                                                           │ Context/Role  │
│                                                           │ Contention    │
│                                                           │ Controls      │
├───────────────────────────────────────────────────────────┴───────────────┤
│ FOCUSA WORK RAIL · SURFACE LOCAL / PROJECT AGGREGATE                      │
├───────────────────────────────────────────────────────────────────────────┤
│ STEERING QUEUE · EXPLICIT TARGET                                          │
├───────────────────────────────────────────────────────────────────────────┤
│ FOLLOW-UP QUEUE · EXPLICIT TARGET                                         │
├───────────────────────────────────────────────────────────────────────────┤
│ PROMPT EDITOR                                                             │
└───────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Lane definitions

```text
Work Surface strip
  Open Pi, UIAI, Silent Session, document, research, evidence, and aggregate views.

Work Rail
  Canonical/advisory project work projection in surface-local or aggregate mode.

Steering Queue
  Operator instructions delivered to an explicitly selected attachment at the
  next safe active-turn boundary.

Follow-up Queue
  Operator instructions delivered to an explicitly selected attachment after
  its current agent run.

Prompt Editor
  New immediate user input targeted to the focused Work Surface by default.
```

A task is not steering. Steering is not a task. Follow-up is not an upcoming provider work item. Visual focus is not canonical session authority.

### 3.3 Naming and object boundary

```text
Focusa Mission Canvas
  The complete interactive Focusa workspace projection.

Work Surface
  One tab, pane, split, or detached window within the Mission Canvas.

Spec Workbench
  The distinct Spec 120 adversarial specification environment.

UIAI Engine Cockpit
  The companion rich desktop shell that may host Mission Canvas projections.
```

The word Cockpit is reserved exclusively for UIAI Engine Cockpit. Focusa/Pi workspace overlays, sidebars, tab groups, and session views are Mission Canvas or Work Surface components.

---

## 4. Pi capability levels

### 4.1 Compatibility Mode

For unmodified Pi:

- focused Work Surface only;
- fast session/workstream switcher;
- Work Rail widget above the editor;
- Focusa drawer or modal;
- custom footer/status;
- custom message/tool rendering;
- theme switch where supported;
- artifact cards with Open actions;
- no claim of a guaranteed reserved right column;
- truthful badges showing other running/waiting/blocked sessions.

### 4.2 Focusa-enhanced Pi distribution

The preferred Pi distribution must expose generic composable dock regions:

```text
work_surface_strip
left_sidebar
right_sidebar
above_editor
below_transcript
modal_layer
detail_pane
split_pane
```

Conceptual contract:

```ts
ui.registerDockContribution({
  id: "focusa.mission-canvas.sidebar",
  region: "right_sidebar",
  order: 10,
  render: renderFocusaMissionCanvasSidebar,
});
```

The dock contract must provide:

- stable cross-extension ordering;
- Work Surface identity and focus routing;
- width and split negotiation;
- minimum terminal breakpoints;
- overlay fallback;
- keyboard and mouse behavior;
- targeted invalidation;
- cleanup on session/project switch;
- session inventory and unread indicators;
- explicit steering/follow-up target selection.

Focusa and queue-steering extensions must not install competing editor wrappers.

### 4.3 UIAI Engine Cockpit integration and Focusa Mission Canvas projection

The established **UIAI Engine Cockpit** in `apps/cockpit/` is the single primary rich desktop operator environment for browser execution, FPV, Test Lab, Documents, research, artifacts, diagnostics, and audited operator control. It may run or interact with Pi through SDK/RPC and host Focusa Mission Canvas and professional-workspace projections.

The product and authority boundary is explicit:

```text
UIAI Engine Cockpit
  Operator shell, orchestration, visualization, browser-facing controls,
  rich documents, research, testing, FPV, and artifact inspection.

UIAI Engine
  Browser/search/session/media/diagnostics execution and stable artifact handles.

Focusa
  ProjectIdentity, C.R.I.S.T., semantic project truth, Role, Trajectory,
  Workpoints, Work Rail, authority, Evidence, Receipts, history, recovery,
  session attachments, and next-safe-action cognition.

Pi
  Conversational agent interaction, coding-agent execution, tools,
  steering, and follow-up delivery.
```

The Focusa-enhanced Pi distribution is the primary terminal and coding-harness experience. Focusa may also expose the same Mission Canvas and professional-workspace projections through its standalone Mission Deck PWA, native TUI, Pi sidebar, and menubar.

Every surface must consume the same generated contracts and bounded read models. Hosting a Focusa projection inside UIAI Engine Cockpit does not transfer canonical Focusa authority to UIAI Engine Cockpit or UIAI Engine.

---

## 5. Workspace View Profile

```yaml
schema: focusa.workspace_view_profile.v1
id: software-engineering
version: 1
compatibility_version: 1
label: Software Engineering
extends:
  - focusa.base

domain_packs:
  required:
    - focusa.core.cognition@1
    - focusa.software@1
  optional: []
  missing_pack_behavior: explicit_degraded_card

layout:
  sidebar_width: 42
  sidebar_sections:
    - current_work
    - next_work
    - worktree
    - verification
    - change_summary
    - sessions
    - contention
    - history
    - controls
  home_canvas: software_mission_canvas
  default_detail_view: code_diff
  work_surface_policy: multiplexed

terminology:
  project: Repository
  work_item: Task
  evidence: Proof
  change_set: Diff
  work_surface: Work Surface

visual:
  theme_id: focusa-software-violet
  icon_pack: engineering
  geometry: dense_grid
  density: compact
  motion_profile: execution_progress

artifact_renderers:
  change_set: code_diff
  document: technical_document
  evidence_set: test_and_ci
  image: visual_qa_capture

history_projection:
  include:
    - task_transition
    - workpoint
    - evidence
    - test
    - build
    - provider_closure
    - instance_session_attachment
    - proposal_resolution
```

### 5.1 Resolution order

```text
focusa.base
→ vertical profile
→ optional domain overlay
→ project workspace defaults and constraints
→ user Mission Canvas presentation/accessibility overrides
→ runtime capability fallback
```

### 5.2 Composite profiles

Supported examples:

- software + compliance;
- legal + research;
- markets + research;
- design + software;
- healthcare + compliance.

The system must not force a project into one mutually exclusive vertical.

### 5.3 Workspace profile and domain-pack boundary

A Workspace View Profile controls layout, terminology, visual grammar, renderer selection, and emphasis. It may require or recommend domain packs, but it must not redefine their object, relation, action, verification, status, or promotion semantics.

Resolution must preserve this order:

```text
project-selected domain packs
→ registered semantic contracts
→ bounded read model
→ workspace profile projection
→ Mission Canvas / client renderer
```

An unavailable required domain pack produces an explicit degraded workspace state with migration/recovery guidance. It must not silently fall back to visually convincing but semantically incorrect controls.

### 5.4 Project and user layout ownership

The project owns the shared semantic and team baseline:

- active Workspace View Profile;
- required domain packs;
- required/non-hideable authority, proof, safety, and compliance panels;
- default panel set and order;
- terminology;
- artifact-renderer bindings;
- team-shared workspace overrides.

The user owns personal Mission Canvas presentation preferences:

- open, pinned, grouped, and split Work Surfaces;
- focused Work Surface;
- visual variant;
- density;
- sidebar width and dock position;
- collapsed/expanded state;
- optional panel ordering where the project permits it;
- keyboard shortcuts;
- accessibility and reduced-motion settings;
- device-specific layout overrides.

Project-required safety, authority, proof, and compliance surfaces take precedence over personal hiding or reordering. Runtime capability fallbacks are temporary projections and must not overwrite project defaults or user preferences.

### 5.5 Work Surface projection boundary

Work Surface definitions and lifecycle are governed by Spec 135G. A Workspace View Profile may select default renderers, panels, and grouping behavior for a Work Surface, but it must not create or redefine Instance, Session, Attachment, Silent Session, UIAI session, browser context, browser target, writer lease, or Workpoint authority.

---

## 6. Dynamic registries

Required registries:

```text
PanelRegistry
HomeCanvasRegistry
WorkSurfaceRendererRegistry
WorkSurfaceActionRegistry
ArtifactRendererRegistry
ActionRegistry
TerminologyRegistry
ThemeRegistry
IconRegistry
HistoryProjectionRegistry
WorkspaceProfileRegistry
DomainSemanticBindingRegistry
SessionKindPresentationRegistry
```

Forbidden architecture:

```ts
if (workspace === "legal") renderEntireLegalApplication();
```

Required architecture:

```text
Workspace manifest
→ domain and session contracts
→ resolver
→ Work Surfaces, panels, and canvas
→ renderers
→ terminology
→ theme tokens
→ resolved projection
```

Unknown panel or renderer IDs must produce explicit degraded-state cards and profile migration warnings rather than crashes or silent omissions. Unknown semantic type, action, domain-pack, verification-policy, slice-policy, session-kind, or attachment-role IDs must remain visible as unsupported references and may not be reinterpreted by client-local logic.

---

## 7. Theme system

### 7.1 Theme tokens

```yaml
theme:
  background:
  surface:
  elevated_surface:
  border:
  primary_text:
  secondary_text:
  accent:
  focus:
  selection:
  success:
  warning:
  error:
  blocked:
  stale:
  advisory:
  canonical:
  active_session:
  waiting_session:
  conflict:
  shared_context:
  isolated_context:
```

### 7.2 Invariant safety semantics

Across all themes:

- error remains recognizably error;
- warning remains warning;
- blocked cannot resemble success;
- canonical/advisory/stale state must remain distinct;
- active, waiting, conflicted, shared-context, and isolated-context states remain distinguishable;
- color cannot be the only status signal;
- reduced-motion and high-contrast variants must exist.

### 7.3 Independent switches

```text
Workspace
● Legal
○ Software
○ Markets
○ Research

Visual variant
● Legal Obsidian
○ Legal Parchment
○ High Contrast
○ Monochrome
```

Workspace changes projection. Visual variant changes appearance. Neither silently changes operational authority or session binding.

---

## 8. Sidebar modes

Default compact sections:

```text
Project
Workspace
Focused Work Surface
Active sessions
Current Workpoint
Current work
Next work
Proof status
C.R.I.S.T. status
Contention
Recent history
Controls
```

Focused modes:

```text
Now
Work
Sessions
Contention
Proof
Research
History
Context
Role
Interview
Spec
Controls
```

The sidebar must provide progressive disclosure: one obvious primary action with deeper detail available without hiding the complete capability set.

### 8.1 Session inventory

The session view groups and filters:

- projects;
- workstreams;
- Instances;
- Sessions;
- Attachments;
- Pi foreground sessions;
- Silent Sessions and runs;
- UIAI sessions, browser contexts, and targets;
- health and lifecycle;
- pending input, approvals, conflicts, and writer leases.

It must never imply that the focused tab is the only running session.

---

## 9. Work Rail

### 9.1 Required states

```text
○ READY
▶ ACTIVE
◐ VERIFYING
! PROOF MISSING
↗ RECONCILING
✓ VERIFIED COMPLETE
! PROVIDER CLOSED / FOCUSA UNVERIFIED
⊘ CANCELLED
```

Strike-through is allowed only for `VERIFIED COMPLETE`.

### 9.2 Required row model

```yaml
work_item:
  provider:
  provider_item_id:
  title:
  provider_status:
  focusa_status:
  workpoint_id:
  project_root:
  continuity_id:
  instance_id:
  session_id:
  attachment_id:
  work_surface_ids: []
  priority:
  rank:
  dependencies: []
  blockers: []
  evidence_refs: []
  artifact_refs: []
  change_set_ref:
  receipt_ref:
  closure_claim_ref:
  updated_at:
```

### 9.3 Interaction

A selected row supports:

- open or focus related Work Surface;
- open Workpoint;
- open provider item;
- inspect evidence;
- inspect change artifact;
- inspect Receipt;
- steer an explicit active attachment;
- defer;
- request approval;
- reopen;
- inspect history;
- inspect session origin and contention;
- copy stable reference.

All mutations must use typed preview/commit actions and existing authority rules.

### 9.4 Provider capability truth

The UI must distinguish:

```text
operational
read-only
credentials missing
unhealthy
adapter unavailable
schema-only support
approval required
```

An enum is not an adapter.

### 9.5 Rail modes

```text
Surface-local
  Focused Work Surface and primary Attachment.

Project aggregate
  All work under one verified ProjectRootKey.

Cross-project advisory
  Labeled read-only aggregation with no implicit mutation target.
```

---

## 10. Multiplexed Work Surface behavior

Spec 135G is authoritative for the complete schema and lifecycle. This spec requires the visual implementation to support:

- tab strip and keyboard switcher;
- project/workstream/session grouping;
- pinned and unread surfaces;
- horizontal and vertical splits;
- side-by-side comparison;
- suspend and rehydrate;
- close-view versus terminate-session distinction;
- session-local and aggregate Work Rail;
- per-surface inspector;
- contention/proposal indicators;
- writer lease/worktree indicators;
- browser-context isolation badges;
- explicit steering and follow-up recipients.

Work Surface tabs are active work objects, not static application-module navigation.

---

## 11. Artifact projection

The same canonical change artifact may render as:

```text
Software
→ unified code diff

Legal
→ side-by-side redline

Markets
→ thesis revision with changed assumptions

Research
→ claim delta with evidence changes
```

Every renderer remains traceable to the same artifact, before/after refs, evidence, scope, session origin, and freshness.

---

## 12. Vertical visual grammars

### 12.1 Software Engineering

- cobalt/electric-violet identity;
- dense technical grid;
- monospace-heavy artifacts;
- branch/file/test iconography;
- change markers;
- test/build/CI meters;
- code diff as primary change view;
- worktree, deployments, session writers, and dependency graph.

### 12.2 Legal

- deep navy, burgundy, or parchment-gold identity;
- formal document hierarchy;
- strong rules and spacious reading surfaces;
- citation, exhibit, docket, deadline, confidentiality, and privilege indicators;
- redline as primary change view;
- matter timeline, authority table, claim/issue map, and source-session provenance.

### 12.3 Markets

- emerald, cyan, or amber identity;
- data-dense horizontal bands;
- sparklines, timestamps, freshness, catalysts, confidence, exposure, and contrary evidence;
- thesis revision as primary change view;
- research-only authority by default;
- explicit data/browser-session freshness.

### 12.4 Research

- indigo/teal identity;
- notebook and graph composition;
- source cards, claim/evidence links, contradiction indicators, reading queue, and provenance trails;
- research synthesis and claim delta as primary views;
- multiple source/research sessions visible without merging origin.

### 12.5 General

- neutral Focusa living-field identity;
- Mission, Workpoint, Evidence, C.R.I.S.T., sessions, and next safe action;
- no domain-specific assumptions.

### 12.6 Custom

- profile builder using registered panels/renderers/tokens;
- schema validation;
- preview;
- project/user ownership selection;
- export/import;
- migration support;
- Work Surface defaults without redefining runtime identities.

---

## 13. Dynamic update model

```text
Focusa/UIAI/provider/session event
→ validate project/workstream/origin identity
→ invalidate named read-model keys
→ bounded refetch
→ rerender affected Work Surfaces and panels
```

Do not push whole project state or large artifacts through every event.

Required invalidation examples:

```text
mission_canvas.open_surfaces
mission_canvas.surface_detail
mission_canvas.session_inventory
mission_canvas.contention
workspace.current
workspace.sidebar
workspace.history
workspace.artifacts
work_items.timeline
workpoint.current
crist.progress
connectors.health
```

Polling remains a reconnect/degraded fallback, not the primary live mechanism.

---

## 14. Responsive and accessibility requirements

Required:

- desktop persistent Work Surface strip and sidebar;
- compact/narrow session switcher and drawer;
- stacked terminal layout;
- keyboard-only Work Surface navigation;
- clear focus indicators;
- accessible labels for session kind/state/isolation;
- color-independent states;
- high contrast;
- reduced motion;
- motion reserved for genuine activity/state change;
- minimum touch target support in PWA/UIAI Engine Cockpit;
- virtualized long session/artifact lists;
- restored focus after modal closure;
- no loss of running-session visibility on narrow clients.

---

## 15. Client parity

All clients consume shared contracts:

- stock Pi compatibility widgets/drawer and session switcher;
- enhanced Pi Mission Canvas docks/sidebar;
- Mission Deck PWA;
- UIAI Engine Cockpit Tauri shell hosting Focusa Mission Canvas projections;
- menubar peeks/living field;
- native Ratatui TUI;
- CLI and JSON read models.

The UIAI Engine Cockpit is the primary rich desktop surface. The Focusa-enhanced Pi distribution is the primary terminal/harness-native surface. The menubar remains compact and ambient under Spec 121A and must not expose the entire Mission Canvas as equal application tabs.

---

## 16. Acceptance criteria

Spec 135A is accepted when:

1. Workspace profiles are versioned, schema-validated, inherited, and migratable.
2. General, Software, Legal, Markets, Research, Custom, and composite profiles operate.
3. Workspace and visual-variant switching are independent.
4. The same active projects/workstreams recompose without losing mission, queues, sessions, or Work Surfaces.
5. A real persistent Mission Canvas sidebar and Work Surface switcher operate in the Focusa-enhanced Pi distribution.
6. Stock Pi receives a truthful compatibility layout and active-session inventory.
7. Work Rail states update live and strike through only verified completion.
8. Provider and session capability truth is displayed.
9. Artifact renderer dispatch works by profile and artifact kind.
10. All verticals are visually striking beyond color changes.
11. Responsive, keyboard, reduced-motion, and high-contrast tests pass.
12. PWA, UIAI Engine Cockpit, Pi, menubar, native TUI, API, and CLI use shared contracts.
13. Visual selection cannot escalate permissions.
14. Actual visual-regression and live-update evidence is captured.
15. Every workspace profile declares its domain-pack requirements or explicitly declares itself presentation-only.
16. No Pi, PWA, UIAI Engine Cockpit, menubar, TUI, or renderer implementation owns or duplicates canonical domain policy.
17. Project-owned workspace defaults and user-owned Mission Canvas preferences resolve deterministically and preserve non-hideable authority/proof/safety panels.
18. Multiple Work Surfaces remain open, active, and independently attributable.
19. Focused Work Surface state never becomes singleton canonical authority.
20. Work Surface close, pause, and terminate actions have distinct behavior and previews.
21. Surface-local, project aggregate, and cross-project advisory Work Rail modes are proven.
22. Steering/follow-up recipient selection and broadcast preview are proven.
23. Session, contention, writer-lease, and browser-isolation indicators are visible and accurate.
24. The word Cockpit is used only for UIAI Engine Cockpit.

---

## 17. Closure blockers

This spec cannot close while:

- Pi lacks a stable Mission Canvas dock/sidebar contract;
- Focusa and UIAI extensions use incompatible Pi roots;
- a required profile is color-only or mock-only;
- a required renderer is absent;
- Work Rail completion is provider-only or model-claim-only;
- normal updates require manual refresh;
- narrow-terminal or accessibility states are unproven;
- shared client contracts are replaced by duplicated local interfaces;
- a visual profile silently substitutes for a missing domain pack;
- a client or renderer embeds canonical domain policy instead of consuming generated semantic contracts;
- project and user layout ownership is ambiguous or a personal override can hide required authority/proof/safety state;
- the UI assumes one global active session;
- Work Surface focus can change canonical project/session authority;
- multiple sessions cannot be inventoried, switched, split, or rehydrated;
- steering can route implicitly to the wrong attachment;
- generic Focusa/Pi surfaces are named Cockpit.

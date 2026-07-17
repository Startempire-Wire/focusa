# Spec 135A — Workspace Projection, Pi Sidebar, Work Rail, Themes, and Vertical UX

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135A.  
**Scope:** Workspace View Profiles, dynamic panel/layout/rendering registries, Pi integration and enhanced distribution, Work Rail, steering/follow-up composition, themes, professional verticals, artifact viewers, history, accessibility, responsive behavior, and client parity.

---

## 0. One-line definition

Focusa should project one canonical project runtime into a dynamic, project-selectable professional cockpit that stays visible and live inside Pi and other Focusa clients, while each vertical becomes visually striking through layout, artifact views, terminology, geometry, density, iconography, and theme — not through color alone.

---

## 1. Normative basis

This spec instantiates:

- [Projection and View Semantics](75-projection-and-view-semantics.md);
- [Visual/UI Ontology Core](58-visual-ui-ontology-core.md);
- [Visual/UI Evidence and Workflow](62-visual-ui-evidence-and-workflow.md);
- [Visual/UI Focusa Integration](65-visual-ui-focusa-integration.md);
- [Spec 117](117-mission-deck-onboarding-recall-pwa-spec.md) and [117A](117a-living-mission-field-pwa-spec.md);
- [Spec 119](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md);
- [Spec 121](121-menubar-rearchitecture-spec.md) and [121A](121a-menubar-discipline-and-living-field-spec.md);
- [Spec 124](124-focusa-cli-redesign-project-dashboard-project-creation-scoped-authority-first-mission-command-hierarchy-and-launch-hardening-spec.md);
- [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md).

The workspace is a projection over canonical state. It is not a new authority or memory store.

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
- is pinned to a different Pi package line than UIAI’s extension.

The implementation must therefore add a shared dynamic UI substrate and converge the Pi extension package/runtime before the full sidebar is considered operational.

---

## 3. Experience architecture

### 3.1 Preferred desktop composition

```text
┌──────────────────────── PI SESSION ───────────────────────┬──── FOCUSA ────┐
│ Conversation, tools, streaming activity                  │ Project         │
│                                                          │ Workspace       │
│                                                          │ Current work    │
│                                                          │ Next work       │
│                                                          │ Proof           │
│                                                          │ Context/Role    │
│                                                          │ History         │
│                                                          │ Controls        │
├──────────────────────────────────────────────────────────┴─────────────────┤
│ FOCUSA WORK RAIL                                                           │
├────────────────────────────────────────────────────────────────────────────┤
│ STEERING QUEUE                                                             │
├────────────────────────────────────────────────────────────────────────────┤
│ FOLLOW-UP QUEUE                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│ PROMPT EDITOR                                                              │
└────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Lane definitions

```text
Work Rail
  Canonical/advisory project work projection.

Steering Queue
  Operator instructions delivered at the next safe active-turn boundary.

Follow-up Queue
  Operator instructions delivered after the current agent run.

Prompt Editor
  New immediate user input.
```

A task is not steering. Steering is not a task. Follow-up is not an upcoming provider work item.

---

## 4. Pi capability levels

### 4.1 Compatibility Mode

For unmodified Pi:

- Work Rail widget above the editor;
- Focusa drawer or modal;
- custom footer/status;
- custom message/tool rendering;
- theme switch where supported;
- artifact cards with Open actions;
- no claim of a guaranteed reserved right column.

### 4.2 Focusa-enhanced Pi distribution

The preferred Pi distribution must expose generic composable dock regions:

```text
left_sidebar
right_sidebar
above_editor
below_transcript
modal_layer
detail_pane
```

Conceptual contract:

```ts
ui.registerDockContribution({
  id: "focusa.workspace.sidebar",
  region: "right_sidebar",
  order: 10,
  render: renderFocusaSidebar,
});
```

The dock contract must provide:

- stable cross-extension ordering;
- focus routing;
- width negotiation;
- minimum terminal breakpoints;
- overlay fallback;
- keyboard and mouse behavior;
- invalidation;
- cleanup on session/project switch.

Focusa and queue-steering extensions must not install competing editor wrappers.

### 4.3 Focusa Cockpit

A Focusa-owned PWA/Tauri outer shell may run Pi through SDK/RPC and fully control transcript, editor, sidebar, images, charts, FPV, task rail, and modal panes.

It must consume the same contracts and read models as Pi mode.

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

layout:
  sidebar_width: 42
  sidebar_sections:
    - current_work
    - next_work
    - worktree
    - verification
    - change_summary
    - history
    - controls
  home_canvas: software_mission_canvas
  default_detail_view: code_diff

terminology:
  project: Repository
  work_item: Task
  evidence: Proof
  change_set: Diff

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
```

### 5.1 Resolution order

```text
focusa.base
→ vertical profile
→ optional domain overlay
→ project overrides
→ user accessibility preferences
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

---

## 6. Dynamic registries

Required registries:

```text
PanelRegistry
HomeCanvasRegistry
ArtifactRendererRegistry
ActionRegistry
TerminologyRegistry
ThemeRegistry
IconRegistry
HistoryProjectionRegistry
WorkspaceProfileRegistry
```

Forbidden architecture:

```ts
if (workspace === "legal") renderEntireLegalApplication();
```

Required architecture:

```text
Workspace manifest
→ resolver
→ panels and canvas
→ renderers
→ terminology
→ theme tokens
→ resolved projection
```

Unknown panel or renderer IDs must produce explicit degraded-state cards and profile migration warnings rather than crashes or silent omissions.

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
```

### 7.2 Invariant safety semantics

Across all themes:

- error remains recognizably error;
- warning remains warning;
- blocked cannot resemble success;
- canonical/advisory/stale state must remain distinct;
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

Workspace changes projection. Visual variant changes appearance. Neither silently changes operational authority.

---

## 8. Sidebar modes

Default compact sections:

```text
Project
Workspace
Agent state
Current Workpoint
Current work
Next work
Proof status
C.R.I.S.T. status
Recent history
Controls
```

Focused modes:

```text
Now
Work
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

- open Workpoint;
- open provider item;
- inspect evidence;
- inspect change artifact;
- inspect Receipt;
- steer active work;
- defer;
- request approval;
- reopen;
- inspect history;
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

---

## 10. Artifact projection

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

Every renderer remains traceable to the same artifact, before/after refs, evidence, scope, and freshness.

---

## 11. Vertical visual grammars

### 11.1 Software Engineering

- cobalt/electric-violet identity;
- dense technical grid;
- monospace-heavy artifacts;
- branch/file/test iconography;
- change markers;
- test/build/CI meters;
- code diff as primary change view;
- worktree, deployments, and dependency graph.

### 11.2 Legal

- deep navy, burgundy, or parchment-gold identity;
- formal document hierarchy;
- strong rules and spacious reading surfaces;
- citation, exhibit, docket, deadline, confidentiality, and privilege indicators;
- redline as primary change view;
- matter timeline, authority table, claim/issue map.

### 11.3 Markets

- emerald, cyan, or amber identity;
- data-dense horizontal bands;
- sparklines, timestamps, freshness, catalysts, confidence, exposure, and contrary evidence;
- thesis revision as primary change view;
- research-only authority by default.

### 11.4 Research

- indigo/teal identity;
- notebook and graph composition;
- source cards, claim/evidence links, contradiction indicators, reading queue, and provenance trails;
- research synthesis and claim delta as primary views.

### 11.5 General

- neutral Focusa living-field identity;
- Mission, Workpoint, Evidence, C.R.I.S.T., and next safe action;
- no domain-specific assumptions.

### 11.6 Custom

- profile builder using registered panels/renderers/tokens;
- schema validation;
- preview;
- project/user ownership selection;
- export/import;
- migration support.

---

## 12. Dynamic update model

```text
Focusa/UIAI/provider event
→ invalidate named read-model keys
→ bounded refetch
→ rerender affected panels
```

Do not push whole project state or large artifacts through every event.

Required invalidation examples:

```text
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

## 13. Responsive and accessibility requirements

Required:

- desktop persistent sidebar;
- compact/narrow drawer;
- stacked terminal layout;
- keyboard-only navigation;
- clear focus indicators;
- accessible labels;
- color-independent states;
- high contrast;
- reduced motion;
- motion reserved for genuine activity/state change;
- minimum touch target support in PWA/Tauri;
- virtualized long lists;
- restored focus after modal closure.

---

## 14. Client parity

All clients consume shared contracts:

- stock Pi compatibility widgets/drawer;
- enhanced Pi docks/sidebar;
- Mission Deck PWA;
- Tauri shell;
- menubar peeks/living field;
- native Ratatui TUI;
- CLI and JSON read models.

The menubar remains compact and ambient under Spec 121A. It must not expose the entire professional cockpit as equal tabs.

---

## 15. Acceptance criteria

Spec 135A is accepted when:

1. Workspace profiles are versioned, schema-validated, inherited, and migratable.
2. General, Software, Legal, Markets, Research, Custom, and composite profiles operate.
3. Workspace and visual-variant switching are independent.
4. The same active project/session recomposes without losing mission or queues.
5. A real persistent sidebar operates in the Focusa-enhanced Pi distribution.
6. Stock Pi receives a truthful compatibility layout.
7. Work Rail states update live and strike through only verified completion.
8. Provider capability truth is displayed.
9. Artifact renderer dispatch works by profile and artifact kind.
10. All verticals are visually striking beyond color changes.
11. Responsive, keyboard, reduced-motion, and high-contrast tests pass.
12. PWA, Tauri, Pi, menubar, native TUI, API, and CLI use shared contracts.
13. Visual selection cannot escalate permissions.
14. Actual visual-regression and live-update evidence is captured.

---

## 16. Closure blockers

This spec cannot close while:

- Pi lacks a stable dock/sidebar contract;
- Focusa and UIAI extensions use incompatible Pi roots;
- a required profile is color-only or mock-only;
- a required renderer is absent;
- Work Rail completion is provider-only or model-claim-only;
- normal updates require manual refresh;
- narrow-terminal or accessibility states are unproven;
- shared client contracts are replaced by duplicated local interfaces.

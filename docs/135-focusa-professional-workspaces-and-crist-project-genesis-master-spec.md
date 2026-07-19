# Spec 135 — Focusa Professional Workspaces and C.R.I.S.T. Project Genesis Master Spec

**Status:** draft, iterable, NOT FINAL — operator approval is required before decomposition or implementation  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Scope:** Focusa core, daemon API, ProjectIdentity, Context Cognition, Workpoints, Trajectory, Evidence, Receipts, provider-neutral work items, Spec Workbench, Mission Deck, Mission Canvas, multiplexed Work Surfaces, Pi integration/distribution, native TUI, PWA/Tauri surfaces, menubar, UIAI Engine, context-source adapters, project onboarding, workspace projection, domain-semantic composition, typed ontology registries, domain packs, browser-context isolation, themes, professional verticals, history, artifacts, and governed controls.  
**Series:** Spec 135, 135A, 135B, 135C, 135D, 135E, 135F, and 135G form one required implementation and closure set.  
**Numbering:** `134` is already occupied by the superseded Pre-MVP STG closure-evidence snapshot. No Spec 135 series existed when this document was created.

---

## 0. One-line definition

Focusa should provide one canonical, project-scoped agent runtime that can become a visually and operationally distinct professional workspace for software, legal, markets, research, and custom domains, with every project initialized and continuously refined through a governed C.R.I.S.T. process and every concurrent interaction presented through a multiplexed Focusa Mission Canvas:

```text
Context
→ Role
→ Interview
→ Spec
→ Tasks
→ Persistent Focusa Workspace
→ Multiplexed Mission Canvas
```

---

## 1. Product thesis

Focusa already contains domain-general primitives:

```text
ProjectIdentity
Continuity
Mission
Trajectory
Workpoint
Work item
Action intent
Constraint
Risk
Blocker
Evidence
Verification
History
Receipt
Authority
Approval
Recovery
```

A coding repository, legal matter, market-research program, and general research project do not require separate cognitive runtimes. They require purpose-specific projections of the same governed truth:

- different terminology;
- different layouts;
- different visual grammars;
- different artifact renderers;
- different evidence emphasis;
- different role profiles;
- different interview questions;
- different provider integrations;
- different operational policies.

The canonical Focusa runtime remains stable. The professional workspace changes around it.

The same architecture must support many projects, workstreams, interactive sessions, autonomous sessions, browser contexts, targets, devices, and harnesses simultaneously. Removing singleton authority is therefore both an integrity correction and a product-enabling multiplexing requirement.

This directly instantiates [Projection and View Semantics](75-projection-and-view-semantics.md): canonical truth remains canonical; views are contextual; projections remain traceable to canonical state; and `switch_view_profile` changes representation without mutating truth.

---

## 2. Product promise

A user should be able to create or bind a project, complete a guided Project Genesis flow, and enter a living workspace that immediately shows:

```text
what this project is;
why it exists;
what context supports it;
what role the agent serves;
which questions remain unanswered;
which approved spec governs it;
which tasks follow from the spec;
what every attached session is doing now;
what comes next;
what evidence exists;
what remains unproven;
what authority permits or blocks action;
and what the next safe move is.
```

The same active Focusa project can switch among professional views without losing:

- mission;
- Workpoint;
- task state;
- steering queue;
- follow-up queue;
- evidence;
- history;
- project scope;
- operator authority;
- open Work Surfaces;
- attached sessions;
- browser-context identity;
- unread and contention state.

The user can keep multiple Work Surfaces open simultaneously, including Pi sessions, UIAI browser contexts, Silent Sessions, Documents, Research, evidence views, and project aggregates.

---

## 3. Series structure and authority

This master spec defines the product contract. Companion specs define required implementation detail.

| Spec | Required subject |
|---|---|
| [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Mission Canvas presentation, Pi sidebar/docks, Work Rail, themes, artifact views, vertical visual systems, and dynamic UX |
| [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | C.R.I.S.T. Project Genesis: Context, Role, Interview, Spec, and Tasks |
| [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI rich artifacts, screenshots, browser research, evidence handoff, FPV, browser identity, and live UI refresh |
| [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Complete implementation graph, no-deferral law, framework reuse, performance, multiplexing prerequisites, and decomposition rules |
| [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Cross-spec amendments, precedence, migration, compatibility, naming, and closure matrix |
| [135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Domain-general ontology core, semantic graph, domain packs, verification policies, slice policies, and reactive context |
| [135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Mission Canvas naming, multiplexed Work Surfaces, Instances/Sessions/Attachments, browser-context isolation, concurrency, restoration, and interaction routing |

Hard rule:

```text
No companion is optional for Spec 135 closure.
Dependency ordering does not remove a requirement from the implementation graph.
```

---

## 4. Normative basis

Spec 135 extends rather than replaces:

- [Spec 38](38-thread-thesis-spec.md) — cognitive workspaces and thread thesis.
- [Spec 39](39-thread-lifecycle-spec.md) — thread lifecycle.
- [Spec 40](40-instance-session-attachment-spec.md) — Instances, Sessions, Attachments, and multiplexing engineers.
- [Spec 41](41-proposal-resolution-engine.md) — asynchronous concurrent proposals and deterministic resolution.
- [Spec 43](43-multi-device-sync.md) — local-first multi-device synchronization.
- [Specs 45–50](45-ontology-overview.md) — ontology overview, primitives, software world, links/actions, working sets/slices, and reducer integration.
- [Spec 61](61-domain-general-cognition-core.md) — reusable domain-neutral cognition primitives.
- [Spec 70](70-shared-interfaces-statuses-and-lifecycle.md) — shared statuses and lifecycle semantics.
- [Spec 74](74-identity-and-reference-resolution.md) — canonical identity and alias resolution.
- [Spec 77](77-ontology-governance-versioning-and-migration.md) — ontology versions, compatibility, migration, deprecation, and governance.
- [Spec 75](75-projection-and-view-semantics.md) — Projection, ViewProfile, ProjectionRule, ProjectionBoundary.
- [Spec 72](72-agent-identity-role-and-self-model-ontology.md) — RoleProfile, CapabilityProfile, PermissionProfile, Responsibility, HandoffBoundary.
- [Spec 88](88-ontology-backed-workpoint-continuity.md) — typed Workpoint continuity rather than transcript-tail authority.
- [Spec 98](98-project-root-crdt-reconciliation-foundation-spec.md) — ProjectRootKey, WorkstreamKey, AttachmentKey, multi-session updates, and deterministic reconciliation.
- [Spec 100](100-context-cognition-spec.md) — bounded advisory context selection and exclusion.
- [Spec 104](104-typed-scoped-runtime-and-singleton-elimination-spec.md) — strict typed scope and eradication of authority-bearing singleton state.
- [Spec 107](107-spec-first-feature-lifecycle-and-claim-discipline-spec.md) — Idea → Spec → Tasks → Implementation → Proof → Closure.
- [Spec 109](109-agent-first-api-redesign-ax-spec.md) — typed, bounded, versioned, discoverable, preview/commit Agent Experience contracts.
- [Spec 111](111-agent-context-bootstrap-and-delivery-spec.md) — bounded project context delivery and bootstrap receipts.
- [Spec 116](116-provider-neutral-work-item-closure-authority-spec.md) — provider-neutral work items and closure truth.
- [Spec 117](117-mission-deck-onboarding-recall-pwa-spec.md) and [117A](117a-living-mission-field-pwa-spec.md) — Mission Deck, onboarding, Recall, PWA, and living experience language.
- [Spec 119](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md) — proof-backed work Receipts and governed execution ledger.
- [Spec 120](120-adversarial-spec-workbench-and-operator-approval-gates.md) — reality-grounded adversarial spec creation and task decomposition.
- [Spec 121](121-menubar-rearchitecture-spec.md) and [121A](121a-menubar-discipline-and-living-field-spec.md) — typed Svelte/Tauri surfaces and living-field experience posture.
- [Spec 124](124-focusa-cli-redesign-project-dashboard-project-creation-scoped-authority-first-mission-command-hierarchy-and-launch-hardening-spec.md) — project creation, templates, settings, First Mission, and project selection semantics.
- [Spec 125](125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md) — mandatory Trajectory and HLT authority.
- [Spec 130](130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md) — bounded context continuity and compaction mission packets.
- [Spec 133](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md) — durable governed execution, runs, leases, worktree isolation, and daemon-native sessions.
- UIAI Engine’s `UIAI_FOCUSA_PI_HAND_IN_GLOVE_SPEC.md` — UIAI as browser/research/proof execution plane; Focusa as cognitive continuity and authority plane.

Any mismatch between this series and current code is an implementation gap, not permission to weaken this series.

---

## 5. Current codebase reality

### 5.1 Implemented foundations

The repository already contains substantial foundations:

- Rust daemon, API, CLI, TUI, menubar, and Pi extension.
- Project listing, discovery, creation, templates, settings, binding, switching, and status.
- Project-scoped Pi configuration and immediate live config replacement.
- Workpoints carrying project root, continuity, mission, next slice, work-item ID, blockers, verification, and evidence refs.
- Trajectory and mandatory HLT direction.
- Evidence/ECS handles and event-chain persistence.
- Focusa SSE event streaming.
- Provider-neutral work-item types.
- A concrete Beads adapter.
- UIAI browser sessions, screenshots, reads, snapshots, diagnostics, research packets, evidence handles, and FPV streams.
- SvelteKit 2, Svelte 5, Vite, static adapter, and Tauri 2 in `apps/menubar`.
- typed scoped runtime and anti-singleton evidence under Specs 98 and 104.
- durable Silent Session/run concepts, writer leases, and worktree isolation under Spec 133.

### 5.2 Partial foundations

The repository currently has partial rather than complete support for:

- profile-style settings in Pi, but not canonical Workspace View Profiles;
- First Mission, but not C.R.I.S.T. Project Genesis;
- fixed TUI tabs and hard-coded theme values, but not a dynamic workspace/panel/theme registry;
- UIAI artifact creation, but not rich insertion into the Focusa/Pi workspace;
- provider-neutral schemas, but not complete Linear, Asana, GitHub Issues, and Markdown adapters;
- written Spec 119/120/117/121 product direction, but not all corresponding runtime surfaces;
- reducer-backed ontology state, events, projections, and Pi context injection, but not a core-owned typed registry, strict candidate/canonical graph separation, policy-backed promotion, or domain-pack composition;
- scoped session identities, but not a complete multiplexed Mission Canvas, Work Surface, browser-container, and targeted-interaction experience.

### 5.3 Planned foundations

The following are required by this series and remain implementation work:

- domain-general ontology registry, candidate/canonical semantic graphs, verification ledger, domain packs, generalized slice policies, semantic subscriptions, and V1 compatibility projection;
- canonical workspace selection and profile history;
- persistent Pi sidebar/dock composition;
- Focusa Mission Canvas and versioned Work Surfaces;
- multiplexed Instance/Session/Attachment inventory and interaction routing;
- isolated UIAI browser contexts and target/tab identity;
- dynamic vertical layouts and themes;
- C.R.I.S.T. state and Project Genesis profile;
- source-linked context ingestion and connectors;
- AI Role Composer and operator approval;
- persistent dynamic interview compendium;
- operational Spec 120 Workbench integration;
- complete provider-neutral task materialization;
- rich UIAI artifact bridge;
- Receipt-backed Work Rail history;
- parity across Pi, PWA, UIAI Engine Cockpit, menubar, native TUI, API, and CLI.

The UI must display this capability truth and must not present docs-only or enum-only support as operational.

---

## 6. Four-layer architecture

```text
┌──────────────────────────────────────────────┐
│ Workspace Projection / Mission Canvas       │
│ Layout · Work Surfaces · theme · renderers  │
├──────────────────────────────────────────────┤
│ Project Operating Profile                   │
│ Context · role · interview · spec · tasks   │
├──────────────────────────────────────────────┤
│ Domain Semantic Composition                 │
│ Shared cognition · domain packs · policies  │
│ typed objects/links/actions · bounded slices │
├──────────────────────────────────────────────┤
│ Canonical Focusa Runtime                    │
│ Identity · reducer · Workpoint · proof      │
│ authority · sessions · history · recovery   │
└──────────────────────────────────────────────┘
```

### 6.1 Workspace Projection / Mission Canvas Layer

Controls what is visible and emphasized:

- workspace and visual profile;
- Work Surface tabs, panes, grouping, and focus;
- sidebar sections;
- home canvas;
- terminology;
- theme tokens;
- iconography;
- density;
- motion;
- artifact renderers;
- history filters;
- default detail views;
- aggregate versus surface-local presentation.

It is not action authority. Keyboard focus on a Work Surface is not a global active session or canonical project pointer.

### 6.2 Project Operating Profile

References the project’s:

- context sources and accepted claims;
- approved agent role;
- interview corpus;
- approved Project Genesis Spec;
- task-decomposition plan;
- evidence profile;
- operational policy;
- connectors;
- selected workspace;
- current Trajectory and Workpoints;
- attached sessions and Work Surfaces as projections.

### 6.3 Domain Semantic Composition

Defines and resolves the project’s semantic working world without creating another runtime:

- shared cognition primitives;
- active, versioned domain packs;
- typed object, relation, action, status, verification, and slice-policy definitions;
- candidate and canonical semantic graphs;
- evidence-backed promotion policies;
- ontology-derived Workpoint candidates;
- semantic delta subscriptions.

Domain Semantic Composition runs through the existing Focusa reducer, persistence, authority, and event systems. A Workspace View Profile may render or emphasize domain semantics, but it does not define canonical truth or grant permission.

### 6.4 Canonical Focusa Runtime

Continues to own:

- ProjectIdentity;
- explicit project root;
- continuity ID;
- ProjectRootKey, WorkstreamKey, and AttachmentKey;
- Instance, Session, and Attachment records;
- Trajectory;
- Workpoints;
- Evidence;
- Context Authority;
- provider closure truth;
- Receipts;
- proposal resolution and writer leases;
- event history;
- operator precedence.

---

## 7. Required terminology separation

The implementation must keep these distinct:

### Selected Project Profile

Existing non-authoritative CLI convenience pointer identifying the recently selected project.

### Project Genesis Profile

Versioned output of C.R.I.S.T.: context, role, interview, approved spec, task plan, and initial operating posture.

### Workspace View Profile

Purpose-specific visual and interaction projection: General, Software, Legal, Markets, Research, Custom, or composite.

### Focusa Mission Canvas

The multiplexed interactive Focusa workspace projection shown inside Pi, Mission Deck, native TUI, menubar expansions, or UIAI Engine Cockpit.

### Work Surface

One visible tab, pane, split, or detached window in a Mission Canvas, projecting one primary Attachment and related supporting objects.

### UIAI Engine Cockpit

The only product/surface in this series that uses the term Cockpit: UIAI Engine’s companion rich desktop shell.

### Domain Pack

A versioned semantic package defining domain-specific object, relation, action, status, verification, artifact-interpretation, and slice-policy contracts over the shared cognition core. A domain pack is not a workspace, role, permission, or evidence profile.

### Agent Role Profile

The expert function and responsibilities the agent is expected to serve.

### Operational Policy Profile

What the agent is actually permitted to do.

### Evidence Profile

What proof is required for claims and closure.

No one object may silently substitute for another.

---

## 8. Required product experience

### 8.1 Main Mission Canvas composition

Preferred full layout:

```text
┌ WORK SURFACES ──────────────────────────────────────────────────────────────┐
│ [Overview] [Pi · task 23] [UIAI · admin] [Silent · tests] [Research]      │
├──────────────────────── FOCUSED WORK SURFACE ──────────────┬──── FOCUSA ──┤
│ Conversation, tool activity, browser, document, or result │ Project       │
│                                                           │ Workspace     │
│                                                           │ Session       │
│                                                           │ Current work  │
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

The Work Rail, Steering Queue, Follow-up Queue, and Prompt Editor are separate lanes and must not be conflated. Multiple sessions may remain active while one Work Surface has keyboard focus.

### 8.2 C.R.I.S.T. genesis progress

During Project Genesis, the workspace must visibly show:

```text
C Context    source and claim readiness
R Role       draft / review / approved
I Interview  answered / open / blocker questions
S Spec       Workbench and approval progress
T Tasks      plan / approval / materialization
```

After genesis, this remains available as a Project Profile panel and may be opened in one or more Work Surfaces.

### 8.3 One-click workspace switching

A project may switch among installed workspaces through a radio/select surface. Switching must preserve the current project, mission, Workpoint, task, evidence, history, queues, sessions, Work Surfaces, and browser-context identity.

Appearance changes immediately when safe. Governing-rule changes apply only through their own approval and policy paths.

### 8.4 Multiplexed session interaction

The operator can:

- open many Work Surfaces across one or many projects;
- group by project, workstream, or session kind;
- split and compare surfaces;
- keep Pi, Silent Session, UIAI browser, document, and research work active together;
- route steering/follow-up to explicit attachments;
- inspect contention, proposal resolution, writer leases, and isolation;
- close a view without implicitly terminating its underlying session;
- rehydrate the Mission Canvas after client restart.

---

## 9. Hard design laws

### 9.1 One runtime, many projections

Do not create separate Focusa cores for verticals or sessions.

### 9.2 Visual selection is not authority escalation

Selecting Markets must not grant trading authority. Selecting Legal must not grant filing authority. Selecting Software must not grant production-deployment authority.

### 9.3 Context is not a prompt blob

Project context is source-linked, indexed, versioned, and bounded through Context Cognition. It is never indiscriminately inserted into every prompt.

### 9.4 Role is not permission

RoleProfile, CapabilityProfile, PermissionProfile, and HandoffBoundary remain distinct.

### 9.5 Interview answers remain operator-owned

AI may summarize, challenge, and propose changes, but may not silently overwrite answers.

### 9.6 Spec authority remains operator-gated

C.R.I.S.T. uses Spec 120; it does not self-approve specifications.

### 9.7 Tasks remain provider-neutral

C.R.I.S.T. uses Spec 120 decomposition, Spec 116 provider adapters and closure authority, Workpoints for execution, and Spec 119 Receipts.

### 9.8 UIAI remains execution/proof plane

UIAI creates browser/search/research/media/diagnostic artifacts. Focusa captures, links, evaluates, and governs their meaning.

### 9.9 Real-time UI is invalidation-based

Events carry references and invalidation keys. Clients refetch bounded read models rather than receiving full state or large artifacts in every event.

### 9.10 No proof, no verified done

Provider closure, agent claims, or UI state alone cannot create verified completion.

### 9.11 Operator steering wins

The operator may pause, redirect, reopen, revise, or stop workspace, C.R.I.S.T., spec, task, and work-loop activity.

### 9.12 Workspace projection is not domain semantics

Workspace profiles select presentation and emphasis. Domain packs define semantic contracts. Neither workspace selection nor domain-pack activation grants operational authority, bypasses verification, or replaces the Workpoint reducer path.

### 9.13 Focused Work Surface is not singleton authority

`focused_work_surface_id`, selected tab, current pane, current browser target, and visible project are client projection state only. They may not become daemon-global project, Workpoint, session, task, or browser-context authority.

### 9.14 Cockpit naming is reserved

The word Cockpit is reserved for UIAI Engine Cockpit. Focusa/Pi interactive workspace projections are Mission Canvas; their tabs and panes are Work Surfaces.

---

## 10. Required workspace families

The complete accepted workspace set is:

- General;
- Software Engineering;
- Legal;
- Markets;
- Research;
- Custom;
- composite profiles combining multiple workspaces.

Each must include more than a color variation:

- visual grammar;
- home canvas;
- panel hierarchy;
- terminology;
- iconography;
- density;
- artifact renderers;
- history projection;
- evidence presentation;
- C.R.I.S.T. presentation;
- Mission Canvas and Work Surface behavior;
- controls;
- empty, blocked, stale, degraded, and recovery states;
- sample/demo project.

---

## 11. Required client surfaces

The complete implementation graph includes:

- stock Pi compatibility mode;
- Focusa-enhanced Pi distribution with Mission Canvas sidebar/dock composition;
- Focusa Mission Deck PWA;
- UIAI Engine Cockpit Tauri desktop shell hosting Focusa Mission Canvas and professional-workspace projections;
- compact menubar;
- native Focusa TUI;
- API;
- CLI;
- headless/RPC/JSON parity.

The UIAI Engine Cockpit is the primary rich desktop surface. The Focusa-enhanced Pi distribution is the primary terminal/harness-native surface. Focusa does not create a second independently branded desktop product that competes with UIAI Engine Cockpit.

Different clients may use different renderers, but they must share generated contracts, semantic type and action IDs, status enums, domain-pack manifests, verification and slice-policy IDs, workspace manifests, Mission Canvas and Work Surface schemas, Instance/Session/Attachment identities, theme tokens, artifact descriptors, semantic delta envelopes, and invalidation events.

---

## 12. C.R.I.T. attribution and C.R.I.S.T. modification

C.R.I.T. is attributed to Geoff Woods and stands for Context, Role, Interview, Task. The Interview step has the AI ask the operator questions one at a time before producing the requested output.

Spec 135 modifies the pattern for Focusa’s governed, spec-first lifecycle:

```text
C.R.I.T. inspiration:
Context → Role → Interview → Task

Focusa C.R.I.S.T.:
Context → Role → Interview → Spec → Tasks
```

The modification is required because Focusa’s lifecycle mandates a durable approved spec and provider-neutral task decomposition before implementation.

The permanent public feature name is **Focusa C.R.I.S.T. Project Genesis**. Product UI may use **Project Genesis** as the compact label while retaining the five C.R.I.S.T. stages. On first use in public architecture or educational documentation, spell out Context, Role, Interview, Spec, and Tasks and attribute the underlying C.R.I.T. inspiration to Geoff Woods. Do not imply endorsement or represent C.R.I.S.T. as Geoff Woods’ published framework.

Reference:

- AI Leadership, “CRIT Happens: The Viral Framework Transforming How Leaders Think” — `https://www.aileadership.com/newsletter/crit-happens-the-viral-framework-transforming-how-leaders-think`

---

## 13. Complete implementation posture

This series is not an MVP-selection document.

Its implementation order exists to:

- establish contracts before clients;
- parallelize safe work;
- prevent duplicate systems;
- integrate existing Focusa/UIAI/Pi foundations;
- preserve performance;
- create actual proof.

It does not authorize omission or indefinite deferral.

The controlling rules live in Spec 135D.

---

## 14. Master acceptance criteria

Spec 135 is accepted only when all companion acceptance criteria are proven and the following are true:

1. A project can complete C.R.I.S.T. and produce a versioned Project Genesis Profile.
2. Context is source-linked, incrementally synchronized, searchable, bounded, and impact-assessed.
3. An AI-composed Role Profile is grounded, editable, versioned, and operator-approved.
4. Interview questions are dynamic, one-at-a-time, persistent, amendable, and resumable.
5. C.R.I.S.T. launches an operational Spec 120 Workbench and produces an approved Project Genesis Spec.
6. The approved spec decomposes through a provider-neutral preview/approval/materialization path.
7. The first selected task becomes a scoped Workpoint.
8. The Work Rail updates in real time and only strikes through verified completion.
9. UIAI screenshots, research, diagnostics, snapshots, data, and FPV state enter the workspace through stable artifacts and evidence links.
10. Workspace selection can recompose an active project without losing mission, session, or Work Surface state.
11. Every accepted workspace is visually and functionally distinct.
12. Appearance switching cannot grant authority.
13. Pi, PWA, UIAI Engine Cockpit, menubar, native TUI, API, CLI, and headless surfaces consume shared contracts.
14. All required framework, security, migration, performance, accessibility, and release-proof criteria pass.
15. No required Complete Feature Ledger entry remains missing, partial, open, or blocked.
16. A core-owned, versioned domain-semantic substrate supports General, Software, Legal, Markets, Research, Custom, and composite projects without duplicating the Focusa runtime.
17. Existing ontology state, snapshots, events, routes, Pi behavior, and clients migrate through a proven V1 compatibility projection with unknown-event preservation and downgrade-write protection.
18. Multiple projects, workstreams, Pi sessions, Silent Sessions, UIAI sessions, browser contexts, and browser targets operate simultaneously without state bleed.
19. Mission Canvas and Work Surface schemas remain projections over ProjectRootKey, WorkstreamKey, AttachmentKey, and runtime-specific identities.
20. Browser-context isolation, explicit shared-context posture, session-targeted steering, writer leases, conflict resolution, and restart rehydration are proven.
21. The word Cockpit is used only for UIAI Engine Cockpit in the Spec 135 series and implementation labels.

---

## 15. Closure policy

Spec 135 and its companions cannot close while:

- an accepted connector is missing;
- a workspace is mock-only or color-only;
- a required vertical has only visual terminology and lacks an operational domain pack or truthful degraded-state behavior;
- candidate semantic state can become canonical without its registered verification and promotion policy;
- a provider exists only as an enum;
- the Spec Workbench is only a document generator;
- interview state exists only in transcript text;
- role approval is unversioned;
- screenshots or rich artifacts cannot reach the workspace;
- normal updates require manual refresh;
- Pi and UIAI remain on incompatible extension package roots;
- a required client is absent from the implementation graph;
- Mission Canvas assumes one global active session;
- browser context and browser target are conflated;
- session-origin identity is absent from artifacts/events;
- a generic Focusa/Pi surface is named Cockpit;
- an acceptance-critical proof is partial or surrogate;
- a companion spec remains open.

Final closure requires:

```text
all required feature-ledger entries verified
+ complete cross-client and multi-session integration proof
+ operator acceptance
+ provider reconciliation where applicable
+ stable Evidence and Receipt references
```

---

## 16. Decomposition directive

When Spec 120 decomposes this series, the implementing agent must treat Spec 135, 135A, 135B, 135C, 135D, 135E, 135F, and 135G as one dependency graph.

The agent must not produce a small backend MVP and assign the professional workspace, connectors, rich artifacts, verticals, Mission Canvas, multiplexed sessions, browser isolation, PWA/UIAI Engine Cockpit integration, provider adapters, or proof requirements to an unspecified future date.

The exact no-deferral and decomposition language in Spec 135D is normative.

---

## 17. Resolved operator decisions

The following decisions are final for this specification revision and are no longer approval blockers.

### 17.1 Public name and attribution

The permanent public feature name is **Focusa C.R.I.S.T. Project Genesis**. Compact UI labels may use **Project Genesis**. Public architecture and educational documentation attribute C.R.I.T. inspiration to Geoff Woods on first use and clearly identify C.R.I.S.T. as Focusa’s modification.

### 17.2 Canonical storage boundary

Canonical Project Genesis, workspace selection, context claims, Role Profiles, interview records, Spec Workbench refs, task-plan refs, domain-pack bindings, semantic state, Instance/Session/Attachment state, and artifact links live in the local Focusa node’s reducer-backed SQLite event/state system. Large source and evidence payloads live behind ECS/content-addressed handles.

Mission Canvas layout and Work Surface presentation preferences are user/device projections. Their attachment references are durable and rehydratable, but visual focus is not canonical project/session authority.

Project files are explicit, versioned import/export or collaboration projections only. `.focusa-project.json` remains a ProjectIdentity marker and does not become a Project Genesis database. `.pi/settings.json` may mirror bootstrap/package/theme choices for Pi but is not canonical. An approved spec may be written to the repository only through the existing operator-gated Spec 120 repo-write path. Raw connected documents and email are not copied into the repository by default.

Focusa Cloud may provide opt-in encrypted synchronization, relay, hosted Receipts, and team projections. It does not silently become canonical project-semantic authority. Multi-node reconciliation follows existing ProjectIdentity/CRDT and authority rules.

### 17.3 Selected ingestion and retrieval frameworks

Document execution belongs to the UIAI Engine Documents subsystem. The selected structured extraction engine is **Docling**, running locally in a persistent isolated worker/service for PDF, Office, email, image, table, layout, and OCR-aware conversion. Plain text, Markdown, code, JSON, and CSV use direct bounded ingestion without a second document-conversion framework.

The selected local hybrid retrieval stack is:

```text
SQLite FTS5
+ sqlite-vec behind a pinned versioned adapter
+ fastembed-rs for local ONNX embeddings and reranking
```

The baseline embedding profile is versioned and begins with `BAAI/bge-small-en-v1.5`; projects may select another registered compatible embedding profile without changing the storage contract. Model ID, model revision, dimensions, chunking profile, and embedding generation version are persisted so re-embedding and migration are deterministic.

The selected Svelte UI foundations are **shadcn-svelte/Bits UI** for accessible generic primitives and **TanStack Query for Svelte** for server-state caching, mutation state, targeted invalidation, and bounded background refetch.

### 17.4 Canonical Pi namespace

The canonical Pi runtime, SDK, TUI, peer-dependency, and extension import namespace is `@earendil-works/pi-*`, pinned to one tested compatible release across Focusa, UIAI Engine, queue-steering, and UIAI Engine Cockpit. The legacy `@mariozechner/pi-*` dependency is migration input only.

The Focusa-enhanced Pi distribution is a packaged configuration over the upstream Earendil Pi runtime, not a separately renamed Pi fork. A public Focusa npm package name is chosen only after registry-scope ownership is verified and does not reopen the runtime-namespace decision.

### 17.5 Layout ownership

The project owns the shared workspace semantic baseline:

- active Workspace View Profile;
- required domain packs;
- required/non-hideable authority, proof, and compliance panels;
- default panel set and order;
- terminology and artifact-renderer bindings;
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

Project-required safety and authority surfaces take precedence over personal hiding/reordering. Runtime responsive fallbacks are temporary capability projections and do not overwrite either project or user preferences.

### 17.6 Connector reference implementation

**Google Drive** is the first reference connector. It must prove the complete connector contract: minimum-scope OAuth, account and folder scoping, bounded initial import, Google-native document export, revisions, permissions/provenance, changes-feed cursoring, incremental synchronization, cursor recovery, health, revocation, and ProjectContextDelta emission.

The Google OAuth/account substrate is then reused by Gmail. OneDrive, SharePoint, Outlook/Microsoft mail, task providers, and the remaining accepted connectors stay in the same required implementation graph; choosing Google Drive first does not defer or remove them.

### 17.7 Primary rich desktop surface and naming

The **UIAI Engine Cockpit** in `apps/cockpit/` is the single primary rich desktop operating environment. It hosts Focusa Mission Canvas and professional-workspace projections alongside browser/FPV, Test Lab, Documents, Research, artifacts, diagnostics, and audited operator controls.

The Focusa-enhanced Pi distribution is the primary terminal and coding-harness experience. Focusa Mission Deck PWA, native TUI, and menubar remain required portable/compact Focusa surfaces.

**Cockpit** is reserved exclusively for UIAI Engine Cockpit. The Focusa/Pi interactive overlay/workspace is **Focusa Mission Canvas**, and its tabs/panes are **Work Surfaces**. Focusa does not create a separate independently branded desktop shell that competes with UIAI Engine Cockpit.

These decisions must be reflected in companion specs and decomposition. Reopening one requires an explicit versioned operator amendment rather than an implementation-time substitution.

# Spec 135 — Focusa Professional Workspaces and C.R.I.S.T. Project Genesis Master Spec

**Status:** draft, iterable, NOT FINAL — operator approval is required before decomposition or implementation  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Scope:** Focusa core, daemon API, ProjectIdentity, Context Cognition, Workpoints, Trajectory, Evidence, Receipts, provider-neutral work items, Spec Workbench, Mission Deck, Pi integration/distribution, native TUI, PWA/Tauri surfaces, menubar, UIAI Engine, context-source adapters, project onboarding, workspace projection, themes, professional verticals, history, artifacts, and governed controls.  
**Series:** Spec 135, 135A, 135B, 135C, 135D, and 135E form one required implementation and closure set.  
**Numbering:** `134` is already occupied by the superseded Pre-MVP STG closure-evidence snapshot. No Spec 135 series existed when this document was created.

---

## 0. One-line definition

Focusa should provide one canonical, project-scoped agent runtime that can become a visually and operationally distinct professional workspace for software, legal, markets, research, and custom domains, with every project initialized and continuously refined through a governed C.R.I.S.T. process:

```text
Context
→ Role
→ Interview
→ Spec
→ Tasks
→ Persistent Focusa Workspace
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
what the agent is doing now;
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
- operator authority.

---

## 3. Series structure and authority

This master spec defines the product contract. Companion specs define required implementation detail.

| Spec | Required subject |
|---|---|
| [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Pi sidebar/docks, Work Rail, themes, artifact views, vertical visual systems, and dynamic UX |
| [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | C.R.I.S.T. Project Genesis: Context, Role, Interview, Spec, and Tasks |
| [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI rich artifacts, screenshots, browser research, evidence handoff, FPV, and live UI refresh |
| [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Complete implementation graph, no-deferral law, framework reuse, performance, and decomposition rules |
| [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Cross-spec amendments, precedence, migration, compatibility, and closure matrix |

Hard rule:

```text
No companion is optional for Spec 135 closure.
Dependency ordering does not remove a requirement from the implementation graph.
```

---

## 4. Normative basis

Spec 135 extends rather than replaces:

- [Spec 75](75-projection-and-view-semantics.md) — Projection, ViewProfile, ProjectionRule, ProjectionBoundary.
- [Spec 72](72-agent-identity-role-and-self-model-ontology.md) — RoleProfile, CapabilityProfile, PermissionProfile, Responsibility, HandoffBoundary.
- [Spec 88](88-ontology-backed-workpoint-continuity.md) — typed Workpoint continuity rather than transcript-tail authority.
- [Spec 100](100-context-cognition-spec.md) — bounded advisory context selection and exclusion.
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
- [Spec 133](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md) — durable governed execution and daemon-native sessions.
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

### 5.2 Partial foundations

The repository currently has partial rather than complete support for:

- profile-style settings in Pi, but not canonical Workspace View Profiles;
- First Mission, but not C.R.I.S.T. Project Genesis;
- fixed TUI tabs and hard-coded theme values, but not a dynamic workspace/panel/theme registry;
- UIAI artifact creation, but not rich insertion into the Focusa/Pi workspace;
- provider-neutral schemas, but not complete Linear, Asana, GitHub Issues, and Markdown adapters;
- written Spec 119/120/117/121 product direction, but not all corresponding runtime surfaces.

### 5.3 Planned foundations

The following are required by this series and remain implementation work:

- canonical workspace selection and profile history;
- persistent Pi sidebar/dock composition;
- dynamic vertical layouts and themes;
- C.R.I.S.T. state and Project Genesis profile;
- source-linked context ingestion and connectors;
- AI Role Composer and operator approval;
- persistent dynamic interview compendium;
- operational Spec 120 Workbench integration;
- complete provider-neutral task materialization;
- rich UIAI artifact bridge;
- Receipt-backed Work Rail history;
- parity across Pi, PWA, Tauri, menubar, native TUI, API, and CLI.

The UI must display this capability truth and must not present docs-only or enum-only support as operational.

---

## 6. Three-layer architecture

```text
┌──────────────────────────────────────────────┐
│ Workspace Projection Layer                  │
│ Layout · theme · terminology · renderers    │
├──────────────────────────────────────────────┤
│ Project Operating Profile                   │
│ Context · role · interview · spec · tasks   │
├──────────────────────────────────────────────┤
│ Canonical Focusa Runtime                    │
│ Identity · trajectory · Workpoint · proof   │
│ authority · history · receipts · recovery   │
└──────────────────────────────────────────────┘
```

### 6.1 Workspace Projection Layer

Controls what is visible and emphasized:

- workspace and visual profile;
- sidebar sections;
- home canvas;
- terminology;
- theme tokens;
- iconography;
- density;
- motion;
- artifact renderers;
- history filters;
- default detail views.

It is not action authority.

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
- current Trajectory and Workpoint.

### 6.3 Canonical Focusa Runtime

Continues to own:

- ProjectIdentity;
- explicit project root;
- continuity ID;
- Trajectory;
- Workpoint;
- Evidence;
- Context Authority;
- provider closure truth;
- Receipts;
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

### Agent Role Profile

The expert function and responsibilities the agent is expected to serve.

### Operational Policy Profile

What the agent is actually permitted to do.

### Evidence Profile

What proof is required for claims and closure.

No one object may silently substitute for another.

---

## 8. Required product experience

### 8.1 Main session composition

Preferred full layout:

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

The Work Rail, Steering Queue, Follow-up Queue, and Prompt Editor are separate lanes and must not be conflated.

### 8.2 C.R.I.S.T. genesis progress

During Project Genesis, the workspace must visibly show:

```text
C Context    source and claim readiness
R Role       draft / review / approved
I Interview  answered / open / blocker questions
S Spec       Workbench and approval progress
T Tasks      plan / approval / materialization
```

After genesis, this remains available as a Project Profile panel.

### 8.3 One-click workspace switching

A project may switch among installed workspaces through a radio/select surface. Switching must preserve the current project, mission, Workpoint, task, evidence, history, and queues.

Appearance changes immediately when safe. Governing-rule changes apply only through their own approval and policy paths.

---

## 9. Hard design laws

### 9.1 One runtime, many projections

Do not create separate Focusa cores for verticals.

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
- controls;
- empty, blocked, stale, degraded, and recovery states;
- sample/demo project.

---

## 11. Required client surfaces

The complete implementation graph includes:

- stock Pi compatibility mode;
- Focusa-enhanced Pi distribution with dock/sidebar composition;
- Focusa Mission Deck PWA;
- Tauri desktop shell;
- compact menubar;
- native Focusa TUI;
- API;
- CLI;
- headless/RPC/JSON parity.

Different clients may use different renderers, but they must share generated contracts, action IDs, status enums, workspace manifests, theme tokens, artifact descriptors, and invalidation events.

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

This series must describe C.R.I.S.T. as Focusa’s adaptation inspired by C.R.I.T.; it must not imply Geoff Woods’ endorsement or that C.R.I.S.T. is his published framework.

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
10. Workspace selection can recompose the active project without losing mission state.
11. Every accepted workspace is visually and functionally distinct.
12. Appearance switching cannot grant authority.
13. Pi, PWA, Tauri, menubar, native TUI, API, CLI, and headless surfaces consume shared contracts.
14. All required framework, security, migration, performance, accessibility, and release-proof criteria pass.
15. No required Complete Feature Ledger entry remains missing, partial, open, or blocked.

---

## 15. Closure policy

Spec 135 and its companions cannot close while:

- an accepted connector is missing;
- a workspace is mock-only or color-only;
- a provider exists only as an enum;
- the Spec Workbench is only a document generator;
- interview state exists only in transcript text;
- role approval is unversioned;
- screenshots or rich artifacts cannot reach the workspace;
- normal updates require manual refresh;
- Pi and UIAI remain on incompatible extension package roots;
- a required client is absent from the implementation graph;
- an acceptance-critical proof is partial or surrogate;
- a companion spec remains open.

Final closure requires:

```text
all required feature-ledger entries verified
+ complete cross-client integration proof
+ operator acceptance
+ provider reconciliation where applicable
+ stable Evidence and Receipt references
```

---

## 16. Decomposition directive

When Spec 120 decomposes this series, the implementing agent must treat Spec 135, 135A, 135B, 135C, 135D, and 135E as one dependency graph.

The agent must not produce a small backend MVP and assign the professional workspace, connectors, rich artifacts, verticals, PWA/Tauri, provider adapters, or proof requirements to an unspecified future date.

The exact no-deferral and decomposition language in Spec 135D is normative.

---

## 17. Open operator decisions before final approval

1. Confirm the permanent public name `C.R.I.S.T.` and attribution language.
2. Confirm canonical versus project-file storage boundaries for Project Genesis records.
3. Approve the document-extraction and semantic-index framework qualification result required by Spec 135D.
4. Confirm the canonical Pi distribution/package namespace.
5. Confirm project-level versus user-level ownership of visual layout preferences.
6. Confirm the first connector reference implementation while retaining the full accepted connector graph.
7. Confirm whether the Focusa-enhanced Pi distribution or Focusa Cockpit becomes the primary rich desktop experience.

These are approval blockers, not permission to omit their affected requirements.

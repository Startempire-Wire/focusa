# Spec 135 — Focusa Professional Workspaces and C.R.I.S.T. Project Genesis Master Spec

**Status:** draft, iterable, NOT FINAL — operator approval required before decomposition or implementation  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Scope:** Focusa core, daemon API, ProjectIdentity, Context Cognition, Workpoints, Trajectory, Evidence, Receipts, provider-neutral work items, Spec Workbench, Mission Deck, Mission Canvas, multiplexed Work Surfaces, Pi, native TUI, menubar, PWA surfaces, UIAI Engine Cockpit, connectors, onboarding, generated UI, UXP/UFI, domain packs, browser isolation, themes, verticals, artifacts, history, and governed controls.  
**Series:** Spec 135 and 135A–135K form one required implementation and closure set. The series is frozen at 135K.  
**Delivery authority:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) resolves implementation, framework, testing, sequencing, and compatibility conflicts.

---

## 0. One-line definition

Focusa provides one canonical project-scoped agent runtime that becomes visually and operationally distinct professional workspaces while every project is initialized and continuously refined through a governed, real-time, nontechnical C.R.I.S.T. process and every concurrent interaction is presented through a multiplexed Focusa Mission Canvas.

```text
Context
→ Role
→ Interview
→ Spec
→ Tasks
→ persistent professional workspace
→ multiplexed Mission Canvas
```

---

## 1. Product thesis

Focusa already contains or specifies domain-general primitives:

```text
ProjectIdentity
ProjectRootKey / WorkstreamKey / AttachmentKey
Continuity
Mission
Trajectory
Workpoint
Work item
Context artifact and claim
Role Profile
Interview record
Action intent
Constraint
Risk
Blocker
Evidence
Verification
History
Receipt
Capability
Permission
Authority
Approval
Recovery
Instance / Session / Attachment
```

Software, legal, markets, research, and other professional domains MUST share this runtime. They differ through versioned projections, terminology, layouts, visual grammar, artifact renderers, evidence emphasis, role profiles, Interview strategy overlays, provider integrations, and operational policies.

The canonical runtime remains stable. Workspace and domain projections change around it.

The same architecture MUST support many projects, workstreams, Pi sessions, autonomous sessions, UIAI browser contexts, browser targets, devices, and harnesses simultaneously. Removing singleton authority is a product-enabling multiplexing requirement.

---

## 2. Product promise

A nontechnical operator can create or bind a project, complete Project Genesis through real-time generated UI, and enter a living workspace that shows:

```text
what the project is
why it exists
what context supports it
what role Focusa serves
which questions remain
which approved spec governs work
which tasks follow from the spec
what every attached session is doing
what comes next
what evidence exists
what remains unproven
what authority permits or blocks action
what the next safe move is
```

The same project can switch among professional Workspace View Profiles without losing mission, Trajectory, Workpoint, task state, steering, follow-up, Evidence, history, scope, authority, open Work Surfaces, session identity, browser context, unread state, or contention state.

Multiple Work Surfaces remain open simultaneously, including Pi sessions, UIAI browser contexts, Silent Sessions, Documents, Research, Evidence, and project aggregates.

---

## 3. Required series

| Spec | Required subject |
|---|---|
| [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Mission Canvas, Work Rail, themes, artifact views, and vertical UX |
| [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | C.R.I.S.T. Project Genesis: Context, Role, Interview, Spec, Tasks |
| [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI rich artifacts, browser research, Evidence handoff, FPV, and live refresh |
| [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Complete implementation graph, framework reuse, performance, and no-deferral law |
| [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Cross-spec amendments, precedence, migration, compatibility, and closure |
| [135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Ontology core, semantic graphs, domain packs, verification, and reactive context |
| [135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Mission Canvas, Work Surfaces, sessions, attachments, browser isolation, concurrency, and restoration |
| [135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md) | Grill Interview, Cross-Functional Alpha, fixed OSS stack, and speed law |
| [135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md) | Real-time generated C.R.I.S.T. UI, A2UI, typed actions, and nontechnical onboarding |
| [135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md) | Operation Registry, durable stream, shared envelopes, and runtime reuse |
| [135K](135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md) | UXP/UFI adaptation, friction learning, and nontechnical usability proof |

Hard rule:

```text
No companion is optional.
Dependency ordering does not remove a requirement.
The Delivery Contract governs conflicting implementation wording.
```

---

## 4. Normative basis

This series extends rather than replaces:

- Specs 38–41 for threads, lifecycle, Instances/Sessions/Attachments, and concurrent proposal resolution;
- Spec 43 for local-first multi-device synchronization;
- Specs 45–50, 61, 70, 72, 74, 75, and 77 for ontology, shared lifecycle, role, identity, projections, and governance;
- Spec 88 for ontology-backed Workpoint continuity;
- Specs 98 and 104 for project-root reconciliation, typed scope, and singleton elimination;
- Spec 100 for Context Cognition;
- Specs 107, 109, and 111 for spec-first lifecycle, agent-first API, and context delivery;
- Spec 116 for provider-neutral work and closure authority;
- Specs 117/117A for Mission Deck and living-field UX;
- Spec 119 for Receipts;
- Spec 120 for adversarial spec creation and task decomposition;
- Specs 121/121A for typed Svelte/Tauri surfaces and living-field discipline;
- Spec 124 for project creation and First Mission;
- Spec 125 for mandatory Trajectory and HLT;
- Spec 130 for bounded context and compaction;
- Spec 133 for governed durable sessions, runs, leases, and isolated worktrees;
- UIAI Engine companion specifications for browser, Documents, Eval, research, artifacts, diagnostics, FPV, and proof.

Any mismatch with current code is an implementation gap, not permission to weaken this series.

---

## 5. Current implementation truth

### Implemented foundations

- Rust daemon, API, CLI, TUI, menubar, and Pi extension;
- project discovery, creation, templates, settings, binding, switching, and status;
- project-scoped Pi configuration;
- Workpoints, Trajectory, Evidence/ECS handles, and event persistence;
- provider-neutral work-item types and Beads adapter;
- Focusa SSE live stream and canonical SQLite events;
- UIAI browser sessions, contexts, actions, screenshots, snapshots, diagnostics, research, artifacts, and FPV;
- SvelteKit/Svelte/Tauri UI foundations;
- typed scope and anti-singleton foundations;
- durable session, run, lease, and worktree concepts.

### Partial foundations

- settings profiles but no complete canonical Workspace View Profile runtime;
- First Mission but no complete C.R.I.S.T. runtime;
- fixed TUI tabs/themes but no dynamic registry;
- UIAI artifacts without complete Focusa rich projection;
- provider-neutral schema without all adapters;
- written Spec Workbench, Receipt, Mission Deck, and menubar direction without all runtime surfaces;
- scoped identities without complete Mission Canvas multiplexing;
- SSE live tail without durable reconnect/replay integration;
- API error envelopes with route-local duplication.

### Required implementation

- C.R.I.S.T. canonical state and Project Genesis profile;
- source-linked Context ingestion and connectors;
- Role Composer and approval;
- Grill Interview and persistent compendium;
- Spec 120 integration;
- provider-neutral task materialization;
- generated Operation Registry and generated clients;
- durable replayable UI event stream;
- A2UI web core, permanent Lit renderer, and Focusa Svelte Custom Elements;
- UXP/UFI runtime;
- rich UIAI artifact bridge and UIAI Engine Eval;
- dynamic Workspace View Profiles, themes, verticals, and domain packs;
- multiplexed Mission Canvas and browser-context isolation;
- Receipt-backed Work Rail history;
- parity across Pi, Mission Deck/PWA, UIAI Engine Cockpit, menubar, native TUI, API, and CLI.

The UI MUST display truthful capability state and MUST NOT present docs-only, enum-only, mock, or disconnected support as operational.

---

## 6. Architecture

```text
┌──────────────────────────────────────────────┐
│ Workspace Projection / Mission Canvas       │
│ Work Surfaces · layout · theme · renderers  │
├──────────────────────────────────────────────┤
│ Project Operating Profile                   │
│ Context · Role · Interview · Spec · Tasks   │
├──────────────────────────────────────────────┤
│ Domain Semantic Composition                 │
│ ontology · domain packs · verification      │
├──────────────────────────────────────────────┤
│ Canonical Focusa Runtime                    │
│ reducer · Workpoint · authority · Evidence  │
│ sessions · Receipts · history · recovery    │
└──────────────────────────────────────────────┘
```

### Workspace Projection / Mission Canvas

Controls presentation: Workspace View Profile, Work Surfaces, panels, home canvas, terminology, theme, icons, density, motion, renderers, history filters, and aggregate/local views.

It is not action authority. UI focus is not a canonical active project or session pointer.

### Project Operating Profile

Contains the approved Project Genesis state: Context sources and claims, Role Profile, Interview compendium, approved Spec, task plan, evidence policy, operational policy, and selected workspace.

### Domain Semantic Composition

Provides typed ontology objects, links, actions, candidate/canonical separation, domain packs, verification policy, reactive context, and bounded projections.

### Canonical runtime

Owns ProjectIdentity, exact scope, reducer state, Trajectory, Workpoints, tasks, capabilities, permissions, Evidence, Receipts, sessions, proposals, conflict resolution, recovery, and event history.

---

## 7. C.R.I.S.T. Project Genesis

```text
C — Context
  local and connected sources, UIAI research, Focusa history,
  provenance, claims, contradictions, and continuous growth

R — Role
  grounded AI draft, responsibilities, non-responsibilities,
  quality, evidence expectations, escalation, operator approval

I — Interview
  focusa.interview.strategy.grill-with-docs.v1,
  facts before questions, one decision at a time,
  recommendations and sources, branches, compendium, resume

S — Spec
  Spec 120 Project Genesis Workbench,
  reality scan, adversarial challenge, reconciliation, approvals

T — Tasks
  provider-neutral decomposition, preview, operator approval,
  materialization, Workpoint selection, closure authority
```

Every stage MUST be implemented as real-time generated plain-language UI. Project Genesis remains available after onboarding for continued Context, Role, Interview, Spec, and Task revision.

---

## 8. Fixed implementation decisions

The Delivery Contract locks:

- A2UI v0.9.1, `web_core`, permanent Lit renderer, and Focusa Svelte Custom Elements;
- native durable Focusa event replay plus broadcast tail;
- AG-UI as external compatibility after native stabilization;
- JSON Schema 2020-12 and OpenAPI 3.0.3;
- generated TypeScript and Go clients;
- Pi RPC AgentExecutionAdapter and Spec 133 sessions for model work;
- UIAI Engine Eval for all browser proof;
- no Playwright in Focusa;
- no Vercel AI SDK runtime ownership;
- Docling, FTS5, sqlite-vec, fastembed-rs, selected Svelte/TanStack stack, and selected connector/test/license tools;
- machine-readable delivery graph and greater Focusa primitive submission.

Agents MUST NOT reopen these decisions as option menus.

---

## 9. Implementation order

The full closure DAG contains every requirement. The Foundation Train and Cross-Functional Alpha establish the earliest integrated route.

```text
Foundation: contracts → generated clients → Operation Registry → shared envelopes
→ durable stream → capability projection → Pi execution adapter → A2UI/Lit
→ Focusa components → UIAI Eval → first real Context operation

Alpha: Context → Role/Interview → Spec/Task → Workpoint/Evidence/Receipt
→ UIAI artifact → multiplexing → vertical projection → permanent dogfood
```

After generated operation contracts stabilize, Context, Role/Interview, Spec/Tasks, Mission Canvas, UIAI, vertical, provider, and hardening lanes run concurrently in scoped worktrees with writer leases and exact Attachments.

---

## 10. Greater Focusa primitive law

Implementation order:

```text
general Focusa primitive
→ reducer and canonical state
→ typed Focusa API
→ generated TypeScript/Go contracts
→ C.R.I.S.T. projection
→ renderer
→ UIAI Engine Eval when browser-facing
→ Evidence
→ Receipt
```

General operation, capability, permission, preview/commit, replay, ToolResult, Evidence, Receipt, Context, Role, Interview, task, Workpoint, session, generated-surface, action-binding, UXP/UFI, health, and artifact behavior MUST NOT remain trapped in C.R.I.S.T. or client code.

---

## 11. Machine-readable decomposition

Before decomposition, create and validate:

```text
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Every normative requirement has a stable ID, owner, dependencies, tasks, primitives, operations, generated surfaces, client surfaces, UIAI Eval scenarios, tests, Evidence, Receipts, migrations, and closure state.

Agents MUST NOT infer the delivery DAG from prose alone.

---

## 12. Authority and safety laws

1. Project and session scope is explicit and typed.
2. Workspace selection never grants authority.
3. Role never grants permission.
4. Context claims retain provenance and promotion state.
5. Interview answers remain operator-owned and versioned.
6. Spec approval is operator-controlled.
7. Task-provider writes require approved preview/materialization.
8. UIAI artifacts are evidence candidates until Focusa captures and links them.
9. Generated UI actions resolve only through typed Focusa operations.
10. UXP/UFI changes presentation only.
11. Browser execution and browser proof remain in UIAI Engine.
12. Operator pause, redirect, reopen, and stop take precedence.

---

## 13. Permanent dogfood gate

```text
Onboarding
→ Context
→ Role
→ Grill Interview
→ Project Genesis Spec
→ Tasks
→ Workpoint
→ Evidence
→ Receipt
→ UIAI artifact
→ multiplexed Mission Canvas
→ pause
→ restart
→ exact resume
```

Every user-facing step is completed through generated nontechnical UI. Browser-facing proof uses UIAI Engine Eval exclusively.

---

## 14. Master acceptance criteria

Spec 135 is accepted when:

1. Every companion 135A–135K is implemented and verified.
2. Every C.R.I.S.T. stage operates through generated nontechnical UI.
3. Canonical Focusa primitives own all authority-bearing state.
4. Workspace profiles visibly distinguish verticals without duplicating runtimes.
5. Multiple sessions and browser contexts operate concurrently without bleed.
6. UIAI artifacts update relevant Work Surfaces automatically.
7. Work Rail completion is Evidence- and Receipt-backed.
8. Operation Registry, generated clients, capability projection, and durable replay are operational.
9. UXP/UFI adaptation is transparent and authority-neutral.
10. UIAI Engine Eval proves browser, responsive, visual, reconnect, accessibility, and isolation behavior.
11. No Playwright or duplicate browser-test runtime exists in Focusa.
12. No required behavior remains docs-only, mock-only, enum-only, CLI-only, static-card-only, or client-local.
13. Every machine-readable ledger requirement is verified.
14. The permanent dogfood traversal passes with actual Evidence and Receipts.

---

## 15. Closure blockers

Spec 135 cannot close while:

- any companion is incomplete;
- a C.R.I.S.T. stage lacks generated UI, recovery, or resume;
- a reusable primitive exists only inside C.R.I.S.T. or a client;
- a provider or connector is represented as operational without a working adapter;
- browser proof bypasses UIAI Engine Eval;
- Playwright exists in the Spec 135 implementation path;
- AG-UI replaces canonical Focusa state or blocks native Alpha delivery;
- a custom renderer duplicates A2UI web core or Lit;
- Vercel AI SDK or another framework duplicates Focusa/Pi authority;
- scope is ambient or singleton-derived;
- required Evidence, Receipts, migrations, compatibility, accessibility, security, performance, or client parity are missing;
- accepted requirements disappear from the machine-readable delivery graph.

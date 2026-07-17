# Spec 135D — Complete Implementation Order, Framework Reuse, Performance, and No-Deferral Constitution

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135D.  
**Scope:** complete feature ledger, decomposition law, dependency ordering, cross-spec ownership, framework qualification, code reuse, shared contracts, performance, parallel execution, UX completeness, proof discipline, and final closure.

---

## 0. One-line definition

Spec 135 must be implemented as one complete dependency graph: sequence by dependency, parallelize where safe, reuse and integrate existing systems, never silently omit or indefinitely defer accepted features, and never claim completion from stubs, schemas, mock providers, backend-only behavior, or surrogate proof.

---

## 1. Core implementation directive

```text
Sequence by dependency.
Parallelize where safe.
Integrate existing systems.
Do not omit.
Do not silently defer.
Do not declare partial infrastructure to be the completed product.
```

The implementation orders in this document are execution ordering. They do not split the series into “real now” and “maybe someday.”

Every accepted normative feature in Spec 135/135A/135B/135C/135D/135E must enter the implementation graph before implementation begins.

---

## 2. Sequencing, blocking, and deferral

### 2.1 Sequenced

A requirement is sequenced when:

- it has an implementation task;
- its dependencies are explicit;
- it has acceptance criteria;
- it has proof requirements;
- it remains inside the parent closure graph;
- work begins when prerequisites are satisfied.

### 2.2 Blocked

A requirement is blocked when:

- its task exists;
- blocker and owner are explicit;
- recovery path is explicit;
- it remains unfinished;
- it blocks parent closure unless the operator amends the spec.

### 2.3 Deferred

A requirement is deferred when it is removed from the active implementation graph and assigned to an unspecified future initiative.

Deferral is forbidden unless the operator creates a versioned amendment:

```yaml
removed_requirement:
  requirement_id:
  original_text:
  removal_reason:
  consequences:
  affected_acceptance_criteria: []
  operator_approval_ref:
  spec_revision:
```

### 2.4 Optional

Optional may describe a user runtime choice. It must not describe whether the implementation team may omit the feature.

Example:

```text
A user may choose not to connect Google Drive.
The accepted Google Drive connector requirement is not optional implementation work.
```

---

## 3. Forbidden planning language

Normative implementation work must not use these phrases as substitutes for tasks:

```text
later
eventually
future enhancement
post-MVP
nice to have
when time permits
out of scope for now
can be added afterward
phase two someday
optional implementation
```

Permitted form:

```text
Execution Order 7
Blocked by: Workspace Artifact contract and OAuth credential store
Required for parent closure: yes
```

---

## 4. Complete Feature Ledger

Before task materialization, every requirement becomes a ledger entry.

```yaml
schema: focusa.complete_feature_ledger.v1

spec_id: 135
spec_revision:

requirements:
  - requirement_id:
    title:
    normative_text_ref:
    implementation_status: missing | partial | implemented | verified
    owner:
    dependency_ids: []
    task_refs: []
    affected_surfaces:
      core: false
      api: false
      cli: false
      pi: false
      pwa: false
      tauri: false
      menubar: false
      native_tui: false
      uiai: false
      connector: false
    acceptance_criteria: []
    evidence_requirements: []
    closure_status: open | blocked | verified | operator_removed
```

The parent series cannot close while a required ledger entry is missing, partial, open, or blocked.

---

## 5. Cross-spec dependencies remain in the completion boundary

Different ownership does not mean deferred dependency.

Required dependencies from Specs 109, 111, 116, 117, 119, 120, 121, 124, 125, 130, and 133 remain explicit blockers in the Spec 135 graph until implemented and proven.

A task may be owned by its original spec and cross-linked to Spec 135. It may not disappear from the Spec 135 completion ledger.

---

## 6. Architectural reuse laws

### 6.1 One Focusa runtime

Do not create:

- a workspace daemon;
- a C.R.I.S.T. daemon;
- a connector-state daemon;
- a sidebar state service;
- a second task authority;
- a second evidence store;
- a second project-memory system;
- a second spec engine.

Use the existing Focusa runtime.

### 6.2 One canonical persistence path

Canonical workspace, Project Genesis, context claim, role, interview, task plan, and artifact-link events use Focusa reducers, event persistence, state snapshots, and event-chain discipline.

Project JSON, Pi globals, browser storage, Svelte stores, menubar state, UIAI state, and connector caches may project/cache state but may not become canonical truth.

### 6.3 One browser/research engine

UIAI owns browser/search/session/media/diagnostics execution. Focusa does not rebuild it.

### 6.4 One specification engine

C.R.I.S.T. `S` uses Spec 120. It does not create another Workbench, challenger, auditor, approval system, or spec-to-task engine.

### 6.5 One work-item/closure model

C.R.I.S.T. `T` uses Spec 120 decomposition, Spec 116 adapters/closure, Workpoints, and Spec 119 Receipts.

### 6.6 One shared UI contract

Pi, PWA, Tauri, menubar, and native TUI share:

- generated schemas/types;
- read models;
- action IDs;
- status enums;
- workspace manifests;
- theme tokens;
- artifact descriptors;
- invalidation events.

No client reimplements domain policy.

---

## 7. Required framework and stack reuse

### 7.1 Rust backend

Use the existing Rust workspace and conventions:

- `focusa-core` for domain types/reducers;
- `focusa-api` and Axum for HTTP/SSE;
- existing SQLite persistence/event chain;
- `focusa-cli` and Clap;
- `focusa-tui` / `focusa-terminal-ui` and Ratatui;
- ECS/content-addressed handles;
- Tokio for async work.

New domain modules should align with existing crate boundaries rather than become a separate service.

### 7.2 Schema generation

Spec 109 requires authoritative JSON Schema/OpenAPI.

Required flow:

```text
Rust/domain schemas
→ generated JSON Schema and OpenAPI
→ generated TypeScript contracts/client
→ Pi, Svelte, connector, and test consumers
```

Hand-maintained duplicate request/response interfaces are forbidden where generation can preserve the contract.

### 7.3 Shared rich UI stack

Use the existing Focusa frontend direction:

- SvelteKit 2;
- Svelte 5;
- Vite;
- static adapter;
- Tauri 2;
- shared design/workspace packages.

Required packages or equivalent workspace modules:

```text
focusa-contracts
  generated schemas and types

focusa-client
  API, auth, scope, mutation, SSE invalidation

focusa-design-system
  tokens, components, states, accessibility, motion

focusa-workspace-ui
  panel/layout/theme/renderer/profile registries
```

The PWA, Tauri app, and menubar must consume shared packages rather than copy screens.

### 7.4 UI primitives

Use established accessible Svelte-compatible primitives for generic dialogs, sheets, tabs, radio groups, forms, command palettes, menus, tooltips, and scroll areas.

Custom development should focus on Focusa-specific components:

- Mission Ladder;
- Work Rail;
- Evidence and Receipt cards;
- C.R.I.S.T. progress;
- Role redline;
- Interview compendium;
- Spec approval;
- vertical artifact viewers.

A component framework selection must be qualified for Svelte 5, Tailwind 4, accessibility, licensing, bundle size, and Tauri/static compatibility before the UI substrate task closes.

### 7.5 Server-state synchronization

Use a mature Svelte server-state query/cache framework or a documented equivalent that provides:

- typed queries;
- stale state;
- mutation state;
- targeted invalidation;
- background refetch;
- optimistic UI only where safe;
- SSR/static/Tauri compatibility.

The default candidate is TanStack Query for Svelte, subject to repository qualification.

Do not build a second normalized canonical client database.

### 7.6 Pi package convergence

Mandatory foundation work:

```text
choose canonical Pi distribution/package namespace
→ migrate Focusa extension
→ migrate UIAI extension
→ compatibility suite
→ shared dock, theme, image, and event contracts
```

Focusa and UIAI currently target different Pi package roots. No sidebar/editor/theme/artifact implementation may assume this mismatch can remain indefinitely.

### 7.7 Local document extraction qualification

Do not hand-write parsers for PDF, DOCX, PPTX, XLSX, email, images, and archives.

Before Context ingestion implementation, select a document extraction foundation through a recorded qualification matrix:

```text
license and commercial redistribution
supported formats
Rust/local integration
cross-platform packaging
binary/runtime footprint
memory behavior
text/table/image quality
OCR posture
sandboxability
security maintenance
batch performance
provenance/page mapping
```

Candidate classes include:

- a Rust-native extraction library/service;
- an isolated MIT-compatible converter adapter such as Microsoft MarkItDown;
- Apache Tika when its JVM/service footprint is justified.

The selection is a required Order 0 decision. It is not a future TODO.

### 7.8 Context retrieval

Use local SQLite lexical search, preferably FTS5, as the canonical local text-index baseline rather than a mandatory external search server.

Semantic retrieval remains required. Its embedding/vector implementation must be qualified for:

- local-first operation;
- cross-platform packaging;
- bounded memory;
- license compatibility;
- source linkage;
- deterministic filtering;
- no required cloud dependency.

Target retrieval:

```text
project/workstream scope
+ source and permission filter
+ freshness filter
+ lexical rank
+ semantic rank
+ provenance-preserving rerank
```

### 7.9 Connected-source synchronization

Use provider-native delta/change mechanisms rather than repeated full crawling.

Required connector shape:

```text
bounded initial import
→ persist provider cursor/subscription
→ notification or scheduled health check
→ fetch delta
→ normalize/deduplicate/index
→ emit ProjectContextDelta
→ recover expired cursor/subscription
```

Google, Microsoft, and mail connectors must implement their real incremental models and recovery paths.

---

## 8. Complete dependency-ordered build graph

Every order is required. Orders are not release-deferral buckets.

### Order 0 — Specification lock and complete decomposition

Required outputs:

1. approved Spec 135 series;
2. Complete Feature Ledger;
3. current-code Reality Pack;
4. cross-spec dependency graph;
5. framework qualification records;
6. canonical/projection map;
7. security/privacy model;
8. full parent/child task graph;
9. acceptance/proof mapping;
10. client parity matrix;
11. migration matrix;
12. open decisions resolved or explicit blockers.

### Order 1 — Contract and runtime convergence

- Pi package convergence;
- operation/schema/status IDs;
- Workspace Profile contracts;
- C.R.I.S.T./Project Genesis contracts;
- Context source/artifact/claim/delta contracts;
- Role/Interview contracts;
- Workspace Artifact contracts;
- task/read models;
- invalidation events;
- preview/commit/idempotency/version rules;
- generated OpenAPI/JSON Schema/TypeScript.

### Order 2 — Canonical state, reducer events, and read models

Suggested modules:

```text
focusa-core/src/workspace/
focusa-core/src/project_genesis/
focusa-core/src/project_context/
focusa-core/src/project_role/
focusa-core/src/project_interview/
focusa-core/src/workspace_artifact/
```

Required:

- reducers;
- SQLite persistence;
- event-chain participation;
- versions;
- scope enforcement;
- provenance;
- answer supersession;
- role revisions;
- workspace-selection history;
- context-claim lifecycle;
- bounded read models;
- SSE events.

No placeholder success routes.

### Order 3 — Shared dynamic UI substrate

- design tokens;
- state vocabulary;
- layout/panel/home-canvas registries;
- renderer/action/terminology/theme/icon/history registries;
- workspace resolver/inheritance/composition;
- responsive behavior;
- keyboard/focus/accessibility;
- reduced motion/high contrast;
- server-state query keys;
- invalidation mapping;
- loading/empty/stale/degraded/blocked/offline/recovery states.

### Order 4 — Context ingestion and continuous growth

Implement all accepted source classes and complete connector lifecycles:

- local files/folders;
- repository docs/code;
- existing Focusa state;
- UIAI public research;
- Google Drive;
- OneDrive/SharePoint;
- Gmail;
- Outlook/Microsoft mail;
- work-item providers;
- operator notes/uploads.

Include extraction, OAuth, bounded import, delta sync, health, revocation, indexing, claims, contradiction, impact, Context Cognition, and live UI.

### Order 5 — Role and Interview

Role:

- seed;
- Role Composer;
- grounding;
- assumptions;
- responsibility/non-responsibility;
- evidence expectations;
- handoffs;
- redline;
- approval;
- role/permission separation.

Interview:

- dynamic generation;
- one-question flow;
- persistent compendium;
- rationale/gaps;
- answer types/attachments;
- defer/skip/amend/withdraw;
- tranches;
- readiness;
- resume;
- context impact.

### Order 6 — Spec Workbench integration

- C.R.I.S.T. handoff;
- Project Genesis template;
- Reality Scanner;
- UIAI research;
- proposer/challenger/auditor/synthesis;
- section gates;
- reconciliation;
- operator approvals;
- approved artifact;
- repo-write preview;
- Receipt preview;
- Trajectory proposal.

Generated Markdown alone is insufficient.

### Order 7 — Task decomposition, Work Rail, closure, and Receipts

- provider-neutral plan;
- dependency graph;
- acceptance/evidence mapping;
- preview/edit/approval/materialization;
- Beads;
- GitHub Issues;
- Linear;
- Asana;
- Markdown Checklist;
- provider health;
- Workpoint binding;
- Work Rail;
- closure/reconciliation;
- verified strike-through;
- Receipt/history.

### Order 8 — UIAI rich artifacts and live browser integration

- Workspace Artifact descriptor;
- screenshots;
- research documents;
- snapshots;
- diagnostics;
- charts/datasets;
- FPV;
- evidence linkage;
- provenance;
- renderer dispatch;
- SSE invalidation;
- image tiers;
- terminal fallback;
- redaction/freshness.

### Order 9 — Complete vertical workspace set

- General;
- Software;
- Legal;
- Markets;
- Research;
- Custom;
- composite profiles.

Each requires theme, visual grammar, home canvas, panels, terminology, icons, density, renderers, history, C.R.I.S.T., evidence, controls, all states, and a demo project.

### Order 10 — Complete client/package parity

- stock Pi compatibility;
- enhanced Pi sidebar/docks;
- Mission Deck PWA;
- Tauri shell;
- menubar;
- native TUI;
- API/CLI/headless.

### Order 11 — Full-system hardening and release proof

- schema compatibility;
- migrations;
- isolation;
- connector expiry/recovery/revocation;
- replay;
- SSE reconnect;
- offline states;
- rate/size limits;
- large project/document/interview/task/artifact tests;
- accessibility;
- visual regression;
- all client tests;
- installer/release integration;
- actual evidence bundle.

---

## 9. Parallel execution lanes

After stable contracts/core:

```text
                    ┌─ UI substrate and workspace engine
Contracts + Core ───┼─ Context extraction and connectors
                    ├─ Pi convergence and docks
                    ├─ UIAI artifact bridge
                    └─ Spec Workbench integration
```

Convergence:

```text
Context + UI
→ Role and Interview

Role + Interview + Workbench
→ approved Project Genesis Spec

Approved Spec + adapters
→ Tasks and Work Rail

Workspace engine + artifacts + manifests
→ professional verticals

All lanes
→ parity, hardening, release proof
```

---

## 10. UX completeness laws

### No dead ends

Every blocked, stale, disconnected, unauthorized, empty, or failed state shows:

```text
what happened
why
what remains safe
what the operator can do
exact recovery action
```

### Progressive disclosure, not removal

```text
Beginner → one next action
Operator → dense state and controls
Advanced → policy, schemas, connectors, diagnostics, receipts
```

### Autosave and resumability

Persist uploads, cursors, role drafts, answers, open questions, spec progress, approvals, task edits, workspace choice, and panel preferences.

### Preview before consequential mutation

```text
dry run
→ preview
→ explicit approval where required
→ commit
→ Receipt
```

### One obvious primary action

Examples:

```text
Add Context
Review Role
Answer Next Question
Open Spec Workbench
Approve Task Plan
Start First Workpoint
```

### Dynamic capability truth

Buttons and panels derive availability from capabilities/provider health rather than failing after activation.

---

## 11. Performance laws

1. Bounded purpose-specific read models.
2. SSE invalidation with targeted refetch.
3. Stable handles for large blobs.
4. Background extraction/indexing/sync outside canonical locks.
5. Virtualized large lists.
6. Lazy rich artifacts.
7. Incremental provider sync.
8. Resource-mode-aware queues and throttling.
9. Content hashing and deduplication.
10. Paged dataset/history reads.
11. Bounded diagnostics/transcripts.
12. No whole-state client mirroring.

---

## 12. Required decomposition prompt

The Spec 120 decomposition instruction must include this language verbatim or equivalently:

```text
This specification is not being decomposed to identify a small MVP while
moving the remaining requirements to an unspecified future date.

Create the complete implementation graph for every accepted normative
requirement before implementation begins.

Execution order may sequence work by dependency. Sequencing does not remove
any requirement from the completion graph.

Every requirement must map to implementation tasks, dependency edges,
acceptance criteria, affected runtime and UI surfaces, tests, proof artifacts,
documentation, and closure evidence.

Do not use “later,” “future enhancement,” “post-MVP,” “nice to have,” or
“out of scope for now” as substitutes for tasks.

A blocked dependency remains open and blocks parent closure.

Do not implement a second system when Focusa, UIAI Engine, Pi, Spec 120,
Spec 116, Spec 119, or the existing Svelte/Tauri stack already owns the
relevant primitive.

Do not claim completion from schemas, stubs, static cards, mock providers,
placeholder success envelopes, or backend-only behavior when the specification
requires an integrated user experience.

All accepted professional workspaces, connectors, clients, artifact renderers,
approval paths, recovery states, and proof requirements remain required for
specification closure unless the operator explicitly removes them through a
versioned specification amendment.
```

---

## 13. Required decomposition hierarchy

```text
EPIC A — Contracts, schemas, and Pi convergence
EPIC B — Canonical state, reducers, events, and persistence
EPIC C — Shared design system and dynamic workspace engine
EPIC D — Context ingestion, indexing, OAuth, and connectors
EPIC E — Role Composer and approval
EPIC F — Dynamic Interview and compendium
EPIC G — Spec 120 Project Genesis integration
EPIC H — Provider-neutral decomposition and adapters
EPIC I — Work Rail, closure, and Receipts
EPIC J — UIAI rich artifacts and FPV
EPIC K — Software workspace
EPIC L — Legal workspace
EPIC M — Markets workspace
EPIC N — Research workspace
EPIC O — General, Custom, and composite workspaces
EPIC P — Pi enhanced distribution and compatibility
EPIC Q — Mission Deck PWA and Tauri
EPIC R — Menubar and native TUI parity
EPIC S — Migration, accessibility, security, performance, and release proof
```

Each epic includes implementation, integration, tests, docs, and evidence children.

---

## 14. Acceptance criteria

Spec 135D is accepted when:

1. The full series has a Complete Feature Ledger.
2. Every normative requirement maps to implementation and proof tasks.
3. No indefinite deferral language remains.
4. Cross-spec dependencies remain visible blockers.
5. Framework qualification records are approved.
6. Pi package convergence is resolved.
7. Generated schemas/types serve clients.
8. Shared UI packages replace duplicated client logic.
9. Incremental sync and hybrid retrieval architecture are defined and proven.
10. All Orders 0–11 remain in the closure graph.
11. Parallel work preserves dependencies and integration tests.
12. Performance, accessibility, security, migration, and recovery tasks are first-class.
13. Parent closure is mechanically blocked by incomplete ledger entries.
14. Actual integrated proof exists across every acceptance-critical surface.

---

## 15. Closure blockers

This spec cannot close while:

- a required feature lacks a task;
- a required task lacks acceptance/proof criteria;
- cross-spec work is treated as somebody else’s future problem;
- duplicated frameworks replace existing Focusa/UIAI/Pi primitives;
- Pi package mismatch remains;
- schema/client contracts are manually divergent;
- document extraction/search frameworks remain unqualified;
- client parity is absent from the graph;
- security/accessibility/performance are cleanup-only tasks;
- backend success is used as proof of complete UX;
- any accepted feature is silently deferred.

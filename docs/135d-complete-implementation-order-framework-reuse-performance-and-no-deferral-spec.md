# Spec 135D — Complete Implementation Order, Framework Reuse, Performance, and No-Deferral Constitution

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135D.  
**Scope:** complete feature ledger, decomposition law, dependency ordering, cross-spec ownership, domain-general ontology and domain-pack foundations, typed multiplexed runtime, Mission Canvas and Work Surface contracts, browser-context isolation, snapshot/event compatibility, framework qualification, code reuse, shared contracts, performance, parallel execution, UX completeness, proof discipline, and final closure.

---

## 0. One-line definition

Spec 135 must be implemented as one complete dependency graph: sequence by dependency, parallelize where safe, reuse and integrate existing systems, preserve multi-project and multi-session scope, never silently omit or indefinitely defer accepted features, and never claim completion from stubs, schemas, mock providers, backend-only behavior, or surrogate proof.

---

## 1. Core implementation directive

```text
Sequence by dependency.
Parallelize where safe.
Integrate existing systems.
Preserve multiplexing and typed scope.
Do not omit.
Do not silently defer.
Do not declare partial infrastructure to be the completed product.
```

The implementation orders in this document are execution ordering. They do not split the series into “real now” and “maybe someday.”

Every accepted normative feature in Spec 135/135A/135B/135C/135D/135E/135F/135G must enter the implementation graph before implementation begins.

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
      mission_canvas: false
      multiplexing: false
    acceptance_criteria: []
    evidence_requirements: []
    closure_status: open | blocked | verified | operator_removed
```

The parent series cannot close while a required ledger entry is missing, partial, open, or blocked.

---

## 5. Cross-spec dependencies remain in the completion boundary

Different ownership does not mean deferred dependency.

Required dependencies from Specs 38–41, 43, 45–50, 61, 70, 72, 74, 75, 77, 88, 98, 100, 104, 109, 111, 116, 117, 119, 120, 121, 124, 125, 130, 133, 135F, and 135G remain explicit blockers in the Spec 135 graph until implemented and proven.

A task may be owned by its original spec and cross-linked to Spec 135. It may not disappear from the Spec 135 completion ledger.

---

## 6. Architectural reuse laws

### 6.1 One Focusa runtime

Do not create:

- a workspace daemon;
- a C.R.I.S.T. daemon;
- a Mission Canvas authority daemon;
- a connector-state daemon;
- a sidebar state service;
- a second task authority;
- a second evidence store;
- a second project-memory system;
- a second spec engine;
- a second session or browser authority model.

Use the existing Focusa runtime, Instance/Session/Attachment model, scoped project/workstream state, Silent Session control plane, and UIAI session model.

### 6.2 One canonical persistence path

Canonical workspace, Project Genesis, context claim, role, interview, task plan, artifact-link, ontology registry, candidate/canonical graph, verification-ledger, domain-pack binding, slice-policy, semantic-subscription, Instance, Session, Attachment, proposal-resolution, writer-lease, and runtime-session events use Focusa reducers, event persistence, state snapshots, and event-chain discipline in the local Focusa node’s SQLite-backed runtime. Large source and evidence payloads remain externalized through ECS/content-addressed handles.

Mission Canvas and Work Surface layout state may be persisted as user/device projection state, but focused tabs, selected panes, and layout order may not become canonical project/session authority.

Project JSON, Pi globals, browser storage, Svelte stores, menubar state, UIAI state, connector caches, `.focusa-project.json`, and `.pi/settings.json` may identify, configure, project, import, export, or cache state but may not become canonical Project Genesis or session authority. Approved repository specs are written only through the operator-gated Spec 120 repo-write path. Raw connected documents and email are not copied into a repository by default.

Focusa Cloud may provide opt-in encrypted synchronization, relay, hosted Receipts, and team projections. It does not silently replace local project-semantic authority. Multi-node reconciliation follows existing ProjectIdentity/CRDT and scoped authority rules.

### 6.3 One browser/research/document execution engine

UIAI owns browser/search/session/media/diagnostics execution and the rich Documents execution boundary. Focusa does not rebuild those systems. Focusa consumes normalized, source-linked artifacts, claims, chunks, diagnostics, browser-context identities, target identities, and handles through typed contracts.

### 6.4 One specification engine

C.R.I.S.T. `S` uses Spec 120. It does not create another Workbench, challenger, auditor, approval system, or spec-to-task engine.

### 6.5 One work-item/closure model

C.R.I.S.T. `T` uses Spec 120 decomposition, Spec 116 adapters/closure, Workpoints, and Spec 119 Receipts.

### 6.6 One shared UI contract

Pi, PWA, UIAI Engine Cockpit/Tauri, menubar, and native TUI share:

- generated schemas/types;
- read models;
- action IDs;
- status enums;
- Workspace View Profile manifests;
- Mission Canvas and Work Surface schemas;
- Instance/Session/Attachment identities;
- browser-context and browser-target identities;
- theme tokens;
- artifact descriptors;
- invalidation events.

No client reimplements domain, session, routing, isolation, conflict, or authority policy.

### 6.7 One semantic substrate

Do not create separate legal, markets, research, software, connector, C.R.I.S.T., UIAI, session-local, or client-local ontology engines. Spec 135F extends the existing Focusa ontology through one core-owned registry, candidate graph, canonical graph, verification ledger, V1 compatibility projection, domain-pack composition system, and semantic delta plane.

### 6.8 One concurrency substrate

Specs 40, 41, 98, 104, 133, and 135G own the concurrency model:

```text
ProjectRootKey
→ WorkstreamKey
→ AttachmentKey
→ runtime-specific session/run/context/target identities
→ Work Surface projection
```

Mission Canvas visual focus must never replace this hierarchy.

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

New domain modules should align with existing crate boundaries rather than become a separate service. Required modules or equivalent boundaries include:

```text
focusa-core/src/workspace/
focusa-core/src/mission_canvas/
focusa-core/src/work_surface/
focusa-core/src/runtime_attachments/
focusa-core/src/ontology_registry/
focusa-core/src/ontology_graph/
focusa-core/src/domain_packs/
focusa-core/src/verification_policy/
focusa-core/src/slice_policy/
focusa-core/src/semantic_subscriptions/
```

Mission Canvas modules own projection contracts and restoration metadata only. Canonical Instance/Session/Attachment, Silent Session, Workpoint, proposal, and lease authority remains in the appropriate existing core subsystems.

### 7.2 Schema generation

Spec 109 requires authoritative JSON Schema/OpenAPI.

Required flow:

```text
Rust/domain schemas
→ generated JSON Schema and OpenAPI
→ generated TypeScript contracts/client
→ Pi, Svelte, connector, UIAI, and test consumers
```

Hand-maintained duplicate request/response interfaces are forbidden where generation can preserve the contract.

### 7.3 Shared rich UI stack

Use the existing Focusa/UIAI frontend direction:

- SvelteKit 2;
- Svelte 5;
- Tailwind CSS 4;
- Vite;
- static adapter;
- Tauri 2 in the UIAI Engine Cockpit;
- shared design/Mission Canvas packages.

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

focusa-mission-canvas-ui
  Work Surface strip, splits, session inventory, routing, restoration,
  contention, and aggregate/local projection behavior
```

The Mission Deck PWA, UIAI Engine Cockpit, and menubar must consume shared packages rather than copy screens.

### 7.4 UI primitives — selected framework

The selected generic component foundation is **shadcn-svelte with Bits UI primitives**, using Svelte 5 and Tailwind CSS 4. This stack owns generic dialogs, sheets, tabs, radio groups, forms, command palettes, menus, tooltips, scroll areas, focus behavior, and accessibility primitives.

Custom development remains focused on Focusa-specific components:

- Mission Canvas Work Surface strip;
- session switcher and split manager;
- Mission Ladder;
- Work Rail;
- Evidence and Receipt cards;
- C.R.I.S.T. progress;
- Role redline;
- Interview compendium;
- Spec approval;
- vertical artifact viewers;
- contention and proposal-resolution views;
- browser-context isolation indicators.

The dependency versions must be pinned and validated against static/PWA/Tauri builds, keyboard-only behavior, reduced motion, high contrast, and bundle budgets. Implementation-time substitution requires a versioned operator amendment rather than an informal framework swap.

### 7.5 Server-state synchronization — selected framework

The selected server-state framework is **TanStack Query for Svelte**. It owns typed server-state reads, stale state, mutation state, targeted invalidation, bounded background refetch, reconnect behavior, and optimistic UI only where Focusa preview/commit and version contracts make it safe.

SSE events map to stable query keys and targeted invalidation. TanStack Query is a projection/cache layer only; it does not become a normalized canonical client database or session authority store.

### 7.6 Pi package convergence — selected namespace

The canonical Pi runtime, SDK, TUI, peer-dependency, and extension import namespace is:

```text
@earendil-works/pi-coding-agent
@earendil-works/pi-agent-core
@earendil-works/pi-ai
@earendil-works/pi-tui
```

All Focusa, UIAI Engine, queue-steering, and UIAI Engine Cockpit integration packages must pin one tested compatible Earendil Pi release and share a compatibility suite.

Required migration:

```text
inventory legacy @mariozechner/pi-* imports and pins
→ migrate Focusa extension to @earendil-works/pi-*
→ align UIAI and queue-steering extensions
→ establish one lock/version matrix
→ run extension, RPC, SDK, Mission Canvas dock, theme, image, event,
  multi-session, and targeted-steering compatibility proof
```

The Focusa-enhanced Pi distribution is a packaged configuration over the upstream Earendil Pi runtime, not a separately renamed Pi fork. A future public Focusa npm package name is contingent only on verified registry-scope ownership and does not reopen this runtime-namespace decision.

### 7.7 Local document extraction — selected framework

Do not hand-write parsers for PDF, DOCX, PPTX, XLSX, email, images, and archives.

The selected structured extraction engine is **Docling**, hosted by the UIAI Engine Documents subsystem as a persistent isolated local worker/service. It is responsible for PDF and Office structure, page/layout order, tables, images, email formats, OCR-capable inputs, normalized document JSON, Markdown projections, and page/source provenance.

Plain text, Markdown, source code, JSON, JSONL, and CSV use direct bounded ingestion because they are already structured/textual inputs and do not require a second document-conversion framework.

Required execution contract:

```text
Focusa context-source adapter
→ UIAI Documents extraction request
→ isolated Docling worker
→ normalized document + source/page refs + diagnostics
→ Focusa Workspace Artifact / Project Context Artifact
→ chunk/index/candidate-claim pipeline
```

Required proof covers:

- MIT/code and model-license inventory;
- macOS/Linux/Windows x86_64 and arm64 packaging;
- persistent-worker startup and health;
- CPU/memory/resource-mode budgets;
- malformed and hostile document isolation;
- PDF/DOCX/PPTX/XLSX/EML/MSG/image fixtures;
- OCR and non-OCR paths;
- table and page provenance;
- cancellation, timeout, and recovery.

No separate default MarkItDown or Tika pipeline is introduced. A per-format fallback may be added only through the same extraction interface after a failing conformance fixture proves a Docling capability gap and the fallback passes license, security, provenance, and performance gates.

### 7.8 Context retrieval — selected stack

The selected local hybrid retrieval stack is:

```text
SQLite FTS5
+ sqlite-vec behind a pinned versioned adapter
+ fastembed-rs for local ONNX embeddings and reranking
```

FTS5 is the canonical local lexical index. `sqlite-vec` is isolated behind a Focusa adapter because it is pre-v1 and may introduce storage/API changes; its exact version, vector dimensions, distance metric, and migration format are pinned. `fastembed-rs` provides local embedding and reranking without a mandatory cloud dependency.

The initial versioned embedding profile is:

```yaml
id: focusa.embedding.bge-small-en-v1.5.v1
model: BAAI/bge-small-en-v1.5
runtime: fastembed-rs
storage: sqlite-vec
lexical: sqlite-fts5
```

Every embedding record persists model ID, model revision/hash, dimensions, normalization, chunking profile, source revision, and generation version. Projects may select another registered compatible embedding profile without changing the retrieval contract; profile changes require deterministic re-embedding and migration state.

Target retrieval:

```text
project/workstream scope
+ source and permission filter
+ freshness filter
+ lexical rank
+ semantic rank
+ optional local rerank
+ provenance-preserving result assembly
```

### 7.9 Connected-source synchronization and reference connector

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

**Google Drive is the first reference connector.** It must prove minimum-scope OAuth, account/folder scoping, bounded import, Google-native document export, revisions, permission/provenance capture, changes-feed cursors, incremental synchronization, cursor recovery, health, revocation, and ProjectContextDelta emission.

The Google OAuth/account substrate is reused by Gmail. OneDrive, SharePoint, Outlook/Microsoft mail, task-provider connectors, and the remaining accepted sources remain required in the same implementation graph. Reference order does not create deferral authority.

---

## 8. Complete dependency-ordered build graph

Every order is required. Orders are not release-deferral buckets.

### Order 0 — Specification lock and complete decomposition

Required outputs:

1. approved Spec 135 series, including Specs 135F and 135G;
2. Complete Feature Ledger;
3. current-code Reality Pack;
4. cross-spec dependency graph including Specs 38–41, 43, 98, 104, and 133;
5. resolved framework decision record and exact pinned dependency/model matrix;
6. canonical/projection/domain-semantic/session-origin ownership map;
7. Instance/Session/Attachment and Mission Canvas compatibility constitution;
8. browser-context/target isolation and retention matrix;
9. snapshot and event compatibility constitution;
10. V1 ontology compatibility fixtures and expected projections;
11. domain-pack conformance and isolation matrix;
12. security/privacy model;
13. full parent/child task graph;
14. acceptance/proof mapping;
15. client parity matrix;
16. migration and downgrade matrix;
17. resolved-decision conformance check against Spec 135 §17.

### Order 1 — Contract and runtime convergence

- Pi package convergence on `@earendil-works/pi-*`;
- operation/schema/status IDs;
- ProjectRootKey, WorkstreamKey, and AttachmentKey generated contracts;
- Instance/Session/Attachment inventory and role contracts;
- Mission Canvas and Work Surface contracts;
- Work Surface attachment/runtime-ref contracts;
- browser-session/context/target and isolation-class contracts;
- targeted steering/follow-up recipient contracts;
- contention/proposal and writer-lease projections;
- Workspace Profile contracts;
- C.R.I.S.T./Project Genesis contracts;
- Context source/artifact/claim/delta contracts;
- Role/Interview contracts;
- Workspace Artifact contracts with session-origin identity;
- task/read models;
- invalidation events;
- core ontology registry and namespaced semantic IDs;
- domain-pack manifest, compatibility, composition, and conformance contracts;
- candidate/canonical graph and verification-ledger contracts;
- verification-policy and slice-policy contracts;
- versioned snapshot and stored-event envelopes with minimum reader/writer versions;
- semantic subscription/delta/cursor contracts;
- preview/commit/idempotency/version rules;
- generated OpenAPI/JSON Schema/TypeScript.

### Order 2 — Canonical state, reducer events, and read models

Suggested modules:

```text
focusa-core/src/workspace/
focusa-core/src/mission_canvas/
focusa-core/src/work_surface/
focusa-core/src/runtime_attachments/
focusa-core/src/project_genesis/
focusa-core/src/project_context/
focusa-core/src/project_role/
focusa-core/src/project_interview/
focusa-core/src/workspace_artifact/
focusa-core/src/ontology_registry/
focusa-core/src/ontology_graph/
focusa-core/src/domain_packs/
focusa-core/src/verification_policy/
focusa-core/src/slice_policy/
focusa-core/src/semantic_subscriptions/
```

Required:

- reducers and existing session/attachment integration;
- SQLite persistence;
- event-chain participation;
- versions;
- scope enforcement;
- provenance;
- Instance/Session/Attachment inventory;
- proposal-resolution and contention projections;
- writer-lease and worktree projections;
- Work Surface durable refs and user/device restoration metadata;
- explicit separation of focused Work Surface from canonical active state;
- answer supersession;
- role revisions;
- workspace-selection history;
- context-claim lifecycle;
- typed registry extraction with exact V1 name/route parity;
- separate candidate and canonical semantic stores;
- verification ledger and policy-backed promotion;
- domain-pack activation, versioning, and composition;
- V1 objects/links/action-catalog compatibility projection;
- ontology-derived Workpoint candidate projection without changing Workpoint authority;
- generalized slice-policy registry;
- semantic delta subscriptions and cursor persistence;
- snapshot/event migrations, unknown-event preservation, and downgrade-write protection;
- bounded read models;
- SSE events.

No placeholder success routes.

### Order 3 — Shared dynamic UI and Mission Canvas substrate

- design tokens;
- state vocabulary;
- shadcn-svelte/Bits UI primitives;
- TanStack Query server-state layer;
- Mission Canvas Work Surface strip;
- session/workstream/project switcher;
- tab grouping, pinning, unread state, and splits;
- close-view versus pause/terminate actions;
- layout/panel/home-canvas registries;
- renderer/action/terminology/theme/icon/history registries;
- session-kind and attachment-role presentation registry;
- contention/proposal and writer-lease views;
- targeted steering/follow-up routing UI;
- aggregate versus surface-local Work Rail;
- domain-semantic binding registry that consumes, but never owns, canonical domain policy;
- workspace resolver/inheritance/composition;
- responsive behavior;
- keyboard/focus/accessibility;
- reduced motion/high contrast;
- server-state query keys;
- invalidation mapping;
- loading/empty/stale/degraded/blocked/offline/recovery states;
- deterministic Work Surface rehydration.

### Order 4 — Context ingestion and continuous growth

Implement all accepted source classes and complete connector lifecycles:

- local files/folders through UIAI Documents/Docling where extraction is required;
- repository docs/code;
- existing Focusa state;
- UIAI public research;
- Google Drive as the first reference connector;
- OneDrive/SharePoint;
- Gmail;
- Outlook/Microsoft mail;
- work-item providers;
- operator notes/uploads.

Include extraction, OAuth, bounded import, delta sync, health, revocation, FTS5/sqlite-vec/fastembed indexing, claims, contradiction, impact, Context Cognition, candidate-semantic writes, verification-policy evaluation, reviewed canonical promotion, session-origin attribution, and live UI.

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
- ontology-derived/operator-authored Workpoint candidate validation;
- Workpoint binding through existing authority;
- Work Rail surface-local/project-aggregate/cross-project-advisory modes;
- session and Work Surface linkage;
- closure/reconciliation;
- verified strike-through;
- Receipt/history.

### Order 8 — UIAI rich artifacts and multiplexed browser integration

- Workspace Artifact descriptor with project/workstream and session-origin identity;
- UIAI session inventory;
- browser-context inventory and isolation classes;
- browser-target inventory and context membership;
- multiple targets per context;
- multiple isolated contexts per project/workstream;
- explicit shared-context behavior;
- screenshots;
- research documents;
- snapshots;
- diagnostics;
- charts/datasets;
- FPV per UIAI session/context;
- evidence linkage;
- provenance;
- renderer dispatch;
- SSE invalidation;
- image tiers;
- terminal fallback;
- redaction/freshness;
- context/target close, move, duplicate, restore, and retention behavior.

### Order 9 — Complete vertical workspace set

- General;
- Software;
- Legal;
- Markets;
- Research;
- Custom;
- composite profiles.

Each requires theme, visual grammar, home canvas, panels, terminology, icons, density, renderers, history, C.R.I.S.T., Mission Canvas behavior, session/Work Surface presentation, evidence, controls, all states, a demo project, and an operational or truthfully degraded domain-pack composition. Visual completeness without semantic-pack and multiplexing conformance is incomplete.

### Order 10 — Complete client/package parity

- stock Pi compatibility and session switcher;
- enhanced Pi Mission Canvas sidebar/docks;
- Mission Deck PWA;
- UIAI Engine Cockpit Tauri shell hosting Focusa Mission Canvas projections;
- menubar;
- native TUI;
- API/CLI/headless.

### Order 11 — Full-system hardening and release proof

- schema compatibility;
- snapshot-schema and stored-event-envelope compatibility;
- unknown future event/type preservation;
- old-writer and downgrade protection;
- V1 projection replay equivalence;
- migrations;
- project/workstream/session/attachment isolation;
- browser cookie/storage/permission isolation;
- connector expiry/recovery/revocation;
- replay;
- SSE reconnect;
- Work Surface rehydration;
- offline states;
- rate/size/concurrency limits;
- large project/document/interview/task/artifact/session tests;
- concurrent observations and decision conflicts;
- writer lease/worktree isolation;
- accessibility;
- visual regression;
- all client tests;
- installer/release integration;
- actual evidence bundle.

---

## 9. Parallel execution lanes

After stable contracts/core:

```text
                    ┌─ Mission Canvas and workspace engine
Contracts + Core ───┼─ Context extraction and connectors
                    ├─ Pi convergence and docks
                    ├─ UIAI artifact/browser-context bridge
                    ├─ session/attachment/rehydration substrate
                    └─ Spec Workbench integration
```

Convergence:

```text
Scoped runtime + Mission Canvas
→ multiplexed Work Surfaces

Context + UI
→ Role and Interview

Role + Interview + Workbench
→ approved Project Genesis Spec

Approved Spec + adapters
→ Tasks and Work Rail

Workspace engine + artifacts + sessions + manifests
→ professional verticals

All lanes
→ parity, hardening, release proof
```

---

## 10. UX completeness laws

### No dead ends

Every blocked, stale, disconnected, unauthorized, empty, failed, ended-session, missing-context, or rehydration state shows:

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
Operator → dense state, sessions, and controls
Advanced → policy, schemas, connectors, diagnostics, contention, and receipts
```

### Autosave and resumability

Persist uploads, cursors, role drafts, answers, open questions, spec progress, approvals, task edits, workspace choice, Mission Canvas open/pinned/grouped/split surfaces, unread cursors, and panel preferences.

### Preview before consequential mutation

```text
dry run
→ preview
→ explicit approval where required
→ commit
→ Receipt
```

Closing a view, pausing a session, terminating a session, closing a browser target, and closing a browser context are distinct actions with distinct previews.

### One obvious primary action

Examples:

```text
Add Context
Review Role
Answer Next Question
Open Spec Workbench
Approve Task Plan
Start First Workpoint
Open Work Surface
Respond to Waiting Session
Resolve Contention
```

### Dynamic capability truth

Buttons and panels derive availability from capabilities, attachment roles, session health, browser isolation, writer leases, and provider health rather than failing after activation.

---

## 11. Performance laws

1. Bounded purpose-specific read models.
2. SSE invalidation with targeted refetch.
3. Stable handles for large blobs.
4. Background extraction/indexing/sync outside canonical locks.
5. Virtualized large lists and Work Surface inventories.
6. Lazy rich artifacts and inactive pane content.
7. Incremental provider and session synchronization.
8. Resource-mode-aware queues, concurrency limits, and throttling.
9. Content hashing and deduplication.
10. Paged dataset/history/event/session reads.
11. Bounded diagnostics/transcripts.
12. No whole-state client mirroring.
13. Semantic and session events carry bounded refs, scope, origin IDs, versions, and cursors rather than full graph, browser, or artifact payloads.
14. Background Work Surfaces do not rerender high-frequency content unless visible or subscribed.
15. FPV/browser streams are scoped per UIAI session/context and do not block Focusa state updates.

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

Do not implement a second system when Focusa, UIAI Engine, Pi, Specs 40, 41,
98, 104, 116, 119, 120, 133, 135F, 135G, or the existing Svelte/Tauri stack
already owns the relevant primitive.

Do not claim completion from schemas, stubs, static cards, mock providers,
placeholder success envelopes, backend-only behavior, or a single-active-session
UI when the specification requires an integrated multiplexed experience.

All accepted professional workspaces, connectors, clients, Mission Canvas
Work Surfaces, session attachments, browser-context isolation, artifact
renderers, approval paths, recovery states, and proof requirements remain
required for specification closure unless the operator explicitly removes them
through a versioned specification amendment.
```

---

## 13. Required decomposition hierarchy

```text
EPIC A — Contracts, schemas, Pi convergence, and typed scope
EPIC B — Canonical state, reducers, events, attachments, and persistence
EPIC C — Shared design system and dynamic workspace engine
EPIC D — Context ingestion, indexing, OAuth, and connectors
EPIC E — Role Composer and approval
EPIC F — Dynamic Interview and compendium
EPIC G — Spec 120 Project Genesis integration
EPIC H — Provider-neutral decomposition and adapters
EPIC I — Work Rail, closure, and Receipts
EPIC J — UIAI rich artifacts, contexts, targets, and FPV
EPIC K — Software workspace
EPIC L — Legal workspace
EPIC M — Markets workspace
EPIC N — Research workspace
EPIC O — General, Custom, and composite workspaces
EPIC P — Pi enhanced distribution and compatibility
EPIC Q — Mission Deck PWA and UIAI Engine Cockpit integration
EPIC R — Menubar and native TUI parity
EPIC S — Migration, accessibility, security, performance, and release proof
EPIC T — Domain-general ontology registry, semantic graphs, domain packs, slices, and reactions
EPIC U — Multiplexed Mission Canvas, Work Surfaces, session attachments, contention, and rehydration
```

Each epic includes implementation, integration, tests, docs, and evidence children.

---

## 14. Acceptance criteria

Spec 135D is accepted when:

1. The full series has a Complete Feature Ledger.
2. Every normative requirement maps to implementation and proof tasks.
3. No indefinite deferral language remains.
4. Cross-spec dependencies remain visible blockers.
5. The selected framework stack is pinned, integrated, and proven against its qualification matrix.
6. Pi package convergence on `@earendil-works/pi-*` is resolved and proven.
7. Generated schemas/types serve clients.
8. Shared UI packages replace duplicated client logic.
9. Incremental sync and the selected FTS5/sqlite-vec/fastembed hybrid retrieval architecture are implemented and proven.
10. All Orders 0–11 remain in the closure graph.
11. Parallel work preserves dependencies and integration tests.
12. Performance, accessibility, security, migration, recovery, and concurrency tasks are first-class.
13. Parent closure is mechanically blocked by incomplete ledger entries.
14. Actual integrated proof exists across every acceptance-critical surface.
15. Spec 135F requirements appear in the Complete Feature Ledger and Orders 0–2 before vertical/client implementation.
16. Spec 135G requirements appear in the Complete Feature Ledger and Orders 0–3 before Mission Canvas/vertical/client completion.
17. Archived V1 snapshots/events replay to an equivalent compatibility projection, while unknown future events are preserved and incompatible old writers are blocked.
18. Every accepted vertical has tested domain-pack conformance, isolation, verification, slice, Workpoint-candidate, Mission Canvas, and degraded-mode behavior.
19. Google Drive proves the full reference-connector contract before the shared connector substrate is considered stable.
20. UIAI Documents/Docling extraction proves all required fixture, provenance, isolation, and recovery paths.
21. Multi-project, same-project multi-session, same-workstream contention, browser-container isolation, shared-context warning, Work Surface close semantics, restart rehydration, and concurrent-writer scenarios pass with actual evidence.
22. The word Cockpit is used only for UIAI Engine Cockpit in the Spec 135 series and new implementation labels.

---

## 15. Closure blockers

This spec cannot close while:

- a required feature lacks a task;
- a required task lacks acceptance/proof criteria;
- cross-spec work is treated as somebody else’s future problem;
- duplicated frameworks replace existing Focusa/UIAI/Pi primitives;
- Pi package mismatch remains;
- schema/client contracts are manually divergent;
- selected document extraction or search frameworks are not pinned, integrated, migrated, and proven;
- client parity is absent from the graph;
- security/accessibility/performance/concurrency are cleanup-only tasks;
- backend success is used as proof of complete UX;
- any accepted feature is silently deferred;
- client or vertical work begins before the semantic registry, typed scope, session attachment, and compatibility contracts are stable;
- domain-pack, candidate/canonical graph, verification-policy, slice-policy, or semantic-subscription work is omitted as generic future ontology work;
- Mission Canvas or Work Surface multiplexing is omitted as future UI work;
- the UI assumes one global active session or uses focused view state as canonical authority;
- browser context and target identity or isolation is omitted;
- session-origin identity is omitted from artifacts/events;
- a different framework, Pi namespace, connector reference, desktop shell, or Mission Canvas naming is substituted without a versioned operator amendment;
- a generic Focusa/Pi surface uses the word Cockpit.

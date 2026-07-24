# Spec 135D — Complete Implementation Order, Framework Reuse, Performance, and No-Deferral Constitution

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135D.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs framework, testing, sequencing, browser-proof, generated-UI, and compatibility conflicts.

---

## 0. One-line definition

Implement Specs 135 and 135A–135K as one complete machine-readable dependency graph: sequence by dependency, integrate one real vertical path immediately, open safe parallel lanes as soon as contracts stabilize, submit reusable behavior to greater Focusa primitives, and never omit, silently defer, mock, or misrepresent accepted product behavior.

---

## 1. Core laws

```text
Sequence by dependency.
Parallelize after shared seams stabilize.
Build the Cross-Functional Alpha first.
Keep every accepted requirement in the closure DAG.
Reuse Focusa, UIAI Engine, Pi, and decided frameworks.
Submit general behavior to greater Focusa primitives.
Keep exact project/workstream/attachment scope.
Do not omit.
Do not silently defer.
Do not claim completion from stubs, schemas, mocks, static cards, or backend-only behavior.
```

Execution ordering does not divide requirements into “real now” and “maybe later.”

Surface ownership remains explicit: **Focusa Mission Canvas** is the canonical Focusa work surface, while **UIAI Engine Cockpit** remains the rich browser execution, diagnostics, artifact, and Eval shell. Neither surface creates competing canonical state or assumes the other product’s ownership.

---

## 2. Sequencing and blocking

### Sequenced

A requirement is sequenced only when it has a stable requirement ID, owner, dependency edges, implementation tasks, client surfaces, tests, UIAI Eval scenarios when browser-facing, Evidence, Receipts, migration posture, and closure state.

### Blocked

A blocked requirement retains a task, explicit blocker, owner, recovery path, and closure impact. It remains unfinished and blocks parent closure.

### Removed

A requirement leaves the graph only through a versioned operator-approved amendment containing original text, reason, consequences, affected criteria, and approval reference.

### Optional

Optional describes a user runtime choice, not implementation discretion. A user can decline Google Drive; the accepted Google Drive connector remains required implementation work.

---

## 3. Forbidden planning language

Agents MUST NOT use these phrases to remove implementation work:

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

Required form:

```text
requirement ID
execution order
explicit blockers and owners
required proof
parent closure impact
```

---

## 4. Machine-readable delivery graph

Before decomposition, create and validate:

```text
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Required ledger shape:

```yaml
requirement_id:
source_spec:
source_section:
normative_text:
primitive_owner:
repository_owner:
current_status: missing | partial | implemented | verified
dependencies: []
implementation_tasks: []
core_types: []
reducer_actions: []
api_operations: []
generated_contracts: []
generated_ui_surfaces: []
client_surfaces: []
uiai_eval_scenarios: []
tests: []
evidence_requirements: []
receipt_requirements: []
migration_requirements: []
closure_status: open | blocked | verified | operator_removed
```

Agents MUST NOT infer the delivery DAG from prose alone. Missing IDs, owners, dependencies, proof, or closure state block decomposition.

---

## 5. Cross-spec dependencies

Primitive ownership remains with originating specs. Product-completion ownership remains with Spec 135.

Required dependencies include Specs 14, 38–41, 43, 45–50, 61, 70, 72, 74, 75, 77, 88, 98, 100, 104, 107, 109, 111, 116, 117/117A, 119, 120, 121/121A, 124, 125, 130, 133, and every companion 135A–135K.

A dependency implemented under another task tree remains in the Spec 135 closure graph until proven.

---

## 6. Reuse and authority laws

### One Focusa runtime

Do not create a C.R.I.S.T. daemon, workspace daemon, Mission Canvas authority service, second task authority, second Evidence store, second project-memory store, second Spec engine, second session model, or second canonical UI store.

### One canonical persistence path

Canonical Project Genesis, Context, Role, Interview, Spec references, task plans, ontology, domain packs, sessions, attachments, proposals, leases, generated-surface references, UXP/UFI, Evidence, and Receipts use Focusa reducers, SQLite persistence, snapshots, and event-chain discipline. Large payloads remain externalized through handles.

### One browser, research, Documents, and Eval plane

UIAI Engine owns browser/search/session/media/diagnostics/Documents execution, browser contexts and targets, screenshots, responsive/visual proof, browser accessibility proof, and UIAI Engine Eval. Focusa consumes typed scoped artifacts and Evidence references.

Focusa MUST NOT add Playwright or another browser runtime/test plane.

### One specification engine

C.R.I.S.T. Spec and Task stages use Spec 120. Do not create another Workbench, challenger, reference auditor, approval system, or spec-to-task engine.

### One work and closure model

Use Spec 120 decomposition, Spec 116 work-item adapters and closure authority, Workpoints, and Spec 119 Receipts.

### One semantic substrate

Use the Focusa ontology core, candidate/canonical graph separation, verification policies, domain packs, slice policies, semantic subscriptions, and compatibility projections. Do not create vertical-specific ontology engines.

### One concurrency substrate

Use ProjectRootKey, WorkstreamKey, AttachmentKey, Instances/Sessions/Attachments, PRE, writer leases, Spec 133 sessions/runs, UIAI browser contexts/targets, and Spec 135G Work Surface projections. Visual focus never becomes canonical authority.

### One generated UI path

Use Focusa read models, UiInteractionIntent, A2UI, Operation Registry action bindings, ToolResult envelopes, native event replay, UXP/UFI, and trusted components. Do not duplicate business logic in UI routes, client stores, catalogs, or components.

---

## 7. Fixed framework stack

The [Delivery Contract](135-series-current-manifest.md) is the framework lock.

Required foundations:

```text
Backend and contracts
  Rust workspace, Tokio, Axum, SQLite, Serde, Schemars, Utoipa
  JSON Schema 2020-12, OpenAPI 3.0.3
  openapi-typescript/openapi-fetch
  UIAI Engine-owned adapter generated outside Focusa core from published OpenAPI

Generated UI
  A2UI v0.9.1
  @a2ui/web_core/v0_9
  @a2ui/lit/v0_9 permanent renderer
  Focusa Svelte Custom Elements
  AG-UI external compatibility after native stream stabilization

Model execution
  Spec 133 governed sessions
  Pi RPC AgentExecutionAdapter

Documents and retrieval
  UIAI Documents, Docling Serve v1, HybridChunker
  SQLite FTS5, sqlite-vec adapter, fastembed-rs

UI
  SvelteKit 2, Svelte 5, Tailwind 4
  shadcn-svelte, Bits UI, Paneforge
  TanStack Query/Table/Virtual, Svelte Flow
  PDF.js, CodeMirror Merge, Apache ECharts

Code and graph reality
  petgraph, Tree-sitter, ast-grep

Testing and proof
  cargo-nextest, rstest, proptest, insta, wiremock
  Vitest, Svelte Testing Library, Schemathesis, A2UI fixtures
  UIAI Engine Eval for all browser proof

Supply chain
  cargo-deny, cargo-about, package/model/container inventories, Syft SBOM
```

Forbidden runtime or testing ownership:

- Playwright in Focusa;
- complete custom Svelte A2UI renderer;
- Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, `@ai-sdk/svelte`, or Vercel AI Gateway authority;
- LangChain/LlamaIndex as orchestration authority;
- Temporal/Airflow/Celery as durable session authority;
- external vector database without approved benchmark failure;
- another desktop shell;
- another client-canonical state machine.

---

## 8. Greater Focusa primitive submission

Every ticket includes:

```yaml
primitive_submission:
  canonical_owner:
  reusable_primitive:
  crist_specific_projection:
  affected_repositories: []
  core_change:
  api_change:
  generated_contract_change:
  uiai_change:
  client_change:
  migration:
  proof:
```

Implementation order:

```text
general Focusa primitive
→ reducer and canonical state
→ typed Focusa API
→ generated TypeScript clients and portable OpenAPI/JSON Schema contracts
→ C.R.I.S.T. interaction projection
→ renderer
→ UIAI Engine Eval when browser-facing
→ Evidence
→ Receipt
```

A PR is rejected when generally reusable behavior exists only inside Project Genesis, a UI route, a Svelte store, or an A2UI component.

---

## 9. Foundation Train

```text
F0 — freeze at 135K and compile the Delivery Contract
F1 — machine-readable ledger, DAG, parity, framework, and proof matrices
F2 — generate JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generate TypeScript clients and validate portable OpenAPI/JSON Schema contracts
F4 — generate Operation Registry, capabilities, permissions, action bindings
F5 — centralize ToolResult/error envelopes
F6 — implement stable event IDs and SQLite replay plus broadcast live tail
F7 — compatibility/version handshake
F8 — Pi RPC AgentExecutionAdapter
F9 — A2UI web core and permanent Lit renderer
F10 — Focusa Svelte Custom Elements
F11 — UIAI Engine Eval contracts and first scenario
F12 — one real Context action through generated UI
```

F0–F12 form Alpha 0.

---

## 10. Cross-Functional Alpha

```text
Alpha 1 — real Markdown/code and PDF Context ingestion, provenance, retrieval, generated UI
Alpha 2 — grounded Role, approval, Grill Interview, autosave, close, resume
Alpha 3 — Spec 120 cycle, approval, provider-neutral plan, real Beads task
Alpha 4 — Workpoint, Work Rail, Evidence, closure reconciliation, Receipt
Alpha 5 — UIAI artifact, Evidence, canonical event, automatic Work Surface update
Alpha 6 — Pi and isolated UIAI Work Surfaces, targeted steering, restart proof
Alpha 7 — General, Software, Research projections over identical state
Alpha 8 — permanent nontechnical dogfood traversal
```

Every Alpha ticket crosses requirement ID, greater primitive, schema, reducer, API, generated clients, generated UI, real integration, tests, UIAI Eval when browser-facing, Evidence, and Receipt.

### 10.1 Critical paths to the vertical slices

All listed feeder paths are mandatory merge gates, not optional implementation suggestions. Branches may execute in parallel, but every branch must reach the Alpha merge gate before the vertical slice closes.

| Vertical slice | Critical feeder paths |
|---|---|
| Alpha 1 — Context ingestion | `F12 → C1 → C2 → C3 → Alpha 1`; `F12 → U1 → U2 → Alpha 1` |
| Alpha 2 — Role and Grill Interview | `Alpha 1 → Alpha 2`; `C3 → RI1 → RI2 → RI3 → Alpha 2` |
| Alpha 3 — Spec 120 and Beads task | `Alpha 2 → Alpha 3`; `RI3 → ST1 → ST2 → ST3 → Alpha 3`; `F8 → P1 → ST2 → ST3 → Alpha 3` |
| Alpha 4 — Workpoint and Work Rail | `Alpha 3 → Alpha 4`; `ST3 → ST4 → Alpha 4`; `F12 → M1 → M2 → Alpha 4`; `ST4 → M2 → Alpha 4` |
| Alpha 5 — UIAI artifact live refresh | `Alpha 4 → Alpha 5`; `U1 → U2 → U3 → Alpha 5`; `F12 → M1 → M3 → M4 → M5 → U3 → Alpha 5` |
| Alpha 6 — isolated Pi/UIAI Work Surfaces | `Alpha 5 → Alpha 6`; `M4 → M5 → M6 → Alpha 6`; `F6 → M6 → Alpha 6` |
| Alpha 7 — domain projections | `Alpha 6 → Alpha 7`; `F6 → V1 → V2 → V3 → V6 → Alpha 7`; `V2 → V4 → V6 → Alpha 7`; `V2 → V5 → V6 → Alpha 7`; `U1 → V5 → V6 → Alpha 7` |
| Alpha 8 — permanent dogfood traversal | `Alpha 7 → Alpha 8`; `U3 → U4 → Alpha 8`; `M1 → M2 → M7 → U4 → Alpha 8`; `M1 → M3 → M7 → U4 → Alpha 8`; `F7 → Q1 → Q2 → Alpha 8`; `F12 → Q3 → Alpha 8` |

Every path terminates in the same functional integration traversal:

```text
requirement → greater primitive → schema → reducer/event → typed API
→ generated TypeScript clients and portable OpenAPI/JSON Schema contracts → Operation Registry action binding
→ A2UI/Lit + trusted Focusa Svelte element → real integration
→ focused/runtime tests → UIAI Eval when browser-facing → Evidence → Receipt → closure
```

The machine-readable authority for these paths is `docs/contracts/spec135-delivery-dag.v1.yaml#critical_path_contract`; CI must prove that every adjacent path step is a real blocking edge and that all eight Alpha gates are covered.

Mocks, static cards, transcript-only Interview, manual refresh, ambient scope, placeholder success, provider-only closure, and browser proof outside UIAI Engine Eval do not satisfy an Alpha slice.

---

## 11. Parallel lanes

After F4 stabilizes generated operation contracts:

```text
C — Context, Docling, retrieval, Google Drive, claims, contradictions
R/I — Role Composer, Grill Interview, compendium, autosave, resume
S/T — Spec 120, task plan, adapters, Workpoint, Receipt
M — Mission Canvas, Work Surfaces, multiplexing, steering, restoration
U — UIAI artifacts, browser contexts, Eval, accessibility
V — ontology/domain packs, verticals, renderers, terminology
P — providers, connectors, migrations, client parity, AG-UI
Q — security, licenses, SBOM, performance, recovery, accessibility
```

Use isolated worktrees, writer leases, explicit Workpoints, and exact Attachments. Parallelism never means shared dirty-writer authority.

---

## 12. Performance laws

- use bounded purpose-specific read models;
- use durable event invalidation and targeted refetch;
- keep large artifacts behind handles;
- process extraction, indexing, and synchronization outside request locks;
- virtualize large lists and histories;
- lazy-load rich artifacts;
- use provider-native incremental sync;
- expose queued, processing, throttled, paused, degraded, failed, and completed states;
- benchmark the selected local retrieval stack before adding infrastructure;
- preserve project/workstream/session isolation under load.

---

## 13. UX completeness laws

Every blocked, stale, disconnected, unauthorized, empty, or failed state explains what happened, what remains safe, what was retained, and the exact recovery action.

Use progressive disclosure, not feature removal. Autosave and resume Context, Role, Interview, Spec progress, task edits, workspace selection, and Work Surface state.

Every stage exposes one primary next action. Consequential mutations use preview, confirmation, commit, event, Evidence, and Receipt.

Dynamic capability truth controls availability. Dead buttons and pretend providers are forbidden.

---

## 14. Ticket and merge contract

Every ticket includes:

```yaml
requirement_refs: []
blocking_refs: []
primitive_submission:
reuse_assessment:
framework_lock_refs: []
files_and_packages: []
core_types: []
reducer_actions: []
api_operations: []
generated_contracts: []
generated_ui:
uiai_eval:
migration:
security:
accessibility:
performance:
tests:
evidence:
receipts:
definition_of_done:
not_done_if: []
```

A PR merges only when generated contracts are current, reuse and primitive records are complete, focused tests pass, integration Evidence exists, affected UIAI Eval scenarios pass, license/SBOM gates pass, and the dogfood path remains operational.

---

## 15. Permanent dogfood gate

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

Every user-facing step uses generated nontechnical UI. Browser proof uses UIAI Engine Eval exclusively.

---

## 16. Acceptance and closure

Spec 135D is accepted when:

1. Every requirement in 135A–135K appears in the validated machine-readable graph.
2. Foundation F0–F12 and Alpha 1–8 are implemented and proven.
3. All parallel lanes preserve shared contracts and exact scope.
4. Reuse assessments and primitive submissions exist for every ticket.
5. No accepted requirement is omitted or silently deferred.
6. No duplicate canonical authority or browser test runtime is introduced.
7. The complete dogfood traversal passes with Evidence and Receipts.

Spec 135D cannot close while any required ledger item is missing, partial, open, or blocked; while browser proof bypasses UIAI Engine Eval; while Playwright exists in Focusa; while general behavior is trapped in C.R.I.S.T./client code; or while any client/provider/connector/vertical/proof requirement is absent from the closure graph.

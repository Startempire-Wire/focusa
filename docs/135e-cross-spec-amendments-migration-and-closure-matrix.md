# Spec 135E — Cross-Spec Amendments, Migration, Compatibility, and Closure Matrix

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135E.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs current implementation decisions.

---

## 0. One-line definition

Spec 135 composes existing Focusa primitives into one professional-workspace, C.R.I.S.T. Project Genesis, domain-semantic, real-time generated UI, and multiplexed Mission Canvas product path while preserving primitive ownership, exact scope, compatibility, migration, Evidence, Receipts, and complete closure.

---

## 1. Amendment law

```text
Existing specs retain primitive ownership.
Spec 135 owns complete integrated product delivery.
Cross-spec work remains in the Spec 135 closure DAG.
A client or adapter never overrides canonical Focusa state.
The frozen Delivery Contract resolves current implementation conflicts.
```

---

## 2. Precedence law

When documents disagree:

1. canonical reducer-backed Focusa truth wins over projections;
2. explicit operator direction and the latest approved amendment win over older direction;
3. the Spec 135 Delivery Contract governs framework, testing, sequencing, browser-proof, generated-UI, model-execution, and compatibility decisions;
4. Specs 98 and 104 govern typed project/workstream/attachment scope and prohibit authority-bearing singleton state;
5. Specs 40 and 41 govern Instances/Sessions/Attachments and concurrent proposal resolution;
6. Spec 133 governs durable sessions, runs, writer leases, and worktree isolation;
7. primitive-owning specs govern internal primitive semantics;
8. Spec 135 owns integrated product closure;
9. 135A governs Workspace View and vertical UX;
10. 135B governs C.R.I.S.T. Project Genesis;
11. 135C governs UIAI artifact and live-refresh integration;
12. 135D governs complete implementation, no-deferral, and performance;
13. 135E governs migration, precedence, compatibility, and closure;
14. 135F governs domain-semantic composition;
15. 135G governs Mission Canvas multiplexing and browser-context isolation;
16. 135H governs Grill Interview and implementation acceleration;
17. 135I governs real-time generated UI and typed actions;
18. 135J governs Operation Registry, durable stream, shared envelopes, and runtime reuse;
19. 135K governs UXP/UFI and nontechnical usability proof.

---

## 3. Naming constitution

```text
UIAI Engine Cockpit
  The only product/interface using Cockpit.

Focusa Mission Canvas
  Multiplexed interactive Focusa/Pi workspace projection.

Work Surface
  One tab, pane, split, or detached view in Mission Canvas.

Mission Deck
  Standalone guided Focusa PWA/experience.

Spec Workbench
  Spec 120 adversarial specification environment.
```

Non-UIAI uses of Cockpit are deprecated and migrate to Mission Canvas, Work Surface, Runtime View, or another precise term.

---

## 4. Cross-spec ownership matrix

| Existing owner | Preserved primitive ownership | Required Spec 135 integration |
|---|---|---|
| Specs 14 | UXP/UFI storage, learning, citations, transparency | All generated UI adaptation reuses UXP/UFI and cannot change authority. |
| Specs 38–41 | threads, lifecycle, Instances/Sessions/Attachments, proposal resolution | Work Surfaces expose and route exact session/attachment state and visible contention. |
| Spec 43 | local-first multi-device sync | Device-local focus remains projection; synchronized canonical state preserves scope. |
| Specs 45–50, 61, 70, 74, 75, 77 | ontology, domain cognition, statuses, identity, projections, governance | 135F provides one versioned semantic substrate and domain-pack composition path. |
| Spec 72 | RoleProfile, capabilities, permissions, responsibilities | Role Composer creates project-scoped Role Profiles without granting permission. |
| Spec 88 | Workpoint continuity | active work and Work Rail bind to Workpoints; workspace/project profile does not replace them. |
| Specs 98/104 | ProjectRootKey, WorkstreamKey, AttachmentKey, anti-singleton authority | every API, client, session, browser context, and generated surface preserves typed scope. |
| Spec 100 | Context Cognition | Context corpus feeds bounded advisory selection; it is never dumped wholesale into prompts. |
| Specs 107/109/111 | spec-first lifecycle, typed AX API, context delivery | C.R.I.S.T. follows Spec-first lifecycle, generated contracts, preview/commit, and bounded bootstrap. |
| Spec 116 | provider-neutral work and closure truth | Tasks and Work Rail use real adapters and Focusa closure verification. |
| Specs 117/117A | Mission Deck, onboarding, living-field UX | adds Quick Mission versus Full Genesis, Mission Canvas launch, and nontechnical generated UI. |
| Spec 119 | Receipts | Genesis, approvals, task materialization, session changes, and completion link Receipts. |
| Spec 120 | adversarial Workbench and task decomposition | C.R.I.S.T. Spec and Tasks invoke Spec 120; no second spec engine. |
| Specs 121/121A | typed Svelte/Tauri and compact living-field surfaces | clients consume shared generated contracts, Mission Canvas, and generated UI projections. |
| Spec 124 | project creation, selection, First Mission | adds project genesis/context/role/interview/spec/tasks; First Mission remains Quick Mission. |
| Spec 125 | mandatory Trajectory and HLT | Project Genesis defines/proposes HLT/MLG/STG/Waypoints and degrades loudly when absent. |
| Spec 130 | bounded compaction/context firewall | generated summaries and session state enter context through bounded refs. |
| Spec 133 | durable sessions/runs/leases/worktrees | model execution, autonomous work, and parallel implementation use governed sessions and leases. |
| UIAI Engine | browser, Documents, diagnostics, FPV, artifacts, browser Eval | Focusa consumes typed scoped artifacts and Evidence; UIAI owns all browser proof. |

---

## 5. Current framework and proof amendments

The current Delivery Contract fixes:

```text
A2UI v0.9.1 + web_core + permanent Lit renderer
Focusa Svelte Custom Elements
native SQLite replay + broadcast tail
AG-UI external compatibility after native stabilization
JSON Schema 2020-12 + OpenAPI 3.0.3
TypeScript generated clients and language-neutral OpenAPI/JSON Schema contracts
Pi RPC AgentExecutionAdapter / Spec 133 model execution
UIAI Engine Eval for browser proof
no Playwright in Focusa
no Vercel AI SDK runtime ownership
```

Older requirements for a complete custom Svelte A2UI renderer, Playwright proof, OpenAPI 3.1 transport, or AG-UI on the native Alpha critical path are superseded.

---

## 6. Existing project migration

Existing projects continue operating without a completed Project Genesis record.

Generated UI presents:

```text
Project Genesis not completed
Start full Project Genesis
Import from existing project
Continue current work
```

Import can use project markers, repository docs/code, ProjectIdentity, Project Card, Trajectory, Workpoints, work items, Evidence, Receipts, and settings. Every inferred field is labeled and reviewed before promotion.

The existing First Mission remains Quick Mission.

---

## 7. Canonical storage and projection migration

Canonical Project Genesis, Context claims, Role Profiles, Interview records, Spec references, task plans, workspace selection, domain semantics, sessions, generated surfaces, UXP/UFI, and artifact links live in reducer-backed Focusa state.

Project files, Pi settings, browser storage, Svelte stores, UIAI state, and connector caches can identify, cache, import, export, or project state. They cannot become canonical authority.

Approved repository specs and ADR/glossary projections use preview, approval, governed write, Evidence, and Receipt.

---

## 8. Runtime and snapshot compatibility

Every new canonical record, event, read model, generated surface, catalog, action binding, and client contract carries schema/version compatibility metadata.

Create:

```yaml
schema: focusa.compatibility_lock.v1
focusa_runtime:
focusa_api:
operation_registry:
tool_result:
event_stream:
a2ui_protocol:
a2ui_catalog:
ag_ui_adapter:
pi_runtime:
uiai_engine:
uiai_focusa_client:
docling:
embedding_profile:
domain_pack_versions: []
minimum_reader_versions:
minimum_writer_versions:
```

Clients and UIAI Engine perform startup version/capability handshake. Mismatch produces blocked status, exact incompatible component, retained safe state, required upgrade action, and advanced details. Silent guessing is forbidden.

---

## 9. Event and stream migration

Native stream migration:

```text
add stable event ID and sequence
→ expose cursor / Last-Event-ID
→ replay missed scoped events from SQLite
→ attach broadcast live tail
→ deduplicate
→ produce A2UI snapshot/delta
```

AG-UI translates the native stream for external compatibility. It never owns canonical history.

Legacy visual-workflow evidence routes migrate through typed Workspace Artifact/Evidence operations with exact scope, ECS handle, Evidence link, event, and Receipt. Compatibility aliases remain only during an explicit expand-contract chain.

---

## 10. Contract migration

Canonical contracts:

```text
JSON Schema 2020-12
OpenAPI 3.0.3
openapi-typescript/openapi-fetch
external adapters generated from published OpenAPI outside Focusa core
Operation Registry
A2UI catalog and action bindings
```

Generate one release artifact with schemas, OpenAPI, operation registry, catalog, compatibility lock, and hashes. UIAI Engine consumes an immutable commit or release digest.

Handwritten duplicate DTOs are deprecated and removed after every reader/writer passes compatibility proof.

---

## 11. Browser proof migration

UIAI Engine Eval becomes the exclusive browser-proof path for Focusa.

Required migration:

```text
inventory any browser-test dependency/config/fixture
→ replace scenario with uiai.focusa_ui_eval_scenario.v1
→ produce UIAI session/context/screenshots/diagnostics/accessibility/visual refs
→ link Focusa Evidence and Receipts
→ remove duplicate browser-test dependency
```

Playwright Test, Playwright Library, Playwright CLI, Playwright MCP, `@playwright/test`, Playwright configs, and Playwright fixtures are forbidden in Focusa.

---

## 12. Generated UI migration

```text
add A2UI web_core and permanent Lit renderer
→ generate trusted catalog and action bindings
→ add Focusa Svelte Custom Elements
→ implement generated Context surface
→ widen through Role, Interview, Spec, Tasks, continuation
→ add terminal projections
```

Do not build a complete alternate Svelte renderer. Do not put canonical state, permissions, or business logic in components.

---

## 13. Model-execution migration

```text
add focusa.agent_execution_adapter.v1
→ implement PiRpcExecutionAdapter
→ run model-backed work in Spec 133 governed sessions
→ return structured results
→ reduce to canonical state
→ emit Evidence/Receipt and UI delta
```

Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, Vercel AI Gateway authority, and other duplicate agent runtimes are forbidden.

---

## 14. Machine-readable closure migration

Before implementation decomposition, create:

```text
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Every normative requirement maps to stable IDs, owners, dependencies, tasks, greater primitives, APIs, generated surfaces, UIAI Eval scenarios, tests, Evidence, Receipts, migration, and closure state.

---

## 15. Deprecation law

A deprecated schema, route, alias, renderer, provider adapter, snapshot, or client remains only under an explicit migration task containing replacement, readers/writers, conversion, compatibility window, downgrade behavior, proof, and removal gate.

A compatibility alias cannot silently become permanent architecture.

---

## 16. Closure matrix

| Area | Required closure proof |
|---|---|
| C.R.I.S.T. | Context, Role, Interview, Spec, Tasks complete through generated UI. |
| Runtime | reducer-backed canonical state, exact scope, replay, recovery, restart. |
| Mission Canvas | concurrent Work Surfaces, targeted interaction, contention, restoration. |
| Browser | UIAI contexts/targets, isolation, artifacts, diagnostics, UIAI Engine Eval. |
| Contracts | generated JSON Schema, OpenAPI 3.0.3, TypeScript clients, portable external-adapter contracts, compatibility lock. |
| Generated UI | A2UI web core/Lit, trusted Svelte Custom Elements, typed actions, native stream. |
| UXP/UFI | transparent nontechnical baseline, bounded adaptation, evaluator benchmark. |
| Work | real adapters, Workpoints, Evidence, closure reconciliation, Receipts. |
| Domain | ontology core, domain packs, vertical projections, verification policy. |
| Clients | Pi, Mission Deck/PWA, UIAI Engine Cockpit, menubar, TUI, API, CLI parity. |
| Security/supply chain | scope isolation, OAuth, redaction, license notices, SBOM, recovery. |
| Delivery | every machine-readable ledger requirement verified. |

---

## 17. Final closure blockers

Spec 135 cannot close while:

- any companion 135A–135K is incomplete;
- any accepted requirement is missing from the delivery graph;
- current implementation is represented as more complete than Evidence proves;
- generated UI lacks a nontechnical complete/recover/resume path;
- browser proof bypasses UIAI Engine Eval;
- Playwright or another duplicate browser-test system exists in Focusa;
- AG-UI replaces canonical state or blocks the native Alpha path;
- a complete custom renderer duplicates A2UI web core/Lit;
- Vercel AI SDK or another framework duplicates Focusa/Pi authority;
- canonical state exists only in client/session/cache files;
- session/browser scope can bleed;
- provider adapters are enum-only or mock-only;
- Evidence, Receipts, compatibility, migration, accessibility, security, performance, or parity proof is missing;
- general reusable behavior remains trapped in C.R.I.S.T. or client code;
- the permanent dogfood traversal does not pass with actual Evidence and Receipts.

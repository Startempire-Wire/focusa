# Spec 135 Series — Current Authoritative Delivery Contract

**Status:** current, normative, frozen companion and delivery manifest  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Authority:** This document resolves implementation ordering, framework ownership, testing, generated-UI, cross-repository compatibility, and decomposition conflicts across the Spec 135 series. When older companion wording conflicts with this contract, this contract governs.

Spec 135 and every companion below form one required implementation and closure set. The series is frozen at **135K**. No additional lettered companion is created for implementation clarification; corrections are consolidated into this delivery contract and the affected existing documents.

## Locked release compatibility

The `v0.9.141-locked-release` delta is classified with zero unknown impacts in [`docs/contracts/135-locked-release-compatibility-delta.v1.yaml`](contracts/135-locked-release-compatibility-delta.v1.yaml). Additive temporal, epistemic, instruction-integrity, working-subpath, agent-capability, startup-binding, cross-project-isolation, scoped-refresh, and advisory-boundary changes reuse existing Spec135A–135K governance and Mission Canvas substrates; no Spec135L exists. Every admitted change declares affected primitives, documents, contracts, surfaces, tests, compatibility, migration, rollback, and agent handoff in that packet.

| Order | Spec                                                                                                            | Required subject                                                                                                |
| ----: | --------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
|     1 | [135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)                              | Master product, authority, workspace, C.R.I.S.T., and closure contract                                          |
|     2 | [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md)                                  | Workspace projection, Mission Canvas, Work Rail, themes, and vertical UX                                        |
|     3 | [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md)                                         | Context, Role, Interview, Spec, Tasks, and Project Genesis state                                                |
|     4 | [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md)                                        | UIAI artifacts, browser identity, FPV, and live refresh                                                         |
|     5 | [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md)                  | Complete implementation graph, framework reuse, performance, and no-deferral law                                |
|     6 | [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md)                                              | Cross-spec amendments, migration, compatibility, and closure                                                    |
|     7 | [135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md)              | Ontology core, semantic graphs, domain packs, verification, and reactive context                                |
|     8 | [135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Multiplexed Work Surfaces, sessions, attachments, browser isolation, and restoration                            |
|     9 | [135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md)                     | Grill Interview, Cross-Functional Alpha, decided OSS stack, and speed law                                       |
|    10 | [135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md)              | Real-time generated C.R.I.S.T. UI, nontechnical onboarding, A2UI, and generated action surfaces                 |
|    11 | [135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md)                  | Core API Operation Registry, durable replayable stream, shared envelopes, and runtime reuse                     |
|    12 | [135K](135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md)                 | Canonical UXP/UFI reuse, adaptive generated UI, transparent friction learning, and nontechnical usability proof |

## 1. Final framework and ownership decisions

These are implementation decisions. Agents MUST NOT reopen them as option menus.

### 1.1 Browser execution and proof

UIAI Engine owns all browser-facing execution, evaluation, and proof:

```text
navigation
browser actions and form interaction
DOM and accessibility snapshots
screenshots and viewport changes
browser contexts and authentication state
console and network diagnostics
responsive verification
visual comparison
browser recovery and rehydration
browser evaluation scenarios
browser proof artifacts
```

Focusa repositories and implementation tasks MUST NOT introduce Playwright Test, Playwright Library, Playwright CLI, Playwright MCP, `@playwright/test`, `playwright.config.*`, or Playwright browser fixtures.

All browser, end-to-end, visual, responsive, reconnect, and browser accessibility proof for Spec 135 MUST use **UIAI Engine Eval**, including the eval-capable development version where required.

### 1.2 Generated UI

The canonical generated-UI stack is:

```text
A2UI v0.9.1
@a2ui/web_core/v0_9
@a2ui/lit/v0_9
Focusa Svelte Custom Elements registered in the Focusa A2UI catalog
```

The maintained Lit renderer is the permanent web rendering shell for A2UI v0.9.1. Focusa MUST NOT build a complete second Svelte A2UI renderer, duplicate the A2UI message processor, duplicate `SurfaceModel`, or duplicate A2UI data binding.

Focusa-specific interactions are authored in Svelte and compiled as Custom Elements registered in the trusted A2UI catalog.

### 1.3 Streaming

The primary product stream is:

```text
Focusa SQLite canonical event history
+ stable event IDs and sequence
+ cursor / Last-Event-ID replay
+ existing broadcast live tail
+ scoped invalidation
+ A2UI messages and snapshots
```

AG-UI is a required external compatibility adapter over this native stream. AG-UI MUST NOT own canonical state, event history, approvals, tool authority, or persistence, and MUST NOT block the first complete native C.R.I.S.T. traversal.

### 1.4 Model execution

Model-backed C.R.I.S.T. behavior uses the existing Focusa harness and Spec 133 session architecture:

```text
Focusa typed operation
→ daemon-owned governed execution session
→ Pi RPC AgentExecutionAdapter reference implementation
→ structured result
→ Focusa reducer
→ Evidence / Receipt
→ generated UI delta
```

Vercel AI SDK runtime facilities are not Focusa dependencies. The following are forbidden as runtime owners or client dependencies:

```text
WorkflowAgent
ToolLoopAgent
AI SDK UI / @ai-sdk/svelte
Vercel AI Gateway as a required dependency
Vercel-owned durable workflow or tool authority
```

Vercel documentation and implementation patterns may be studied as references only.

### 1.5 Contracts and generated clients

Canonical contract split:

```text
JSON Schema 2020-12
  canonical domain and generated-surface schemas

OpenAPI 3.0.3
  canonical HTTP operation contract
```

Generated clients:

```text
Rust: Serde + Schemars + Utoipa
TypeScript: openapi-typescript + openapi-fetch
UIAI Engine: external adapter owned outside Focusa core and derived from the published OpenAPI contract
A2UI: generated catalog schemas and action bindings
```

OpenAPI 3.1 is not the transport contract for Spec 135. Handwritten duplicate Focusa DTOs or operation registries in UIAI Engine, Pi, Svelte, or connector code are forbidden when generation can represent the contract.

### 1.6 Shared UI and data stack

The decided reusable stack remains:

```text
SvelteKit 2
Svelte 5
Tailwind CSS 4
shadcn-svelte
Bits UI
Paneforge
TanStack Query
TanStack Table
TanStack Virtual
Svelte Flow
PDF.js
CodeMirror Merge
Apache ECharts
```

These libraries provide generic UI mechanics. Focusa code owns only domain-specific interaction, projection, visual grammar, accessibility rules, and authority-bound action adapters.

## 2. Complete implementation graph and Cross-Functional Alpha

The complete closure DAG contains every accepted requirement. The Cross-Functional Alpha is the mandatory earliest integration route and does not remove anything from the closure DAG.

### Foundation Train

```text
F0 — freeze the series at 135K and compile this Delivery Contract
F1 — create machine-readable requirement, DAG, parity, framework, and proof matrices
F2 — generate JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generate TypeScript clients and validate portable OpenAPI/JSON Schema contracts
F4 — generate the Operation Registry, capability projection, and UI action bindings
F5 — centralize ToolResult and error-envelope construction
F6 — implement durable replayable Focusa events over SQLite plus the broadcast tail
F7 — implement capability/permission projection and compatibility handshake
F8 — implement AgentExecutionAdapter with Pi RPC as the reference adapter
F9 — integrate A2UI web_core and the permanent Lit renderer
F10 — register the initial Focusa Svelte Custom Elements
F11 — implement UIAI Engine Eval contracts and the first browser proof scenario
F12 — complete one real Context operation end to end through generated UI
```

F0–F12 form Alpha 0.

### Cross-Functional Alpha

```text
Alpha 1 — real Markdown/code and PDF Context ingestion, provenance, indexing, retrieval, generated UI
Alpha 2 — grounded Role draft, approval, Grill Interview, autosave, close, resume
Alpha 3 — Spec 120 handoff, real adversarial cycle, approval, real Beads task
Alpha 4 — Workpoint, Work Rail, Evidence, closure reconciliation, Receipt
Alpha 5 — UIAI artifact, Evidence link, event invalidation, automatic Work Surface refresh
Alpha 6 — Pi Work Surface plus isolated UIAI browser Work Surface, targeted steering, restart proof
Alpha 7 — General, Software, and Research projections over the same canonical state
Alpha 8 — permanent Spec 135 dogfood traversal
```

### Parallel lanes after F4

```text
C — Context, Docling, retrieval, Google Drive, claims, contradictions
R/I — Role Composer, Grill Interview, compendium, autosave, resume
S/T — Spec 120 integration, tasks, Beads, Workpoint, Receipt
M — Mission Canvas, Work Surfaces, multiplexing, steering, restoration
U — UIAI artifacts, browser contexts, FPV, Eval scenarios, accessibility
V — domain packs, vertical renderers, terminology, artifact projections
P — remaining providers, connectors, migration, parity, AG-UI compatibility
Q — security, licenses, SBOM, performance, recovery, accessibility hardening
```

## 3. Greater Focusa primitive-submission rule

Every feature MUST be implemented in this order:

```text
general reusable Focusa primitive
→ reducer and canonical state
→ typed Focusa API
→ generated cross-language contracts
→ C.R.I.S.T. interaction projection
→ client renderer
→ UIAI Engine Eval proof
→ Evidence
→ Receipt
```

The following are general Focusa primitives and MUST NOT be trapped inside Project Genesis, C.R.I.S.T., a Svelte component, or an A2UI component:

- operation descriptors;
- capabilities and permissions;
- preview/commit;
- event replay;
- result envelopes;
- Evidence and Receipts;
- Context artifacts and claims;
- Role Profiles;
- Interview records;
- tasks and Workpoint binding;
- Instances, Sessions, Attachments, and Work Surfaces;
- generated surface envelopes and action bindings;
- UXP/UFI;
- connector and provider health;
- browser artifact references.

C.R.I.S.T.-specific code is limited to stage orchestration, readiness, interaction intent, stage language, and stage projection composition.

## 4. Machine-readable decomposition inputs

Before implementation decomposition, create and validate:

```text
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Every normative requirement has a stable requirement ID, owner, dependency edge, implementation task, client surface, UIAI Eval scenario, test, Evidence requirement, Receipt requirement, migration requirement, and closure status.

Agents MUST NOT infer the delivery graph from prose alone.

## 5. Required implementation-reality audit and directives

Before decomposition, read:

1. [Spec 135 Real-Time Generated UI Speed and Core Integration Audit](current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md)
2. [Spec 135 implementation acceleration directive](agent/spec135-implementation-acceleration-directive.md)
3. [Spec 135 real-time generated UI directive](agent/spec135-real-time-generated-ui-directive.md)
4. [Spec 135 UXP/UFI generated UI directive](agent/spec135-uxp-ufi-generated-ui-directive.md)

The audit distinguishes implemented code, reusable seams, required migrations, and normative targets. Docs-only behavior MUST NOT be treated as implemented.

## 6. Progressive product and merge laws

Every merge leaves a truthful working product. Unavailable capability is rendered as unavailable, blocked, credentials required, dependency incomplete, or upgrade required. No mock provider, static success card, dead control, or placeholder result is presented as operational.

Every implementation PR MUST:

1. reference stable requirement IDs;
2. update the machine-readable ledger;
3. submit reusable behavior to its greater Focusa primitive owner;
4. update generated contracts and TypeScript clients and portable OpenAPI/JSON Schema contracts when contracts change;
5. include unit, contract, generated-UI, and UIAI Engine Eval proof as applicable;
6. link Evidence and Receipt references;
7. preserve the permanent dogfood traversal;
8. avoid unapproved frameworks and duplicate runtimes;
9. keep project, workstream, attachment, and browser-context scope explicit;
10. leave the repository in a working capability-gated state.

## 7. Permanent dogfood gate

The following remains operational after every merge:

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
→ Focusa rich Mission Canvas with the same live session
→ real Work Surface switch/split/rehydration
→ pause
→ restart
→ resume exact state
```

Every user-facing step is completed through generated plain-language UI. Browser portions are proven exclusively through UIAI Engine Eval.

## 8. Closure rule

```text
No companion is optional.
No accepted requirement is deferred through sequencing language.
A nontechnical generated UI path is required for every C.R.I.S.T. stage.
Generated UI uses the core Operation Registry, durable Focusa event stream,
capability and permission projection, ToolResult envelopes, Evidence, and Receipts.
UX adaptation uses Spec 14 UXP/UFI and never changes authority or proof requirements.
Browser proof uses UIAI Engine Eval and never Playwright.
The full series closes only when every machine-readable ledger entry is verified.
```

## 9. Current reading order

```text
135
→ this Delivery Contract
→ implementation-reality audit
→ agent directives
→ 135A → 135B → 135C → 135D → 135E → 135F → 135G → 135H → 135I → 135J → 135K
→ machine-readable delivery graph
→ locked-release compatibility/delta packet
```

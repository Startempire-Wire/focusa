# Spec 135H — Cross-Functional Alpha, Grill Interview Strategy, and Full-Series Implementation Acceleration

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md), and [Spec 135E](135e-cross-spec-amendments-migration-and-closure-matrix.md)  
**Closure relationship:** mandatory acceleration amendment; Spec 135 decomposition and implementation cannot proceed without applying this document.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs any conflict.

---

## 0. One-line definition

Implement the complete Spec 135 series fastest by establishing one production-shaped end-to-end path through every major function immediately, keeping that path continuously operational, submitting reusable behavior to the greater Focusa primitives, and widening every remaining requirement in parallel through the decided Focusa, UIAI Engine, Pi, and open-source foundations.

---

## 1. Normative acceleration law

Agents MUST:

```text
use the decided stack;
reuse existing Focusa, UIAI Engine, and Pi ownership;
produce the Cross-Functional Alpha first;
use vertical tracer-bullet tickets;
submit general behavior to greater Focusa primitives;
open all unblocked parallel expansion lanes after shared contracts stabilize;
keep every accepted requirement in the machine-readable closure graph;
retain a continuously working dogfood traversal;
and request a versioned amendment only after a decided dependency fails a recorded conformance gate.
```

Agents MUST NOT:

- present framework, product, sequence, or architecture option menus;
- defer accepted requirements through “later,” “post-MVP,” “future enhancement,” or similar wording;
- substitute mocks, static cards, docs-only types, or placeholder success for integration;
- build a second runtime, event store, browser engine, task authority, Interview store, Evidence store, session model, vector database, document parser, generated-UI protocol, or desktop shell;
- trap reusable behavior inside C.R.I.S.T., Project Genesis, a client, or a component.

---

# Part I — Grill-with-Docs Interview Strategy

## 2. Fixed strategy

Focusa MUST adapt the MIT-licensed `grill-with-docs`, `grilling`, and `domain-modeling` disciplines from `mattpocock/skills` into:

```text
focusa.interview.strategy.grill-with-docs.v1
```

The upstream MIT copyright and permission notice MUST be preserved in the adapted skill package and third-party notices.

Ownership:

```text
Focusa Interview Engine
  Canonical questions, answers, decisions, provenance, sensitivity,
  supersession, readiness, project scope, and Spec handoff.

Grill-with-Docs Interview Strategy
  Fact lookup, questioning discipline, branch traversal, recommendation,
  terminology challenge, edge cases, and decision/glossary/ADR candidates.
```

The strategy is not a second Interview database, workflow authority, repository writer, or canonical source.

## 3. Interview Strategy contract

```rust
pub trait InterviewStrategy {
    async fn generate_next_question(
        &self,
        context: InterviewContext,
    ) -> NextQuestionProposal;

    async fn evaluate_answer(
        &self,
        question: InterviewQuestion,
        answer: InterviewAnswer,
    ) -> AnswerAssessment;

    async fn assess_readiness(
        &self,
        state: InterviewState,
    ) -> InterviewReadinessAssessment;
}
```

```yaml
schema: focusa.interview_next_question_proposal.v1
strategy_id: focusa.interview.strategy.grill-with-docs.v1
strategy_version: 1
session_id:
parent_question_id:
decision_branch_id:
question:
reason_for_asking:
triggering_gap:
recommendation:
recommendation_basis_refs: []
environment_facts_checked: []
contradiction_refs: []
linked_context_refs: []
linked_spec_sections: []
domain_term_candidates: []
architecture_decision_candidates: []
decision_required:
priority: blocker | high | normal | optional
answer_type:
readiness_effect:
stop_condition:
```

## 4. Interview laws

### Fact before question

Before asking, retrieve discoverable answers from project files, code, repository history, Focusa state, connected sources, UIAI research, provider state, and accepted claims. Ask the operator only for preferences, tradeoffs, priorities, authority, acceptance boundaries, ambiguous intent, unresolved contradictions, or unavailable operator-owned facts.

### Recommendation

Every operator-decision question MUST contain one recommendation and cited basis. The operator answer remains authoritative.

### One question

Present one primary question at a time with supporting facts, recommendation, consequences, sources, and branch progress.

### Branch traversal

Complete one dependency branch until prerequisite facts, dependent decisions, contradictions, glossary candidates, ADR candidates, and stop conditions are resolved. Then advance to the highest-value unresolved branch.

### Fatigue control

Rank by blocker value and downstream dependency count, collapse duplicates, autosave every answer, checkpoint each branch, permit pause/resume after every answer, stop low-impact questioning, and reopen a branch when new context materially invalidates it.

## 5. Required tranches

```text
Discovery Grill
Boundary Grill
Failure Grill
Evidence Grill
Architecture Grill
Spec-Readiness Grill
```

Domain packs can add stricter overlays. They MUST NOT remove the core tranches.

## 6. Governed glossary and ADR candidates

```yaml
schema: focusa.domain_term_candidate.v1
candidate_id:
term:
proposed_definition:
conflicting_terms: []
source_question_ref:
source_answer_ref:
context_refs: []
status: candidate | approved | rejected | superseded
```

```yaml
schema: focusa.architecture_decision_candidate.v1
candidate_id:
title:
context:
decision:
alternatives_considered: []
consequences: []
source_question_refs: []
source_answer_refs: []
status: candidate | approved | rejected | superseded
```

Direct repository writes during Interview are forbidden. Approved projections follow preview, operator approval, governed write, Evidence, and Receipt.

---

# Part II — Cross-Functional Alpha

## 7. Two-axis delivery model

```text
Axis A — Full Completion DAG
  Every accepted requirement and final closure proof.

Axis B — Cross-Functional Alpha
  Earliest production-shaped traversal through every major function.

Axis C — Parallel expansion lanes
  Every unblocked connector, provider, vertical, client, renderer,
  migration, security, performance, accessibility, and proof task.
```

The Alpha is not reduced scope. It establishes the integration spine used by the complete product.

## 8. Foundation Train

```text
F0 — freeze Spec 135 at 135K and compile the Delivery Contract
F1 — create machine-readable requirement, DAG, parity, framework, and proof matrices
F2 — generate JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generate TypeScript and Go clients
F4 — generate Operation Registry, capability projection, and UI action bindings
F5 — centralize ToolResult and error-envelope construction
F6 — implement durable SQLite replay plus the broadcast live tail
F7 — implement capability/permission projection and compatibility handshake
F8 — implement Pi RPC AgentExecutionAdapter
F9 — integrate A2UI web_core and permanent Lit renderer
F10 — register initial Focusa Svelte Custom Elements
F11 — implement UIAI Engine Eval contracts and first browser proof
F12 — complete one real Context action through generated UI
```

F0–F12 form Alpha 0.

## 9. Cross-Functional Alpha slices

### Alpha 1 — Real Context

```text
create/bind project
→ ingest Markdown/code
→ ingest real PDF through UIAI Documents and Docling
→ preserve provenance/page refs
→ HybridChunker
→ FTS5/sqlite-vec/fastembed indexing
→ bounded retrieval
→ generated Context surface
```

### Alpha 2 — Real Role and Grill Interview

```text
role seed
→ grounded Role draft through governed Pi session
→ operator approval
→ one Grill question with recommendation and sources
→ answer persisted
→ close client
→ exact resume
```

### Alpha 3 — Real Spec and task

```text
Context + Role + Interview
→ Spec 120 Project Genesis handoff
→ real proposer/challenger/reconciliation cycle
→ operator approval
→ provider-neutral plan
→ real Beads task
```

### Alpha 4 — Workpoint and proof

```text
Beads task
→ scoped Workpoint
→ Work Rail
→ governed action
→ Evidence
→ closure reconciliation
→ Receipt
→ verified strike-through
```

### Alpha 5 — UIAI artifact and live refresh

```text
UIAI browser read or screenshot
→ Workspace Artifact
→ Focusa Evidence
→ canonical event
→ targeted invalidation
→ automatic Work Surface update
```

### Alpha 6 — Multiplexing and isolation

```text
Pi Work Surface
+ isolated UIAI browser Work Surface
+ exact Attachment identities
+ targeted steering
+ restart and rehydration proof
```

### Alpha 7 — Vertical projection

Render identical canonical state through General, Software, and Research Workspace View Profiles. Change layout, terminology, visual grammar, renderer, and evidence emphasis without authority mutation.

### Alpha 8 — Permanent dogfood traversal

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
→ resume exact state
```

The dogfood traversal is a permanent merge and release gate.

## 10. Alpha slice law

Every Alpha ticket crosses:

```text
schema
→ reusable Focusa primitive
→ reducer/persistence
→ typed API
→ generated TypeScript/Go clients
→ generated UI
→ real integration
→ UIAI Engine Eval when browser-facing
→ tests
→ Evidence
→ Receipt
```

The following do not satisfy a slice:

- mock-only provider or connector;
- static card;
- placeholder success;
- unpersisted UI state;
- manual DTO;
- transcript-only Interview;
- manual refresh;
- ambient project/session authority;
- provider closure without Focusa verification;
- generated Markdown without Spec 120 lifecycle;
- browser proof outside UIAI Engine Eval.

---

# Part III — Decided accelerators

## 11. Contract stack

```text
Serde + Schemars + Utoipa
→ JSON Schema 2020-12
→ OpenAPI 3.0.3
→ openapi-typescript + openapi-fetch
→ oapi-codegen v2.7.x Go client/models
→ generated A2UI catalogs and action bindings
```

A schema-drift gate MUST fail when generated artifacts differ. Manual duplicate DTOs and operation registries are forbidden.

## 12. Document and retrieval stack

```text
UIAI Engine Documents
→ pinned Docling Serve v1
→ DoclingDocument JSON
→ Docling HybridChunker
→ Focusa provenance adapter
→ SQLite FTS5 + sqlite-vec + fastembed-rs
```

Focusa MUST NOT build another generic parser, OCR/layout pipeline, table parser, chunker, or external vector database without approved benchmark evidence.

## 13. Shared UI stack

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
A2UI web_core + permanent Lit renderer
Focusa Svelte Custom Elements
PDF.js
CodeMirror Merge
Apache ECharts
```

Focusa builds domain-specific interaction, projection, design tokens, visual grammar, and authority-bound adapters only.

## 14. Code and graph stack

```text
petgraph
Tree-sitter
ast-grep
```

Use these for dependency ordering, graph algorithms, code reality, route/action discovery, singleton detection, migrations, and architecture checks. Do not build replacement parsers or structural search engines.

## 15. Connector stack

```text
oauth2-rs
keyring-rs
reqwest
serde
provider delta cursors
wiremock fixtures
```

Provider adapters:

```text
Google Drive/Gmail — generated google-apis-rs + Focusa adapter
GitHub — Octocrab + Focusa adapter
Linear — graphql_client + Focusa adapter
Microsoft Graph — typed reqwest/OData Focusa adapter
Asana — typed reqwest Focusa adapter
```

Provider types stop at the adapter boundary.

## 16. Model execution

```text
Focusa typed operation
→ Spec 133 governed session
→ Pi RPC AgentExecutionAdapter
→ structured output
→ reducer
→ Evidence / Receipt
→ generated UI
```

Forbidden runtime dependencies:

```text
Vercel WorkflowAgent
Vercel ToolLoopAgent
AI SDK UI / @ai-sdk/svelte
Vercel AI Gateway as a required service
LangChain or LlamaIndex as orchestration authority
Temporal, Airflow, Celery, or another durable session authority
```

## 17. Test and proof stack

```text
Rust: cargo-nextest, rstest, proptest, insta, wiremock
Components: Vitest, Svelte Testing Library
API: Schemathesis and generated fixtures
Generated UI: A2UI Composer/Theater and catalog/action fixtures
Browser/E2E/visual/responsive/reconnect/browser accessibility: UIAI Engine Eval only
```

Focusa MUST NOT add Playwright or another browser automation/test runtime when UIAI Engine Eval owns the required proof.

Required property tests include project isolation, replay determinism, Work Surface focus non-authority, candidate-promotion policy, view-close non-termination, browser-context storage isolation, provider closure versus Focusa verification, and Interview supersession history.

## 18. License and supply-chain stack

```text
cargo-deny
cargo-about
JavaScript/Python/model/container license inventory
Syft SBOM
GitHub dependency review and advisory scanning
```

Every dependency, model, dataset, binary, font, skill, and container image has source, version, license, notice, distribution surface, advisory source, owner, and replacement boundary.

---

# Part IV — Reuse and decomposition

## 19. Reuse assessment

Every ticket includes:

```yaml
reuse_assessment:
  existing_focusa_owner:
  existing_uiai_owner:
  existing_pi_owner:
  decided_framework:
  framework_version_ref:
  license:
  notice_required:
  conformance_fixture:
  integration_mode: adopt | wrap | configure | extend | custom
  custom_code_justification:
```

Allowed order:

```text
Adopt
→ Wrap
→ Configure
→ Extend
→ Custom only after a failing conformance fixture
```

## 20. Greater primitive submission

Every ticket declares:

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
→ reducer/state
→ typed API
→ generated clients
→ C.R.I.S.T. projection
→ renderer
→ UIAI Engine Eval
→ Evidence
→ Receipt
```

General capabilities MUST NOT remain trapped in Project Genesis, a client, or a generated component.

## 21. Machine-readable delivery graph

Before decomposition, create:

```text
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Every requirement has a stable ID, owner, dependencies, task refs, primitive refs, API operations, generated surfaces, UIAI Eval scenarios, tests, Evidence, Receipts, migration, and closure state. Agents MUST NOT infer the DAG from prose alone.

## 22. Mandatory decomposer directive

```text
Implement the complete Spec 135 series for maximum speed without reducing scope.
Do not present option menus. Use the fixed Delivery Contract and framework lock.

Create the complete machine-readable closure DAG and the Cross-Functional Alpha.
Build the Alpha first while retaining every remaining requirement in the DAG.
Open all unblocked parallel lanes after generated operation contracts stabilize.

Use vertical tracer-bullet tickets. Each ticket must cross the reusable Focusa
primitive, reducer, API, generated TypeScript/Go clients, generated UI, real
integration, tests, UIAI Engine Eval when browser-facing, Evidence, and Receipt.

Use focusa.interview.strategy.grill-with-docs.v1. Retrieve discoverable facts,
ask one decision at a time, provide a recommended answer with sources, persist
answers, and create governed glossary/ADR candidates.

Use A2UI web_core and the permanent Lit renderer with Focusa Svelte Custom
Elements. Use the native durable Focusa stream first. Implement AG-UI only as a
compatibility adapter. Use UIAI Engine Eval for all browser proof. Do not add
Playwright. Use Pi RPC/Spec 133 for model execution. Do not add Vercel AI SDK
runtime ownership.

Submit reusable behavior to greater Focusa primitives before C.R.I.S.T.-specific
projection. Do not create duplicate stores, runtimes, schemas, clients, browser
engines, or authority systems. Keep the dogfood path green after every merge.
```

## 23. Parallel lanes

After generated operations stabilize:

```text
Lane C — Context, Docling, retrieval, Google Drive
Lane R/I — Role, Grill Interview, compendium, resume
Lane S/T — Spec 120, tasks, Beads, Workpoint, Receipt
Lane M — Mission Canvas, Work Surfaces, multiplexing
Lane U — UIAI artifacts, browser contexts, Eval, accessibility
Lane V — domain packs, vertical renderers, terminology
Lane P — providers, connectors, migration, AG-UI, parity
Lane Q — security, license, SBOM, performance, recovery
```

Agents use scoped worktrees, writer leases, Workpoints, and Spec 135G Attachments.

---

## 24. Acceptance criteria

Spec 135H is accepted when:

1. Grill-with-Docs is the initial Interview strategy with preserved MIT notice.
2. Discoverable facts are retrieved before questions.
3. Every decision question has recommendation, sources, persistence, and one-question presentation.
4. Glossary and ADR results are governed candidates.
5. F0–F12 and Alpha 1–8 exist as a real integrated chain.
6. Every remaining requirement remains in the machine-readable closure DAG.
7. Generated Rust, OpenAPI 3.0.3, JSON Schema, TypeScript, Go, and A2UI contracts prevent drift.
8. Every ticket contains reuse and primitive-submission records.
9. UIAI Engine Eval owns browser proof and no Playwright dependency exists.
10. The permanent dogfood traversal remains a merge/release gate.
11. License, notice, model, container, and SBOM outputs are generated.
12. Decomposition contains no option menus or silent deferrals.
13. No acceleration decision creates duplicate canonical authority.

## 25. Closure blockers

Spec 135H cannot close while:

- Interview is static-form-only or writes files without approval;
- decomposition is horizontal-only;
- the first integrated traversal waits for every horizontal subsystem;
- an Alpha slice is mock-only;
- generated contracts drift;
- browser proof bypasses UIAI Engine Eval;
- Playwright or a duplicate browser test system is introduced;
- Vercel AI SDK or another framework duplicates Focusa/Pi authority;
- custom code bypasses a decided primitive without failing conformance evidence;
- license, notice, SBOM, Evidence, or Receipt proof is missing;
- remaining requirements disappear from the feature ledger;
- parallel agents share authority-bearing singleton or dirty-writer state.

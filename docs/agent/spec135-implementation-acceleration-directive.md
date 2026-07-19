# Spec 135 Implementation Acceleration Directive for Decomposing and Implementing Agents

**Authority:** [Spec 135H](../135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md)  
**Applies to:** every agent decomposing, scheduling, implementing, reviewing, or closing Spec 135 and companions 135A–135H.

---

## Mandatory directive

Do not present option menus for implementation frameworks, product surfaces, sequencing, connector order, Interview strategy, contract generation, document extraction, retrieval, UI foundations, graph tooling, tests, or license automation. Those decisions are already made in Specs 135, 135D, and 135H.

Implement the complete series for maximum speed without reducing scope.

Create and maintain two linked plans:

```text
Complete Closure DAG
  Every accepted requirement, dependency, test, proof, migration, client,
  connector, provider, vertical, and closure condition.

Cross-Functional Alpha
  The earliest thin but real end-to-end path through every major function.
```

The Cross-Functional Alpha is built first. It does not remove or postpone requirements from the Complete Closure DAG.

---

## Required Alpha chain

```text
Alpha 0 — generated Rust/OpenAPI/JSON Schema/TypeScript contract spine
Alpha 1 — real Markdown/code + PDF Context ingestion and retrieval
Alpha 2 — real Role approval + Grill-with-Docs Interview + resume
Alpha 3 — real Spec 120 cycle + provider-neutral plan + Beads task
Alpha 4 — Workpoint + Work Rail + Evidence + closure + Receipt
Alpha 5 — UIAI artifact + Evidence link + targeted live refresh
Alpha 6 — Pi/UIAI Work Surfaces + Attachment targeting + browser isolation
Alpha 7 — General → Software → Research projection over the same state
Alpha 8 — permanent Spec 135 dogfood integration path
```

Every Alpha ticket crosses the necessary:

```text
schema
→ reducer/persistence
→ API
→ generated client
→ UI
→ real integration
→ tests
→ evidence
```

Static UI, mock-only providers, placeholder success envelopes, transcript-only state, and hand-maintained duplicate DTOs do not satisfy an Alpha slice.

---

## Interview decision

Use:

```text
focusa.interview.strategy.grill-with-docs.v1
```

Required behavior:

1. Retrieve discoverable facts before asking the operator.
2. Ask one decision question at a time.
3. Provide one recommended answer and cited basis.
4. Walk dependent decision branches to an explicit stop condition.
5. Persist every question, answer, recommendation, and branch relationship.
6. Produce governed glossary and ADR candidates rather than writing files directly.
7. Pause and resume without losing state.
8. Use the required Discovery, Boundary, Failure, Evidence, Architecture, and Spec-Readiness Grill tranches.

---

## Decided stack

Use these technologies through Focusa-owned adapters and generated contracts:

```text
Contracts
  Serde + Schemars + Utoipa/utoipa-axum + openapi-typescript

Documents
  UIAI Documents + Docling Serve v1 + Docling HybridChunker

Retrieval
  SQLite FTS5 + sqlite-vec adapter + fastembed-rs

UI
  SvelteKit 2 + Svelte 5 + Tailwind 4
  shadcn-svelte + Bits UI + Paneforge
  TanStack Query + TanStack Table + TanStack Virtual
  Svelte Flow

Graph and code reality
  petgraph + Tree-sitter + ast-grep

Artifact renderers
  PDF.js + CodeMirror Merge + Apache ECharts + Svelte Flow

Connectors
  oauth2-rs + keyring-rs + reqwest + serde
  google-apis-rs, Octocrab, graphql_client, typed Microsoft/Asana adapters

Testing
  cargo-nextest + rstest + proptest + insta + wiremock

Compliance
  cargo-deny + cargo-about + JavaScript license inventory + Syft SBOM
```

Do not introduce a competing orchestration runtime, browser engine, document parser, vector database, Interview store, task authority, Evidence store, session model, client-canonical state machine, or desktop shell.

---

## Reuse requirement

Every ticket contains:

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

Apply this order:

```text
Adopt
→ Wrap
→ Configure
→ Extend
→ Custom only after a failing conformance fixture
```

A missing reuse assessment blocks the ticket.

---

## Parallel execution rule

After Alpha 0 stabilizes contracts, start all unblocked lanes concurrently:

```text
Rust core and migrations
Mission Canvas and generated client
Context/Docling/retrieval/Google Drive
Role/Grill Interview/Project Genesis UI
Spec Workbench/tasks/Beads/Receipts
UIAI artifacts/browser contexts/FPV
remaining connectors/providers
verticals/domain packs/renderers
security/licenses/SBOM/performance/accessibility/proof
```

Use scoped worktrees, writer leases, explicit Workpoints, and Spec 135G Attachments. Do not share a dirty writer workspace.

---

## Ticket shape

Use vertical tracer-bullet tickets with explicit blockers. A ticket must be demoable or independently verifiable and sized for one fresh agent context.

Wide migrations use expand-contract:

```text
add new form beside old
→ migrate consumers in bounded batches
→ verify compatibility
→ remove old form only after every consumer passes
```

Use this for Pi namespace convergence, schema envelopes, renamed types, event versions, and compatibility projections.

---

## Permanent integration gate

The following path must remain green:

```text
Context
→ Role
→ Grill Interview
→ Project Genesis Spec
→ Tasks
→ Workpoint
→ Evidence
→ Receipt
→ UIAI artifact
→ multiplexed Mission Canvas
```

A change that breaks this path does not merge.

---

## No-deferral rule

Do not use:

```text
later
future enhancement
post-MVP
nice to have
out of scope for now
```

A requirement remains implemented, open, or blocked in the Complete Feature Ledger. It leaves the graph only through a versioned operator-approved amendment.

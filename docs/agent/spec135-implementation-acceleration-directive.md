# Spec 135 Implementation Acceleration Directive for Decomposing and Implementing Agents

**Authority:** [Spec 135H](../135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md), [Spec 135I](../135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), and [Spec 135J](../135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md)  
**Applies to:** every agent decomposing, scheduling, implementing, reviewing, testing, or closing Spec 135 and companions 135A–135J.

---

## Mandatory directive

Do not present option menus for implementation frameworks, generated-UI protocols, product surfaces, sequencing, connector order, Interview strategy, contract generation, document extraction, retrieval, UI foundations, graph tooling, event streaming, tests, or license automation. Those decisions are already made in Specs 135, 135D, 135H, 135I, and 135J.

Implement the complete series for maximum speed without reducing scope.

Create and maintain two linked plans:

```text
Complete Closure DAG
  Every accepted requirement, dependency, test, proof, migration, client,
  connector, provider, vertical, generated surface, and closure condition.

Cross-Functional Alpha
  The earliest thin but real end-to-end path through every major function.
```

The Cross-Functional Alpha is built first. It does not remove or postpone requirements from the Complete Closure DAG.

---

## Required Alpha chain

```text
Alpha 0 — Operation Registry, generated contracts, action bindings,
          capability snapshot, shared envelope, durable stream, AG-UI,
          and one A2UI surface
Alpha 1 — real Markdown/code + PDF Context ingestion and generated UI
Alpha 2 — real Role approval + Grill-with-Docs Interview + resume UI
Alpha 3 — real Spec 120 cycle + provider-neutral plan + Beads task UI
Alpha 4 — Workpoint + Work Rail + Evidence + closure + Receipt UI
Alpha 5 — UIAI artifact + Evidence link + targeted live generated refresh
Alpha 6 — Pi/UIAI Work Surfaces + Attachment targeting + browser isolation
Alpha 7 — General → Software → Research projection over the same state
Alpha 8 — permanent nontechnical Spec 135 dogfood integration path
```

Every Alpha ticket crosses the necessary:

```text
schema
→ reducer/persistence
→ core read model
→ typed API and Operation Registry
→ generated client/action binding
→ real-time generated UI
→ real integration
→ tests
→ evidence
```

Static UI, mock-only providers, placeholder success envelopes, transcript-only state, CLI-only completion, hand-maintained duplicate DTOs, manual route/action catalogs, and non-replayable event streams do not satisfy an Alpha slice.

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
8. Use Discovery, Boundary, Failure, Evidence, Architecture, and Spec-Readiness Grill tranches.
9. Render every question and response through the real-time generated Interview surface.

---

## Decided stack

Use these technologies through Focusa-owned adapters and generated contracts:

```text
Contracts and typed operations
  Serde + Schemars + Utoipa/utoipa-axum
  OpenAPI 3.1 + JSON Schema
  openapi-typescript + openapi-fetch
  Focusa Operation Registry

Generated UI and streaming
  A2UI v0.9.1
  @a2ui/web_core/v0_9
  @a2ui/lit/v0_9 for immediate Alpha rendering
  Focusa Svelte mappings on web_core for full production rendering
  AG-UI adapter over Focusa APIs and canonical events

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
  Vitest + Svelte Testing Library + Playwright + Schemathesis
  A2UI and AG-UI deterministic fixtures

Compliance
  cargo-deny + cargo-about + JavaScript/Python/model license inventory
  Syft CycloneDX/SPDX-compatible SBOM
```

Do not introduce a competing orchestration runtime, browser engine, document parser, vector database, Interview store, task authority, Evidence store, session model, client-canonical state machine, generated-UI DSL, event database, route registry, permission system, error taxonomy, or desktop shell.

---

## Core API reuse decision

Generated UI uses one Focusa core/API path:

```text
canonical Focusa state and services
→ bounded read models
→ UiInteractionIntent
→ A2UI generated surface

Rust/OpenAPI Operation Registry
+ current scope/capabilities/permissions
→ UI Action Binding
→ preview/commit Focusa operation
→ shared ToolResult/Error envelope
→ canonical event and Receipt
```

Live updates use:

```text
SQLite canonical event replay
→ existing broadcast live tail
→ AG-UI translation
→ A2UI surface delta
```

A lagged client replays from SQLite using a stable cursor/Last-Event-ID. Do not silently drop events or persist another AG-UI/UI event history.

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

generated_ui:
  surface_kind:
  operation_ids: []
  a2ui_catalog_components: []
  read_model_refs: []
  action_binding_refs: []
  capability_refs: []
  ag_ui_events: []
  durable_event_cursor:
  primary_action:
  autosave_behavior:
  resume_behavior:
  recovery_states: []
  terminal_fallback:
  accessibility_tests: []
  schemathesis_workflow_ref:
  playwright_flow_ref:
```

Apply this order:

```text
Adopt
→ Wrap
→ Configure
→ Extend
→ Custom only after a failing conformance fixture
```

A missing reuse assessment or generated-UI section blocks the ticket.

---

## Parallel execution rule

After Alpha 0 stabilizes the contracts, Operation Registry, action bindings, and durable stream, start all unblocked lanes concurrently:

```text
Rust core, read models, and migrations
Mission Canvas, A2UI catalogs, renderers, and generated client
Context/Docling/retrieval/Google Drive
Role/Grill Interview/Project Genesis generated UI
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

Use this for Pi namespace convergence, schema envelopes, Operation Registry adoption, shared ToolResult constructors, event IDs/cursors, renamed types, protocol versions, and compatibility projections.

---

## Permanent integration gate

The following nontechnical generated-UI path must remain green:

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

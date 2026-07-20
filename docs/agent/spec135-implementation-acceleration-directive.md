# Spec 135 Implementation Acceleration Directive for Decomposing and Implementing Agents

**Authority:** [Spec 135 Series Current Authoritative Delivery Contract](../135-series-current-manifest.md), [Spec 135H](../135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md), [Spec 135I](../135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), [Spec 135J](../135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md), and [Spec 135K](../135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md)  
**Applies to:** every agent decomposing, scheduling, implementing, reviewing, testing, or closing Spec 135 and companions 135A–135K.

## 1. Mandatory delivery posture

Do not present implementation option menus. The framework, ownership, sequencing, browser-proof, generated-UI, contract, document, retrieval, connector, and Interview decisions are fixed.

Implement the complete series for maximum speed without reducing scope.

Create and maintain:

```text
Complete machine-readable closure DAG
  Every requirement, dependency, migration, client, connector, provider,
  vertical, generated surface, proof, Evidence, Receipt, and closure state.

Cross-Functional Alpha
  The earliest production-shaped traversal through every major function.
```

The Alpha is implemented first. It does not remove any requirement from the closure DAG.

## 2. Required Foundation Train and Alpha chain

```text
F0 — freeze series at 135K and compile the Delivery Contract
F1 — feature ledger, delivery DAG, parity, framework, and proof matrices
F2 — JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generated TypeScript and Go clients
F4 — Operation Registry, capability projection, UI action bindings
F5 — shared ToolResult/error envelopes
F6 — durable SQLite replay plus broadcast live tail
F7 — capability/permission projection and version handshake
F8 — Pi RPC AgentExecutionAdapter
F9 — A2UI web_core plus permanent Lit renderer
F10 — Focusa Svelte Custom Elements
F11 — first UIAI Engine Eval scenario
F12 — one real Context operation through generated UI

Alpha 1 — real Markdown/code + PDF Context ingestion and generated UI
Alpha 2 — real Role approval + Grill Interview + close/resume
Alpha 3 — real Spec 120 cycle + provider-neutral plan + Beads task
Alpha 4 — Workpoint + Work Rail + Evidence + closure + Receipt
Alpha 5 — UIAI artifact + Evidence + automatic live refresh
Alpha 6 — Pi/UIAI Work Surfaces + Attachment targeting + browser isolation
Alpha 7 — General → Software → Research projection over identical state
Alpha 8 — permanent nontechnical Spec 135 dogfood path
```

Every slice crosses:

```text
requirement ID
→ greater Focusa primitive
→ schema
→ reducer/persistence
→ typed API and Operation Registry
→ generated TypeScript/Go clients
→ generated UI
→ real integration
→ UIAI Engine Eval when browser-facing
→ tests
→ Evidence
→ Receipt
```

Static UI, mock-only providers, placeholder results, transcript-only state, CLI-only completion, duplicate DTOs, manual route catalogs, and non-replayable streams do not satisfy a slice.

## 3. Interview decision

Use:

```text
focusa.interview.strategy.grill-with-docs.v1
```

Required behavior:

1. Retrieve discoverable facts before asking the operator.
2. Ask one decision question at a time.
3. Provide one recommended answer and source basis.
4. Traverse dependent branches to explicit stop conditions.
5. Persist questions, answers, recommendations, and branch relationships.
6. Produce governed glossary and ADR candidates.
7. Pause, close, reopen, and resume without state loss.
8. Use Discovery, Boundary, Failure, Evidence, Architecture, and Spec-Readiness tranches.
9. Render all interaction through the generated Interview surface.

## 4. Fixed stack

```text
Contracts
  Serde + Schemars + Utoipa
  JSON Schema 2020-12
  OpenAPI 3.0.3
  openapi-typescript + openapi-fetch
  oapi-codegen v2.7.x for UIAI Engine
  Focusa Operation Registry

Generated UI
  A2UI v0.9.1
  @a2ui/web_core/v0_9
  @a2ui/lit/v0_9 permanent renderer
  Focusa Svelte Custom Elements
  AG-UI compatibility adapter after native stream stabilization

Model execution
  Spec 133 governed sessions
  Pi RPC AgentExecutionAdapter

Documents
  UIAI Documents + Docling Serve v1 + HybridChunker

Retrieval
  SQLite FTS5 + sqlite-vec adapter + fastembed-rs

UI
  SvelteKit 2 + Svelte 5 + Tailwind 4
  shadcn-svelte + Bits UI + Paneforge
  TanStack Query + Table + Virtual
  Svelte Flow

Code and graph reality
  petgraph + Tree-sitter + ast-grep

Artifact rendering
  PDF.js + CodeMirror Merge + ECharts + Svelte Flow

Connectors
  oauth2-rs + keyring-rs + reqwest + serde
  generated or typed provider adapters

Testing
  cargo-nextest + rstest + proptest + insta + wiremock
  Vitest + Svelte Testing Library + Schemathesis
  A2UI deterministic fixtures
  UIAI Engine Eval for all browser proof

Compliance
  cargo-deny + cargo-about + package/model/container license inventory
  Syft CycloneDX/SPDX-compatible SBOM
```

Forbidden:

- Do not add Playwright to Focusa; UIAI Engine Eval owns all browser proof.
- Playwright in Focusa;
- a complete custom Svelte A2UI renderer;
- AG-UI as canonical state or an Alpha blocker;
- Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, `@ai-sdk/svelte`, or Vercel AI Gateway as runtime authority;
- competing orchestration, browser, document, vector, Interview, task, Evidence, session, permission, route, error, or desktop systems.

## 5. Core runtime path

```text
canonical Focusa primitives and state
→ bounded read model
→ UiInteractionIntent
→ A2UI surface

Operation Registry
+ exact scope/capabilities/permissions
→ UI Action Binding
→ preview/commit
→ ToolResult/error envelope
→ canonical event
→ Evidence / Receipt
→ generated UI delta
```

Native live path:

```text
SQLite event replay
→ broadcast live tail
→ A2UI snapshot/delta
```

AG-UI translates the native stream for external compatibility. It does not own another history.

## 6. Reuse and primitive-submission requirement

Every ticket contains:

```yaml
requirement_refs: []

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

primitive_submission:
  canonical_owner:
  reusable_primitive:
  crist_specific_projection:
  core_change:
  api_change:
  generated_contract_change:
  uiai_change:
  client_change:
  migration:
  proof:

generated_ui:
  surface_kind:
  operation_ids: []
  catalog_components: []
  action_bindings: []
  durable_event_cursor:
  primary_action:
  autosave_behavior:
  resume_behavior:
  recovery_states: []
  terminal_fallback:
  accessibility_tests: []
  schemathesis_workflow_ref:
  uiai_eval_scenarios: []
  evidence_requirements: []
  receipt_requirements: []
```

Apply:

```text
Adopt
→ Wrap
→ Configure
→ Extend
→ Custom only after a failing conformance fixture
```

General behavior is implemented as a reusable Focusa primitive before C.R.I.S.T.-specific projection.

## 7. Parallel execution

After F4 stabilizes generated operation contracts, start all unblocked lanes concurrently:

```text
C — Context, Docling, retrieval, Google Drive
R/I — Role, Grill Interview, compendium, resume
S/T — Spec 120, tasks, Beads, Workpoint, Receipt
M — Mission Canvas, Work Surfaces, multiplexing
U — UIAI artifacts, browser contexts, Eval, accessibility
V — domain packs, verticals, renderers, terminology
P — providers, connectors, migration, parity, AG-UI
Q — security, licenses, SBOM, performance, recovery
```

Use scoped worktrees, writer leases, explicit Workpoints, and Spec 135G Attachments. Do not share a dirty writer workspace.

## 8. Wide migrations

Use expand-contract:

```text
add new form beside old
→ migrate bounded consumers
→ verify generated compatibility and proof
→ remove old only after every reader and writer passes
```

Use for Pi namespace convergence, schema envelopes, Operation Registry, ToolResult constructors, event IDs/cursors, renamed types, protocol versions, and compatibility projections.

## 9. Permanent integration gate

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

A change that breaks this path does not merge.

## 10. No-deferral rule

Do not use “later,” “future enhancement,” “post-MVP,” “nice to have,” or “out of scope for now” to remove requirements.

A requirement remains implemented, open, or explicitly blocked in the machine-readable ledger. It leaves the graph only through a versioned operator-approved amendment.

# Spec 135 Implementation Acceleration Directive for Decomposing and Implementing Agents

**Authority:** [Spec 135 Series Current Authoritative Delivery Contract](../135-series-current-manifest.md), [Mission Canvas Host and Renderer Contract](../contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml), [Spec 135H](../135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md), [Spec 135I](../135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), [Spec 135J](../135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md), and [Spec 135K](../135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md)  
**Applies to:** every agent decomposing, scheduling, implementing, reviewing, testing, documenting, proving, or closing Spec 135 and companions 135A–135K.

## 0. Mandatory authority preflight

Before touching Mission Canvas, Pi UI, C.R.I.S.T. generated UI, Work Surfaces, workspace verticals, renderer code, proof files, or closure state:

1. Read `docs/135-series-current-manifest.md`.
2. Read `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`.
3. Record both axes in the work item:
   - `interaction_mode`
   - `host_renderer`
4. State whether the slice is:
   - canonical runtime;
   - rich Focusa Pi host;
   - terminal projection/fallback;
   - UIAI Engine Cockpit projection;
   - generated C.R.I.S.T. Work Surface;
   - native TUI, Mission Deck, or menubar projection.
5. Identify the proof class required before writing code.

Do not infer GUI architecture from a screenshot, issue title, source comment, existing TUI component, or handwritten proof JSON.

### 0.1 Operator intent that must not drift

```text
Pi terminal interaction
        ⇅ light switch controlled directly from Pi
Focusa-owned rich Mission Canvas professional GUI
```

Canvas ON opens or focuses the Focusa rich host over the same live Pi session. Canvas OFF closes or unmounts that projection and returns focus to stock Pi. The runtime, Attachment, model stream, tools, transcript, Workpoint, Evidence, queues, drafts, and history continue unchanged.

The primary rich Focusa-enhanced Pi renderer is `focusa_pi_rich_window`: a Focusa-owned local webview/window using the decided Svelte shell and A2UI/Lit generated-surface stack.

A box-drawing `ctx.ui.custom(...)` shell, sidebar, status dashboard, Markdown split, or transcript stage is a **terminal projection or scaffold**, not the rich GUI.

### 0.2 Canonical shell invariant

Rich Mission Canvas hosts preserve this order:

```text
Work Surface strip
→ focused Work Surface + Focusa right inspector
→ Work Rail
→ Steering Queue
→ Follow-up Queue
→ Prompt Editor
```

A global scope bar, left activity rail, detached inspector, or secondary pane may be added by a host/profile, but is not a required replacement for this anatomy.

The active Pi transcript is one `pi_session` Work Surface.

### 0.3 Vertical workspace law

A profile switch must recompose geometry, panel set/order, terminology, artifact renderers, Evidence emphasis, history, icons, density, and controls over identical canonical state. A color swap, hard-coded Markdown template, or separate client-local vertical application does not satisfy a vertical.

### 0.4 Immediate closure firewall

Until runtime proof conforms to the authority contract, do not claim:

- full rich Pi Mission Canvas GUI;
- complete alternate Pi GUI;
- real rich split panes;
- generated C.R.I.S.T. GUI from transcript messages;
- full vertical workspace rendering from Markdown projections;
- final Spec 135 acceptance.

Useful terminal work may remain and be labeled accurately as partial.

## 1. Mandatory delivery posture

Do not present implementation option menus for decisions already fixed by the delivery contract. Implement the complete series without reducing scope.

Create and maintain:

```text
Complete machine-readable closure DAG
  Every requirement, dependency, migration, client, connector, provider,
  vertical, host renderer, generated surface, proof, Evidence, Receipt,
  and closure state.

Cross-Functional Alpha
  The earliest production-shaped traversal through every major function.
```

The Alpha is implemented first. It does not remove requirements from the closure DAG.

## 2. Required Foundation Train and Alpha chain

```text
F0 — compile the Delivery Contract and host/renderer contract
F1 — feature ledger, delivery DAG, parity, framework, and proof matrices
F2 — JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generated TypeScript clients and portable contract validation
F4 — Operation Registry, capability projection, UI action bindings
F5 — shared ToolResult/error envelopes
F6 — durable SQLite replay plus broadcast live tail
F7 — capability/permission projection and version handshake
F8 — Pi RPC AgentExecutionAdapter
F9 — A2UI web_core plus permanent Lit generated-surface renderer
F10 — Focusa Svelte Custom Elements
F11 — first UIAI Engine Eval scenario
F12 — one real Context operation through generated UI
F13 — Focusa Pi rich Mission Canvas window host and typed Pi lifecycle bridge
F14 — same-session Canvas ON/OFF runtime traversal

Alpha 1 — real Markdown/code + PDF Context ingestion and generated UI
Alpha 2 — real Role approval + Grill Interview + close/resume
Alpha 3 — real Spec 120 cycle + provider-neutral plan + Beads task
Alpha 4 — Workpoint + Work Rail + Evidence + closure + Receipt
Alpha 5 — UIAI artifact + Evidence + automatic live refresh
Alpha 6 — Pi/UIAI Work Surfaces + Attachment targeting + browser isolation
Alpha 7 — General → Software → Legal/Markets as applicable → Research projection
Alpha 8 — permanent nontechnical Spec 135 dogfood path
Alpha 9 — Pi terminal ⇄ rich Mission Canvas over the same live session
```

Every slice crosses:

```text
requirement ID
→ greater Focusa primitive
→ schema
→ reducer/persistence
→ typed API and Operation Registry
→ generated clients/contracts
→ host renderer or generated Work Surface
→ real integration
→ UIAI Engine Eval when browser-facing
→ tests
→ Evidence
→ Receipt
```

Static UI, mock-only providers, placeholder results, transcript-only state, CLI-only completion, duplicate DTOs, manual route catalogs, source-string proof, and non-replayable streams do not satisfy a slice.

## 3. Work-item declaration required for every UI slice

Every ticket or Workpoint includes:

```yaml
requirement_refs: []

presentation_contract:
  interaction_mode: canvas-guided | terminal-guided | headless
  host_renderer: focusa_pi_rich_window | uiai_engine_cockpit | mission_deck_web | pi_terminal_projection | native_tui | menubar_peek | headless_none
  surface_kind:
  rich_gui_required: false
  terminal_fallback_required: false
  canonical_shell_regions: []
  continuity_invariants: []

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

General behavior is implemented as a reusable Focusa primitive before C.R.I.S.T.-specific or client-specific projection.

## 4. Fixed stack

```text
Contracts
  Serde + Schemars + Utoipa
  JSON Schema 2020-12
  OpenAPI 3.0.3
  openapi-typescript + openapi-fetch
  UIAI Engine-owned adapter generated outside Focusa core
  Focusa Operation Registry

Rich Focusa Pi host
  Focusa-owned local webview/window
  SvelteKit 2 + Svelte 5 + Tailwind 4
  shadcn-svelte + Bits UI + Paneforge
  TanStack Query + Table + Virtual
  Svelte Flow
  PDF.js + CodeMirror Merge + ECharts

Generated UI
  A2UI v0.9.1
  @a2ui/web_core/v0_9
  @a2ui/lit/v0_9 permanent generated-surface renderer
  Focusa Svelte Custom Elements
  AG-UI compatibility adapter after native stream stabilization

Model execution
  Spec 133 governed sessions
  Pi RPC AgentExecutionAdapter

Documents
  UIAI Documents + Docling Serve v1 + HybridChunker

Retrieval
  SQLite FTS5 + sqlite-vec adapter + fastembed-rs

Code and graph reality
  petgraph + Tree-sitter + ast-grep

Connectors
  oauth2-rs + keyring-rs + reqwest + serde
  generated or typed provider adapters

Testing
  cargo-nextest + rstest + proptest + insta + wiremock
  Vitest + Svelte Testing Library + Schemathesis
  A2UI deterministic fixtures
  UIAI Engine Eval for browser, visual, responsive, reconnect, and browser-accessibility proof

Compliance
  cargo-deny + cargo-about + package/model/container license inventory
  Syft CycloneDX/SPDX-compatible SBOM
```

Forbidden:

- Playwright in Focusa;
- a complete custom Svelte A2UI renderer;
- a second A2UI message processor or `SurfaceModel`;
- AG-UI as canonical state or an Alpha blocker;
- Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, `@ai-sdk/svelte`, or Vercel AI Gateway as runtime authority;
- competing orchestration, browser, document, vector, Interview, task, Evidence, session, permission, route, error, desktop, or rich-host runtimes;
- calling a terminal TUI a rich graphical GUI;
- calling a Focusa/Pi surface a Cockpit.

## 5. Core runtime path

```text
canonical Focusa primitives and state
→ bounded read model
→ host-specific workspace projection or UiInteractionIntent
→ trusted renderer

Operation Registry
+ exact scope/capabilities/permissions
→ UI Action Binding
→ preview/commit
→ ToolResult/error envelope
→ canonical event
→ Evidence / Receipt
→ generated UI or workspace delta
```

Native live path:

```text
SQLite event replay
→ broadcast live tail
→ scoped invalidation
→ workspace/A2UI snapshot or delta
```

AG-UI translates the native stream for external compatibility. It does not own another history.

## 6. C.R.I.S.T. interaction decision

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
9. Render interaction through a real generated C.R.I.S.T. Work Surface in rich hosts.
10. Provide a truthful terminal fallback without claiming it is the generated GUI.

## 7. Rich Mission Canvas implementation sequence

```text
1. Canonical interaction-mode + host-renderer contract
2. Durable Work Surface and Canvas presentation state
3. Focusa Pi rich-window lifecycle operation and Pi bridge
4. Svelte Mission Canvas shell with canonical region invariant
5. Current Pi session projected as a live pi_session Work Surface
6. Work Surface inventory, focus, groups, real splits, suspend, close, rehydrate
7. Focusa right inspector, Work Rail, steering, follow-up, and composer routing
8. Dynamic registries and vertical profile resolution
9. Generated C.R.I.S.T. Work Surfaces through A2UI/Lit
10. UIAI browser Work Surfaces through typed artifact/session references
11. Canvas OFF/ON continuity, reconnect, restart, and draft preservation
12. UIAI Engine Eval and Evidence/Receipt closure
```

Do not begin by expanding the terminal shell and later relabel it as the GUI.

## 8. Proof classes

### Unit and contract proof

Use for reducers, schema, routing, exact scope, operation bindings, layout persistence, and host capability resolution.

### Component proof

Use for Svelte shell regions, profile resolution, keyboard/mouse accessibility, split mechanics, draft preservation, and trusted generated components.

### Runtime integration proof

Must exercise Pi command → rich host lifecycle → same Session/Attachment → live event continuity → Canvas OFF → same Pi session.

### UIAI Engine Eval

Required for rich GUI browser rendering, visual comparison, responsive breakpoints, accessibility, reconnect, browser Work Surfaces, screenshots, diagnostics, and browser proof artifacts.

Invalid as rich GUI proof:

```text
source substring checks
handwritten pass JSON
static screenshot without runtime trace
Markdown split representation
process-local layout map
terminal box drawing
transcript-only stage
```

## 9. Parallel execution

After F4 stabilizes generated operation contracts, start unblocked lanes concurrently:

```text
C — Context, Docling, retrieval, Google Drive
R/I — Role, Grill Interview, compendium, resume
S/T — Spec 120, tasks, Beads, Workpoint, Receipt
M — Mission Canvas, Work Surfaces, multiplexing, rich Pi host
U — UIAI artifacts, browser contexts, Eval, accessibility
V — domain packs, verticals, renderers, terminology
P — providers, connectors, migration, parity, AG-UI
Q — security, licenses, SBOM, performance, recovery
```

Use scoped worktrees, writer leases, explicit Workpoints, and Spec 135G Attachments. Do not share a dirty writer workspace.

## 10. Closure discipline

A requirement may be `implemented`, `partially_implemented`, `terminal_fallback_only`, `rich_host_missing`, `proof_missing`, `blocked`, or `verified`.

Do not collapse those states into `passed`.

The current PR’s useful terminal components, interaction-mode foundations, Work Surface schemas, and isolation work may remain. Reclassify them accurately. Rich Mission Canvas and final Spec 135 closure remain open until the required host and runtime proof exist.

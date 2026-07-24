# Spec 135I — Real-Time Generated C.R.I.S.T. UI, Nontechnical Onboarding, and Core API Integration

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md), [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md), [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md), [Spec 135E](135e-cross-spec-amendments-migration-and-closure-matrix.md), and [Spec 135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135I.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs any conflict.

---

## 0. One-line definition

Every onboarding and C.R.I.S.T. interaction MUST be presented as a live, incrementally regenerated, plain-language, capability-aware interface that a nontechnical operator can complete, recover, close, reopen, and resume without raw APIs, schemas, identifiers, command syntax, or agent internals, while every read and mutation remains bound to canonical Focusa scope, APIs, authority, Evidence, and Receipts.

---

## 1. Product law

```text
C.R.I.S.T. is not a collection of static forms.
C.R.I.S.T. is not a transcript rendered inside a panel.
C.R.I.S.T. is not a CLI workflow with a decorative status page.
C.R.I.S.T. is a server-driven, real-time generated interaction experience.
```

Every stage, transition, approval, recovery state, and continuation action MUST derive from current canonical project state, domain packs, capabilities, permissions, connector/provider health, operator progress, UXP/UFI, and explicit scope.

A backend-complete feature is incomplete when a nontechnical operator cannot understand, complete, recover, and resume it through generated UI.

---

## 2. Fixed protocol and framework stack

Agents MUST use these decisions and MUST NOT present alternatives.

### 2.1 Generated UI protocol and renderer

```text
A2UI protocol v0.9.1
@a2ui/web_core/v0_9
@a2ui/lit/v0_9
Focusa Svelte Custom Elements
Focusa versioned trusted A2UI catalog
```

Ownership:

- `@a2ui/web_core/v0_9` owns message processing, surface state, validation, data binding, incremental updates, catalog behavior, and action routing.
- `@a2ui/lit/v0_9` is the permanent web renderer for A2UI v0.9.1.
- Focusa-specific domain controls are authored in Svelte, compiled as Custom Elements, and registered in the trusted Focusa catalog.

Forbidden:

- a complete custom Svelte A2UI renderer;
- a second A2UI message processor;
- a second `SurfaceModel` or data-binding implementation;
- arbitrary generated JavaScript, HTML, frontend functions, components, or iframes.

### 2.2 Native real-time stream

The primary product stream is:

```text
Focusa SQLite canonical event history
+ stable event ID and sequence
+ cursor / Last-Event-ID replay
+ existing broadcast live tail
+ scoped invalidation
+ A2UI snapshots and incremental messages
```

AG-UI is a required compatibility adapter for external agent clients. It MUST NOT own canonical state, persistence, event history, action authority, or approval. AG-UI implementation proceeds in parallel after the native Focusa/A2UI path is stable and MUST NOT block the first complete C.R.I.S.T. traversal.

### 2.3 Typed contracts and clients

```text
Rust + Serde + Schemars + Utoipa
→ JSON Schema 2020-12
→ OpenAPI 3.0.3
→ openapi-typescript + openapi-fetch
→ UIAI Engine-owned adapter generated outside Focusa core from the published OpenAPI contract
→ generated A2UI catalog schemas and UI action bindings
```

OpenAPI 3.0.3 is the sole Spec 135 transport contract. Manual duplicate DTOs, route wrappers, action registries, and UIAI Focusa client models are forbidden when generation can represent the contract.

### 2.4 Model execution

Role Composer, Grill Interview, grounded recommendations, synthesis, and generated explanations use:

```text
Focusa typed operation
→ Spec 133 daemon-owned governed execution session
→ Pi RPC AgentExecutionAdapter reference implementation
→ structured result
→ reducer
→ Evidence / Receipt
→ generated UI delta
```

The browser client MUST NOT call model providers directly.

Forbidden as Focusa runtime dependencies:

```text
Vercel WorkflowAgent
Vercel ToolLoopAgent
AI SDK UI / @ai-sdk/svelte
Vercel AI Gateway as a required service
any Vercel-owned durable workflow or tool authority
```

### 2.5 Testing and browser proof

Required stack:

```text
Rust: cargo-nextest, rstest, proptest, insta, wiremock
Components: Vitest, Svelte Testing Library
API: Schemathesis and generated contract fixtures
Generated UI: A2UI Composer/Theater fixtures and catalog/action fixtures
Browser/E2E/visual/responsive/reconnect/browser accessibility: UIAI Engine Eval only
```

Focusa MUST NOT use Playwright Test, Playwright Library, Playwright CLI, Playwright MCP, `@playwright/test`, `playwright.config.*`, or Playwright browser fixtures.

UIAI Engine Eval MUST produce stable browser-session, context, screenshot, diagnostic, accessibility, visual-comparison, Evidence, and Receipt references.

---

## 3. Authority boundary

```text
Focusa core and reducers
  Canonical Project Genesis, Context, Role, Interview, Spec, Tasks,
  Workpoint, scope, capabilities, permissions, Evidence, Receipts, and recovery.

Focusa API
  Typed reads, preview/commit, idempotency, versions, operation metadata,
  generated surfaces, action bindings, capabilities, and durable event replay.

Generated UI projection
  A2UI surfaces, trusted components, plain-language copy, recommendations,
  validation presentation, progress, recovery, and bound actions.

Clients
  Render trusted catalogs, retain ephemeral drafts, collect input,
  and invoke generated typed Focusa actions.

UIAI Engine
  Browser execution, browser Eval, screenshots, diagnostics, visual proof,
  responsive proof, accessibility browser proof, and browser artifacts.
```

Generated UI is a projection. It is not a second canonical store, workflow engine, permission system, Interview database, task authority, browser engine, or model-execution runtime.

---

## 4. Generated Surface Envelope

```yaml
schema: focusa.generated_surface.v1
surface_id:
surface_revision:
surface_kind: onboarding | crist_context | crist_role | crist_interview | crist_spec | crist_tasks | project_profile | recovery

protocol:
  a2ui_version: v0.9
  catalog_id:
  catalog_version:

scope:
  project_root:
  project_identity_ref:
  continuity_id:
  workpoint_id:
  attachment_id:
  work_surface_id:

stage:
  crist_stage: context | role | interview | spec | tasks
  lifecycle_state:
  completion_posture:
  next_safe_action_ref:

inputs:
  resolved_project_operating_profile_ref:
  capability_snapshot_ref:
  domain_pack_refs: []
  connector_health_ref:
  uxp_profile_ref:
  client_capability_profile_ref:

messages:
  a2ui_message_refs: []
  initial_messages: []

actions:
  action_binding_refs: []

presentation:
  language_level: plain
  progressive_disclosure: true
  advanced_details_available: true
  primary_action_id:
  help_topic_refs: []

freshness:
  generated_at:
  source_state_revision:
  event_cursor:
  stale_after:

provenance:
  generated_by:
  generation_reason:
  evidence_refs: []
```

The envelope wraps A2UI messages and Focusa action bindings. It MUST NOT become another component-description language or state store.

---

## 5. Focusa UI Action Binding

```yaml
schema: focusa.ui_action_binding.v1
action_binding_id:
ui_action_name:
label:
description:

focusa_operation:
  operation_id:
  route:
  method:
  mode: read | preview | commit
  input_schema_ref:
  output_schema_ref:

scope:
  project_root:
  continuity_id:
  workpoint_id:
  attachment_id:

control:
  capability_ref:
  permission_ref:
  confirmation: none | simple | consequential
  idempotency_required:
  optimistic_concurrency_required:
  receipt_required:
  reversible:

presentation:
  success_message:
  failure_message:
  recovery_action_ref:
```

A2UI actions reference the binding ID. Clients MUST NOT derive canonical routes, methods, or authority from component text.

Every consequential action follows:

```text
validate input
→ validate scope
→ validate capability and permission
→ preview when required
→ operator confirmation
→ typed commit
→ canonical event
→ Receipt when required
→ A2UI delta
```

Forbidden:

```text
POST /v1/ui/execute-arbitrary
POST /v1/ui/run-generated-code
POST /v1/ui/mutate-anything
```

---

## 6. Core API and primitive integration

Required API families:

```text
GET  /v1/ui/catalogs
GET  /v1/ui/catalogs/:catalog_id
GET  /v1/ui/capabilities
GET  /v1/ui/operations

POST /v1/ui/surfaces
GET  /v1/ui/surfaces/:surface_id
GET  /v1/ui/surfaces/:surface_id/stream
POST /v1/ui/surfaces/:surface_id/regenerate

GET  /v1/project-genesis/:genesis_id/ui
GET  /v1/project-genesis/:genesis_id/ui/:stage

POST /v1/ui/actions/preview
POST /v1/ui/actions/commit
```

Required placement:

```text
focusa-core/src/ui_intent/
  Pure bounded interaction intent and action-binding projections.

focusa-api/src/ui_projection/
  A2UI serialization, catalogs, surface envelopes, and routes.

focusa-api/src/ag_ui/
  External compatibility translation over the native durable stream.

packages/focusa-generated-ui/
  A2UI web_core, permanent Lit renderer integration, Svelte Custom Elements,
  catalog contracts, typed action client, query keys, and fixtures.
```

Generated UI routes MUST compose existing Context, Role, Interview, Spec, Task, Workpoint, Evidence, Receipt, connector, provider, session, capability, permission, and ToolResult primitives. Canonical business logic MUST NOT be copied into UI routes or components.

General behavior MUST be submitted to the **Greater Focusa primitive** before C.R.I.S.T.-specific projection code is added.

---

## 7. Surface regeneration and resume

Native flow:

```text
canonical event
→ invalidation map
→ recompute bounded UiInteractionIntent
→ produce A2UI delta or snapshot
→ durable cursor advances
→ subscribed client updates
```

Required behavior:

- deterministic shell renders before generated explanatory copy completes;
- connector, extraction, Interview, Spec, task, and provider progress stream;
- user draft input survives unrelated surface deltas;
- reconnect uses event cursor or `Last-Event-ID`;
- missed events replay from SQLite before live-tail subscription;
- divergence requests a fresh bounded snapshot;
- manual refresh is a recovery action, not normal operation;
- reopening loads canonical progress rather than inferring from transcript history.

Resume flow:

```text
load canonical Project Genesis state
→ replay missed events
→ load current generated surface
→ restore unsent local draft separately
→ show saved and unsaved state truthfully
```

---

## 8. Deterministic and generative boundary

Deterministic from Focusa state and schemas:

- current stage and readiness;
- required fields and valid input types;
- action bindings;
- scope, capabilities, permissions, and approvals;
- source, Evidence, and Receipt references;
- validation, completion, and recovery state;
- one primary next action;
- non-hideable safety and authority state.

AI-generated or adapted:

- concise plain-language explanations;
- Grill question wording;
- recommendations and source-basis summaries;
- contextual help;
- source summaries;
- suggested defaults;
- workspace/domain terminology.

Generated content MUST NOT invent actions, routes, permissions, required fields, completion state, evidence, or authority.

---

## 9. Trusted Focusa component catalog

Use maintained A2UI primitives for ordinary layout and inputs. Initial Focusa catalog:

```text
FocusaStageShell
FocusaProgressStepper
FocusaPrimaryAction
FocusaNextStepCard
FocusaSourceConnectorCard
FocusaDropzone
FocusaImportScopePreview
FocusaContextSummary
FocusaContextClaimReview
FocusaContradictionCard
FocusaRoleSeed
FocusaRoleDraft
FocusaRedline
FocusaGroundingSources
FocusaQuestionCard
FocusaRecommendationCard
FocusaAnswerInput
FocusaInterviewBranchProgress
FocusaReadinessMeter
FocusaSpecSectionStatus
FocusaObjectionCard
FocusaApprovalCard
FocusaTaskPlan
FocusaDependencyGraph
FocusaProviderCapabilityCard
FocusaWorkpointLaunch
FocusaEvidenceSummary
FocusaReceiptCard
FocusaRecoveryCard
FocusaAdvancedDetails
FocusaHelpPopover
```

Unknown components or actions render an explicit unsupported/recovery card and MUST NOT execute.

---

## 10. Nontechnical experience constitution

Default language:

```text
Project
What Focusa knows
What needs your decision
Sources
Project role
Questions
Project plan
Work plan
Proof
Next step
```

Raw `project_root`, continuity, attachment, reducer, schema, route, operation, provider, Evidence-handle, A2UI, AG-UI, and transport details remain under **Advanced details**.

Every surface MUST provide:

- one dominant primary action;
- explanation before input;
- recommendation and source basis;
- consequences and reversibility;
- progressive disclosure;
- autosave and visible save state;
- inline validation;
- explicit recovery;
- keyboard completion;
- visible focus and screen-reader semantics;
- color-independent state and reduced motion;
- responsive desktop, tablet, mobile, Pi, and TUI projection.

UX adaptation MUST use Spec 14 UXP/UFI and MUST NOT change authority, evidence, approval, or safety requirements.

---

## 11. C.R.I.S.T. stage surfaces

### Context

Generate local file/folder input, recommended connector cards, account/folder/label scope, plain-language permissions, import preview, extraction/sync progress, source health, summaries, candidate claims, contradictions, privacy/retention, and next action. Update live as sources change.

### Role

Generate one role-seed input, grounded draft, title, purpose, responsibilities, non-responsibilities, outputs, quality standards, assumptions, sources, redline, permission separation, approve, edit, regenerate-section, and defer actions. Approved Role state is never silently replaced.

### Interview

Implement `focusa.interview.strategy.grill-with-docs.v1`. Each generated question shows one decision, why it matters, facts already checked, recommended answer, sources, consequences, schema-selected input, branch progress, linked gaps, pause, defer, skip, answer, and autosave state. Commit the answer, evaluate it, then stream the next question. Preserve the reviewable compendium.

### Spec

Generate Project Genesis Spec progress, section state, plain-language summaries, proposer/challenger disagreement, contradictions, stale references, sources, pending approvals, one primary action, and launch/open of the full Spec 120 Workbench. Do not duplicate the Workbench.

### Tasks

Generate work-plan summary, parent/child graph, dependencies, blockers, acceptance, proof requirements, provider health, edit/split/merge/reorder, mutation preview, approval, first-task recommendation, and first Workpoint launch.

### Operational continuation

Continue to generate add-context, reconnect, contradiction review, Role revision, Interview continuation, Spec amendment, Task revision, Receipt inspection, and next Workpoint actions after onboarding completes.

---

## 12. UIAI Engine Eval contract

```yaml
schema: uiai.focusa_ui_eval_scenario.v1
scenario_id:
requirement_refs: []
project_scope:
work_surface_ref:
browser_context:
  isolation_class:
  authentication_fixture_ref:
viewport_matrix: []
steps: []
functional_assertions: []
accessibility_assertions: []
diagnostic_assertions: []
visual_assertions: []
reconnect_assertions: []
expected_focusa_events: []
expected_evidence: []
expected_receipts: []
```

```yaml
schema: uiai.focusa_ui_eval_result.v1
scenario_id:
status:
browser_session_refs: []
browser_context_refs: []
step_results: []
screenshots: []
diagnostics: []
accessibility_report_ref:
visual_comparison_refs: []
focusa_evidence_refs: []
receipt_refs: []
failure_class:
recovery_action:
```

UIAI Engine Eval MUST prove onboarding, all C.R.I.S.T. stages, responsive behavior, keyboard/browser accessibility, reconnect, visual state, isolation, recovery, and dogfood flows.

---

## 13. Cross-Functional Alpha amendment

```text
Alpha 0 — contracts, Operation Registry, durable stream, A2UI/Lit, first UIAI Eval
Alpha 1 — Context through generated UI
Alpha 2 — Role and Grill Interview through generated UI, close and resume
Alpha 3 — Spec progress and real Beads task through generated UI
Alpha 4 — Workpoint, Evidence, closure, and Receipt through generated UI
Alpha 5 — UIAI artifact refresh without manual reload
Alpha 6 — two isolated generated Work Surfaces and targeted steering
Alpha 7 — General, Software, and Research projections over identical semantics
Alpha 8 — permanent nontechnical Spec 135 dogfood traversal
```

AG-UI compatibility proceeds in parallel after the native stream and A2UI path are stable. It does not block Alpha 0–8 native traversal.

---

## 14. Mandatory decomposer directive

Every decomposing and implementing agent MUST receive this instruction:

```text
Implement every C.R.I.S.T. and onboarding stage as real-time generated UI for a
nontechnical operator. Do not build static forms, transcript panels, hard-coded
stage screens, or a CLI workflow with a status page.

Use A2UI v0.9.1, @a2ui/web_core/v0_9, and the maintained @a2ui/lit/v0_9
renderer. Build Focusa-specific Svelte Custom Elements for the trusted catalog.
Do not build a complete second Svelte A2UI renderer.

Use Focusa's SQLite event history plus cursor replay and broadcast live tail as
the primary stream. Implement AG-UI only as a compatibility adapter and do not
place it on the native Alpha critical path.

Bind every component action to a generated typed Focusa operation with explicit
scope, capability, permission, preview/commit, idempotency, concurrency,
authority, Evidence, and Receipt posture. Do not create a generic UI mutation
endpoint or duplicate canonical business logic in the UI.

Use OpenAPI 3.0.3, JSON Schema 2020-12, openapi-typescript/openapi-fetch, and
UIAI Engine owns any language-specific adapter outside Focusa core and derives it from the published OpenAPI contract. Do not maintain duplicate DTO authority.

Use UIAI Engine Eval for every browser, end-to-end, responsive, visual,
reconnect, diagnostic, and browser-accessibility proof. Do not add Playwright.

Use the Pi RPC AgentExecutionAdapter and Spec 133 sessions for model-backed
Role, Interview, synthesis, and explanation work. Do not add Vercel AI SDK,
WorkflowAgent, ToolLoopAgent, AI SDK UI, or Vercel AI Gateway as Focusa runtime
owners or dependencies.

Submit general behavior to the greater Focusa primitive before adding the
C.R.I.S.T.-specific projection. Every Alpha slice remains usable through
plain-language generated UI and leaves the permanent dogfood path operational.
```

---

## 15. Ticket contract

```yaml
generated_ui:
  surface_kind:
  requirement_refs: []
  primitive_owner:
  a2ui_catalog_components: []
  read_model_refs: []
  operation_ids: []
  action_binding_refs: []
  plain_language_copy:
  primary_action:
  autosave_behavior:
  resume_behavior:
  recovery_states: []
  advanced_details: []
  terminal_fallback:
  accessibility_tests: []
  uiai_eval_scenarios: []
  evidence_requirements: []
  receipt_requirements: []
```

A missing generated-UI or UIAI Eval section blocks the ticket.

---

## 16. Acceptance criteria

Spec 135I is accepted when:

1. Every C.R.I.S.T. and onboarding stage has a real generated-surface implementation.
2. A2UI v0.9.1 and `web_core` own protocol processing and state.
3. The maintained Lit renderer is the permanent A2UI renderer.
4. Focusa-specific Svelte Custom Elements provide domain components without duplicating the renderer.
5. Native Focusa event replay and live tail update surfaces without manual refresh.
6. AG-UI compatibility exists without owning state or blocking the native Alpha path.
7. OpenAPI 3.0.3 and JSON Schema 2020-12 generate TypeScript clients while remaining the language-neutral contract for external adapters.
8. Every action resolves to a generated typed Focusa operation and exact scope.
9. Every default screen uses plain language, one primary action, progressive disclosure, autosave, resume, and recovery.
10. Context, Role, Interview, Spec, Tasks, Workpoint, Evidence, Receipt, and recovery operate without CLI use.
11. Two concurrent Work Surfaces retain isolated drafts, scope, action bindings, and cursors.
12. General, Software, and Research render identical semantic state through different visual profiles without authority changes.
13. Pi/TUI provides terminal-safe projection and action parity.
14. UIAI Engine Eval proves browser, responsive, accessibility, reconnect, visual, isolation, and recovery behavior.
15. No Playwright dependency or fixture exists.
16. No Vercel AI SDK runtime dependency exists.
17. A nontechnical evaluator completes the full dogfood path without raw diagnostics or developer intervention.

---

## 17. Closure blockers

Spec 135I cannot close while:

- any stage is CLI-only, static-form-only, transcript-only, or mock-only;
- a backend-complete feature lacks generated UI;
- a custom renderer duplicates A2UI web core or Lit;
- AG-UI replaces or blocks the native durable Focusa stream;
- generated actions bypass generated typed Focusa APIs;
- raw IDs, routes, schemas, stack traces, or commands are required in the default flow;
- manual refresh is required during normal operation;
- input is lost during update or restart;
- unsupported actions render enabled;
- concurrent surfaces bleed drafts, scope, authority, or action bindings;
- browser proof uses Playwright or bypasses UIAI Engine Eval;
- model execution bypasses governed Focusa/Pi sessions;
- general reusable behavior remains trapped in C.R.I.S.T. or client code;
- the permanent nontechnical dogfood traversal lacks actual Evidence and Receipts.

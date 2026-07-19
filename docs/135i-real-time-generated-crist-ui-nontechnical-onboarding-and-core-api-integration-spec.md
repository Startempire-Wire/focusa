# Spec 135I — Real-Time Generated C.R.I.S.T. UI, Nontechnical Onboarding, and Core API Integration

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md), [Spec 135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md), [Spec 135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md), [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md), [Spec 135E](135e-cross-spec-amendments-migration-and-closure-matrix.md), and [Spec 135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135I.  
**Scope:** real-time generated user interfaces for the complete C.R.I.S.T. and onboarding lifecycle, nontechnical UX, A2UI declarative surfaces, AG-UI streaming compatibility, Focusa core API integration, trusted component catalogs, action binding, progressive disclosure, accessibility, responsive clients, generated UI testing, speed accelerators, and closure proof.

---

## 0. One-line definition

Every C.R.I.S.T. and onboarding interaction must be presented as a live, incrementally regenerated, plain-language, capability-aware user interface that a nontechnical operator can complete without seeing raw APIs, schemas, identifiers, command syntax, or agent internals, while every read and mutation remains bound to canonical Focusa APIs, scope, authority, Evidence, and Receipts.

---

## 1. Gap closed by this specification

The existing Spec 135 series requires:

- dynamic Interview questions;
- autosave and resume;
- a Project Genesis status view;
- generated contracts;
- dynamic registries;
- real-time invalidation;
- Mission Canvas and Work Surfaces;
- non-admin living-field presentation.

Those requirements are necessary but not sufficient. A static five-stage screen with hard-coded forms could still satisfy much of the prior wording.

This specification closes that ambiguity:

```text
C.R.I.S.T. is not a collection of static forms.
C.R.I.S.T. is not a CLI workflow with a decorative web status page.
C.R.I.S.T. is not a transcript rendered inside a panel.

C.R.I.S.T. is a server-driven, real-time generated interaction experience.
```

Every stage, transition, approval, recovery state, and continuation action is generated from current canonical project state, capabilities, domain packs, operator progress, source health, and authority posture.

---

## 2. Decided protocol and framework stack

This specification makes implementation decisions. Agents must not present alternatives for these choices.

### 2.1 Generated UI protocol

The canonical generated-UI representation is:

```text
A2UI protocol v0.9.1
```

Use:

```text
@a2ui/web_core/v0_9
  Message processing, surface state, data binding, catalog validation,
  incremental updates, and action routing.

@a2ui/lit/v0_9
  Cross-Functional Alpha renderer and compatibility renderer embedded as
  Web Components in Svelte clients.
```

A2UI is Apache-2.0 licensed. Exact package versions are pinned in the dependency matrix and lockfile. The protocol version remains explicitly `v0.9` until a versioned Focusa migration approves another protocol version.

### 2.2 Production Svelte renderer

The full production web experience uses a Focusa Svelte renderer built on `@a2ui/web_core/v0_9` and the existing Focusa component/design stack.

It must not reimplement A2UI message processing, data models, schema validation, component catalogs, or action dispatch.

Required expand-contract sequence:

```text
embed maintained A2UI Lit renderer for immediate Alpha
→ build Focusa Svelte component mappings on web_core
→ prove catalog, action, accessibility, and visual parity
→ make Svelte renderer primary
→ retain Lit renderer as compatibility/test renderer until removal is approved
```

This work remains inside the complete closure graph. The Lit renderer provides immediate usable UI; it does not remove the Svelte renderer requirement.

### 2.3 Real-time agent/user protocol

The real-time interaction compatibility layer is:

```text
AG-UI protocol
@ag-ui/core
@ag-ui/client
```

AG-UI is MIT licensed.

Focusa adopts AG-UI as a compatibility and client-streaming adapter over its existing APIs and SSE event bus. It does not replace Focusa canonical event records, reducers, AX routes, Workpoints, Attachments, Evidence, or Receipts.

Use the middleware translation pattern:

```text
Focusa canonical events and read models
→ Focusa AG-UI adapter
→ lifecycle / activity / tool / state snapshot / state delta events
→ A2UI messages and UI state
→ client action
→ typed Focusa preview/commit API
```

### 2.4 Typed API client

The generated client stack is extended to:

```text
Rust + Serde + Schemars + Utoipa
→ OpenAPI 3.1 + JSON Schema
→ openapi-typescript
→ openapi-fetch
```

`openapi-fetch` is the selected typed HTTP client for web clients. Clients must not hand-write duplicate DTOs or untyped endpoint wrappers when generated contracts cover the operation.

### 2.5 Testing additions

The generated UI test stack is:

```text
Vitest
Svelte Testing Library
Playwright
Schemathesis
A2UI Composer/Theater fixtures
AG-UI event fixtures/Dojo patterns
```

- Vitest and Svelte Testing Library test components, renderers, catalogs, and actions.
- Playwright tests full onboarding, C.R.I.S.T., reconnect, responsive, keyboard, and visual behavior across Chromium, Firefox, and WebKit.
- Schemathesis property-tests generated OpenAPI routes and stateful workflows.
- A2UI fixtures replay incremental surfaces without requiring a live model.

---

## 3. Ownership and authority boundary

```text
Focusa core/reducers
  Canonical project, C.R.I.S.T., scope, Role, Interview, Spec, Tasks,
  Workpoint, Evidence, Receipts, capabilities, and authority.

Focusa API
  Typed read models, preview/commit actions, idempotency, versions,
  generated surface endpoints, and AG-UI compatibility stream.

Generated UI projection
  A2UI surfaces, components, copy, recommendations, validation presentation,
  progress, recovery, and action bindings.

Clients
  Render trusted catalogs, maintain ephemeral presentation state,
  collect user input, and invoke bound typed Focusa actions.
```

The generated UI is a projection. It is not a second canonical store, workflow engine, permission system, Interview database, or task authority.

### 3.1 No generic mutation escape hatch

A generated button or form may invoke only a registered `FocusaUiActionBinding` that resolves to an existing typed Focusa operation or an approved new typed operation.

Forbidden:

```text
POST /v1/ui/execute-arbitrary
POST /v1/ui/run-generated-code
POST /v1/ui/mutate-anything
```

Every consequential action follows:

```text
input validation
→ scope validation
→ capability validation
→ preview where required
→ operator confirmation
→ typed commit operation
→ canonical event
→ Receipt where required
→ generated UI delta
```

---

## 4. Generated Surface Envelope

```yaml
schema: focusa.generated_surface.v1

surface_id:
surface_revision:
surface_kind: onboarding | crist_context | crist_role | crist_interview | crist_spec | crist_tasks | project_profile | recovery

protocol:
  a2ui_version: v0.9
  ag_ui_version:
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
  user_experience_profile_ref:
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

The envelope wraps A2UI messages and Focusa action bindings. It does not create another component-description language.

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
  idempotency_required: true
  optimistic_concurrency_required: true
  receipt_required:
  reversible:

presentation:
  success_message:
  failure_message:
  recovery_action_ref:
```

A2UI component actions reference the binding ID. The client never derives canonical routes or authority from component text.

---

## 6. Core API integration

Required API families:

```text
GET  /v1/ui/catalogs
GET  /v1/ui/catalogs/:catalog_id
GET  /v1/ui/capabilities

POST /v1/ui/surfaces
GET  /v1/ui/surfaces/:surface_id
GET  /v1/ui/surfaces/:surface_id/stream
POST /v1/ui/surfaces/:surface_id/regenerate

GET  /v1/project-genesis/:genesis_id/ui
GET  /v1/project-genesis/:genesis_id/ui/:stage

POST /v1/ui/actions/preview
POST /v1/ui/actions/commit
```

### 6.1 Core implementation placement

Required boundaries or equivalent:

```text
focusa-core/src/ui_intent/
  Pure, bounded interaction intent and action-binding projections from
  canonical state. No A2UI or client dependency required.

focusa-api/src/ui_projection/
  A2UI serialization, catalogs, generated-surface envelopes, and routes.

focusa-api/src/ag_ui/
  AG-UI compatibility adapter over existing Focusa event/read-model streams.

packages/focusa-generated-ui/
  A2UI web_core integration, catalog contracts, renderer adapters,
  query keys, and action client.
```

### 6.2 Existing API reuse law

Generated UI routes compose existing Focusa read models and operations. They must not duplicate Context, Role, Interview, Spec, Task, Workpoint, Evidence, Receipt, connector, provider, or session business logic.

New typed operations are added only when the existing API lacks a real required user action. A UI-specific route may orchestrate presentation, but canonical mutations remain owned by the primitive subsystem.

### 6.3 Surface regeneration

A surface regenerates when relevant state changes:

```text
canonical event
→ invalidation map
→ recompute bounded interaction intent
→ produce A2UI incremental message or fresh snapshot
→ AG-UI stream
→ client renderer update
```

Manual refresh is a recovery action, not normal operation.

---

## 7. Deterministic shell and generative content boundary

The UI is generated, but authority-critical structure is deterministic.

### Deterministic from Focusa state and schemas

- current C.R.I.S.T. stage;
- progress and readiness;
- required fields;
- valid input types;
- action bindings;
- capability and permission posture;
- approval requirements;
- source and evidence references;
- validation rules;
- primary next action;
- recovery actions;
- hidden/non-hideable safety state.

### AI-generated or adapted content

- concise plain-language explanations;
- question wording;
- recommendations and basis summaries;
- contextual help;
- source summaries;
- suggested defaults;
- stage-specific guidance;
- terminology adapted to the active workspace/domain pack.

AI-generated content may not invent actions, routes, permissions, required fields, completion state, evidence, or authority.

---

## 8. Trusted Focusa component catalog

The generated UI may render only pre-approved catalog components.

### 8.1 Base A2UI catalog

Use the maintained A2UI components for standard layout, content, and input primitives where they satisfy the requirement.

### 8.2 Focusa catalog

Required initial components:

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

The catalog is versioned and generated from shared component contracts. Workspace profiles and domain packs may select presentation variants but cannot redefine action authority.

### 8.3 Security

Generated surfaces may not contain:

- arbitrary executable JavaScript;
- arbitrary HTML;
- unvalidated component types;
- unregistered frontend functions;
- unapproved remote iframes;
- direct filesystem paths exposed to nontechnical mode;
- raw secrets, cookies, tokens, or connector credentials.

Unknown components or actions render an explicit unsupported/recovery card and are not executed.

---

## 9. Nontechnical experience constitution

The default C.R.I.S.T. and onboarding experience targets a person who does not know Focusa’s internal terminology.

### 9.1 Plain-language default

The default interface uses:

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

It does not require the user to understand:

```text
project_root
continuity_id
AttachmentKey
reducer
canonical projection
provider-neutral materialization
operation_id
JSON Schema
SSE
A2UI
AG-UI
```

Those details remain available under **Advanced details** for expert diagnosis.

### 9.2 One primary action

Every generated surface has one visually dominant next action. Secondary and advanced actions remain available but subordinate.

Examples:

```text
Add your project information
Connect Google Drive
Review what Focusa found
Approve the project role
Answer the next question
Review the project plan
Approve the work plan
Start the first task
Fix the connection
```

### 9.3 Explain before asking

Before collecting input, the UI explains:

- why the information matters;
- what Focusa already knows;
- the recommended answer or default;
- what will happen after submission;
- whether the decision can be changed later;
- which source supports the recommendation.

### 9.4 Progressive disclosure

The default view shows only information needed for the current decision. Source IDs, schema refs, provider IDs, evidence handles, diagnostics, and raw payloads are collapsed.

### 9.5 Error and recovery language

Do not display raw stack traces or transport errors as the primary message.

Required recovery presentation:

```text
What happened
What is still safe
What Focusa kept
What you can do now
Try again / reconnect / review / continue without this source
Advanced diagnostic details
```

### 9.6 Autosave and confidence

Every accepted input is autosaved. The interface visibly confirms save state and permits return, revision, and supersession.

### 9.7 Accessibility and responsive behavior

Required:

- keyboard completion of all stages;
- visible focus;
- screen-reader labels and live-region updates for generated changes;
- color-independent status;
- reduced motion;
- responsive desktop/tablet/mobile layouts;
- minimum touch targets;
- no interaction available only on hover;
- terminal-safe equivalent for Pi/TUI clients.

---

# Part I — Stage-Generated UI

## 10. Entry and onboarding UI

The first surface is generated from discovered reality.

Inputs:

- whether a project is already bound;
- repository/project markers;
- existing Focusa state;
- current Trajectory/Workpoint;
- available connectors;
- existing context;
- current client capability;
- user experience profile.

The generated entry surface chooses the relevant presentation without asking technical setup questions already answerable from the environment.

Nontechnical entry labels:

```text
Start quickly
  Create a first mission and prove one action.

Build the full project profile
  Let Focusa gather context, define its role, ask questions, build the plan,
  and prepare the work.

Continue setup
  Resume the exact saved stage.

Import an existing project
  Review what Focusa inferred before accepting it.
```

## 11. C — Context generated UI

The Context surface generates:

- local file/folder drop area;
- recommended connector cards based on available capabilities;
- account/folder/label scope selectors;
- plain-language permission explanations;
- import preview;
- extraction/sync progress;
- source health;
- bounded summaries;
- candidate claim review;
- contradiction review;
- privacy/retention explanation;
- next Context action.

The UI updates in real time as documents are discovered, extracted, indexed, contradicted, accepted, or rejected.

A source connector does not appear operational unless its health and capabilities say it is operational.

## 12. R — Role generated UI

The Role surface generates:

- one plain-language role seed input;
- AI-produced role title and summary;
- responsibilities and non-responsibilities;
- expected outputs and quality standards;
- source grounding;
- assumptions and open questions;
- before/after redline on revision;
- permission boundary shown separately;
- approve, edit, regenerate section, and defer actions.

The UI regenerates after new Context or Interview answers affect the draft. Approved Role state is never silently replaced.

## 13. I — Interview generated UI

The Interview surface implements `focusa.interview.strategy.grill-with-docs.v1` through generated UI.

Each question surface contains:

- one primary question;
- why it matters;
- what Focusa already checked;
- recommended answer;
- recommendation sources;
- consequences;
- input control chosen from the answer schema;
- branch progress;
- linked project-plan gaps;
- pause, defer, skip, and answer actions;
- visible autosave state.

After the answer commits, Focusa evaluates it and streams the next generated question surface. The user never fills a large static questionnaire.

The full compendium remains available as a generated review surface with answered, open, deferred, amended, and blocker filters.

## 14. S — Spec generated UI

The generated Spec surface does not duplicate the full Spec Workbench.

It generates:

- Project Genesis Spec progress;
- section states;
- current plain-language section summary;
- proposer/challenger disagreement summary;
- contradictions and stale refs;
- pending approvals;
- source grounding;
- one primary action;
- launch/open the full Spec Workbench;
- section approval cards and final approval confirmation where supported.

The full Workbench remains the rich authoring/research surface. The generated UI makes the workflow understandable and operable for a nontechnical owner.

## 15. T — Tasks generated UI

The Tasks surface generates:

- plain-language work-plan summary;
- parent/child task graph;
- dependencies and blockers;
- acceptance and proof requirements;
- provider capability and connection state;
- task edit/split/merge/reorder controls;
- materialization preview;
- approval action;
- first-task recommendation;
- first Workpoint launch.

Provider names and technical IDs remain secondary. The user sees what work will be created, where it will appear, what it depends on, and what proof will mark it complete.

## 16. Operational continuation UI

After C.R.I.S.T. completes, generated surfaces remain available for:

- add Context;
- reconnect source;
- review new contradiction;
- revise Role;
- continue Interview;
- approve a Spec amendment;
- revise Tasks;
- inspect Receipt;
- launch the next Workpoint.

Project Genesis does not disappear into a completed onboarding record.

---

# Part II — Real-Time Behavior

## 17. AG-UI event mapping

Required mappings:

```text
Focusa operation/run starts
→ AG-UI RUN_STARTED / STEP_STARTED / ACTIVITY_SNAPSHOT

plain-language streaming explanation
→ TEXT_MESSAGE_START / CONTENT / END

typed Focusa tool/action lifecycle
→ TOOL_CALL_START / ARGS / END / RESULT

resolved operating profile or surface state
→ STATE_SNAPSHOT

bounded changes
→ STATE_DELTA using RFC 6902 JSON Patch

A2UI surface messages
→ AG-UI CUSTOM event name: focusa.a2ui.message.v0_9

completion/failure
→ RUN_FINISHED / RUN_ERROR with Focusa recovery refs
```

AG-UI run/thread identifiers remain interaction identifiers. They do not replace Focusa ProjectRootKey, WorkstreamKey, AttachmentKey, Workpoint, or session authority.

## 18. Streaming requirements

- Show stage and activity immediately.
- Render the deterministic shell before AI-generated explanatory copy completes.
- Stream extraction and connector progress.
- Stream Interview recommendation and question when validated.
- Stream Spec Workbench progress and approval needs.
- Stream task-plan changes and provider reconciliation.
- Preserve user input during surface deltas.
- Reconnect using event cursors.
- Request a fresh snapshot after divergence.
- Avoid full-surface replacement when a bounded patch is sufficient.
- Do not render hidden high-frequency surfaces unless subscribed.

## 19. Resume and rehydration

After closing or restarting a client:

```text
load canonical Project Genesis state
→ load current generated-surface envelope
→ resume AG-UI cursor or request fresh snapshot
→ restore unsent local draft separately
→ show saved/unsaved state accurately
```

The client must not infer progress from transcript history.

---

# Part III — Speed and Reuse Opportunities

## 20. No custom generated-UI DSL

Focusa must not create a custom component tree, form-description language, streaming patch language, or generated-UI protocol.

Use A2UI for declarative surfaces and AG-UI for real-time agent/user compatibility.

Focusa-specific behavior belongs in:

- trusted component catalogs;
- typed action bindings;
- deterministic interaction intent;
- domain-pack presentation metadata;
- Focusa API adapters.

## 21. A2UI reuse plan

Use maintained A2UI functionality for:

- message parsing;
- schema validation;
- surface creation;
- incremental component updates;
- data binding;
- basic forms and layout;
- action routing;
- catalog registration;
- multi-surface handling.

Use A2UI Composer and Theater to create and replay fixtures for each C.R.I.S.T. stage before live model integration.

This avoids rebuilding thousands of lines of protocol, state, validation, and rendering infrastructure.

## 22. Schema-driven component and action generation

Generate:

- action input/output TypeScript types from OpenAPI;
- client calls through openapi-fetch;
- input controls from JSON Schema metadata;
- validation messages from schema constraints;
- component catalog contract snapshots;
- UI fixtures from stage read models.

Use the A2UI basic catalog for ordinary layout/input primitives. Create a Focusa custom component only when the basic catalog and existing Focusa design components cannot express a required domain interaction.

## 23. API testing acceleration

Schemathesis becomes a required generated-API gate for:

- schema conformance;
- invalid input rejection;
- preview/commit workflows;
- idempotency;
- optimistic concurrency;
- project/workstream scope;
- stateful C.R.I.S.T. sequences;
- generated UI action bindings.

It supplements, rather than replaces, focused Rust integration tests.

## 24. UI test acceleration

Use:

- A2UI fixture replay for component/surface behavior;
- Vitest for catalog and projection logic;
- Svelte Testing Library for user interaction semantics;
- Playwright for complete nontechnical flows, reconnect, browser/client parity, responsive behavior, and visual snapshots;
- UIAI Engine for screenshot and visual evidence capture.

A live LLM is not required for most generated UI tests. Store deterministic question, recommendation, stage, error, and delta fixtures.

## 25. Generated help and terminology

Use active Workspace View Profiles, domain packs, and approved glossary terms to generate labels and explanations. Do not duplicate terminology logic in each screen.

One `PlainLanguageProjection` service should produce:

- user-facing names;
- short descriptions;
- advanced labels;
- help text;
- technical-term mappings;
- workspace-specific language.

## 26. Catalog-first implementation

Implement generated UI in this order:

```text
A2UI/AG-UI adapter and one StageShell
→ Context components
→ Role components
→ Interview components
→ Spec status/approval components
→ Task-plan components
→ recovery and advanced-details components
→ vertical presentation variants
```

Each component enters a catalog fixture and test gallery immediately.

---

# Part IV — Cross-Functional Alpha Amendment

## 27. Alpha changes

Spec 135H Alpha execution is amended:

### Alpha 0

Add:

```text
A2UI v0.9 surface contract
AG-UI adapter contract
Generated Surface Envelope
UI Action Binding
openapi-fetch generated client
one catalog and one streamed fixture
```

### Alpha 1

The first Context ingestion must be completed through the generated nontechnical onboarding surface, not CLI-only execution.

### Alpha 2

Role approval and Grill Interview must use generated A2UI surfaces and live deltas, including recommendation, answer input, autosave, close, and resume.

### Alpha 3

Spec progress, objection, and approval state must be understandable and actionable through generated UI while the full Spec Workbench remains available.

### Alpha 4

Task-plan approval, first Workpoint selection, Evidence, closure, and Receipt must be surfaced through generated UI.

### Alpha 5

UIAI artifacts must update the relevant generated Work Surface without manual refresh.

### Alpha 6

Two simultaneous generated Work Surfaces must retain independent scope, input drafts, action bindings, and event cursors.

### Alpha 7

The same A2UI semantic surface must render through General, Software, and Research presentation catalogs without changing action bindings or canonical state.

### Alpha 8

The permanent Spec 135 dogfood path is completed by a nontechnical operator entirely through generated UI, except for explicit advanced diagnostics.

---

# Part V — Agent Decomposition Instructions

## 28. Mandatory decomposer directive

Every agent decomposing or implementing Spec 135 must receive this instruction verbatim or equivalently:

```text
Implement every C.R.I.S.T. and onboarding stage as real-time generated UI for a
nontechnical operator. Do not decompose the UI as static forms, a transcript,
a CLI workflow with a status page, or hard-coded stage screens.

Use A2UI v0.9.1 as the generated UI protocol, @a2ui/web_core/v0_9 for message
processing/state/validation, the maintained A2UI Lit renderer for the immediate
Cross-Functional Alpha, and a Focusa Svelte renderer built on web_core for the
full production surface. Use AG-UI as a compatibility adapter over Focusa's
existing APIs and SSE event bus. Do not replace Focusa canonical events or
create a second streaming protocol.

Every generated component action must bind to a typed Focusa operation with
explicit project/workstream scope, capabilities, validation, preview/commit,
idempotency, concurrency, authority, and Receipt posture. Do not create a
generic generated mutation endpoint.

The default experience must use plain language, one primary action, progressive
disclosure, recommendations with sources, autosave, resume, inline validation,
accessible controls, and explicit recovery. Hide raw identifiers, API routes,
schemas, provider IDs, and diagnostic payloads under Advanced details.

Generate the deterministic shell, stage, required fields, validation, actions,
progress, and authority from canonical Focusa state. AI may generate wording,
recommendations, summaries, and help, but may not invent actions, permissions,
completion, evidence, or required fields.

Reuse A2UI's basic catalog, web_core, catalogs, data binding, incremental
updates, multi-surface behavior, and action routing. Build only Focusa-specific
components. Use openapi-typescript + openapi-fetch for typed clients,
Schemathesis for OpenAPI workflow testing, and A2UI fixtures + Vitest + Svelte
Testing Library + Playwright for UI proof.

Every Alpha slice must be usable through generated UI immediately. CLI and raw
JSON remain parity and diagnostic surfaces, not the primary onboarding path.
A feature is incomplete when its backend exists but a nontechnical operator
cannot understand, complete, recover, and resume it through generated UI.
```

## 29. Ticket requirements

Every C.R.I.S.T. implementation ticket must state:

```yaml
generated_ui:
  surface_kind:
  a2ui_catalog_components: []
  read_model_refs: []
  action_binding_refs: []
  ag_ui_events: []
  plain_language_copy:
  primary_action:
  autosave_behavior:
  resume_behavior:
  recovery_states: []
  advanced_details: []
  terminal_fallback:
  accessibility_tests: []
  playwright_flow_ref:
```

A missing generated-UI section blocks the ticket.

---

## 30. Acceptance criteria

Spec 135I is accepted when:

1. Every C.R.I.S.T. stage has a generated-surface implementation.
2. Project onboarding starts and resumes through generated UI.
3. A2UI v0.9.1 is used instead of a custom component protocol.
4. `@a2ui/web_core/v0_9` owns message processing, state, validation, binding, and catalog behavior.
5. The maintained Lit renderer provides the immediate Alpha path.
6. The Focusa Svelte renderer reaches production parity using web_core rather than reimplementing it.
7. AG-UI maps Focusa activity, tools, state snapshots/deltas, and A2UI messages over the existing event architecture.
8. Every generated action resolves to a typed Focusa operation and correct scope.
9. No generic generated mutation endpoint exists.
10. The default UI contains no required raw technical identifiers or command syntax.
11. Every screen has one primary action and progressive disclosure.
12. Recommendations show source basis.
13. All inputs autosave and resume truthfully.
14. Context, Role, Interview, Spec, Tasks, Workpoint, Evidence, Receipt, and recovery flows operate without CLI use.
15. Generated surfaces update incrementally without manual refresh.
16. Two concurrent Work Surfaces retain isolated state and action bindings.
17. General, Software, and Research render the same semantic surface through different visual catalogs without authority changes.
18. Pi/TUI clients provide a terminal-safe generated UI projection and action parity.
19. Vitest, component, Playwright, Schemathesis, reconnect, accessibility, and visual proof pass.
20. A nontechnical evaluator can complete the Spec 135 dogfood flow without assistance from raw diagnostics.

---

## 31. Closure blockers

This specification cannot close while:

- any C.R.I.S.T. stage is CLI-only;
- onboarding is a static wizard disconnected from current canonical state;
- Interview is a static questionnaire or transcript-only experience;
- a backend-complete feature lacks a generated UI path;
- UI components invent or embed authority logic;
- a custom generated-UI or streaming protocol duplicates A2UI or AG-UI;
- generated actions bypass typed Focusa APIs;
- raw IDs, routes, schemas, or stack traces are required in the default flow;
- manual refresh is required during normal operation;
- input is lost during a streamed update or restart;
- unsupported actions render as enabled;
- generated surfaces can execute arbitrary code or components;
- nontechnical error recovery is missing;
- concurrent surfaces bleed drafts, scope, or action bindings;
- Alpha or dogfood proof relies on CLI rather than the generated UI;
- any required client lacks a truthful generated UI or fallback projection.

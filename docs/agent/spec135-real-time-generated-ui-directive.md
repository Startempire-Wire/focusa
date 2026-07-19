# Spec 135 Real-Time Generated UI Directive for Agents

**Authority:** [Spec 135 Series Current Authoritative Delivery Contract](../135-series-current-manifest.md), [Spec 135I](../135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md), [Spec 135J](../135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md), and [Spec 135K](../135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md)  
**Applies to:** every agent decomposing, implementing, reviewing, testing, or closing C.R.I.S.T., Project Genesis, onboarding, Mission Canvas, or Spec 135 client/API work.

## 1. Product rule

Every C.R.I.S.T. and onboarding stage MUST be a real-time generated UI that a nontechnical operator can understand, complete, recover, close, reopen, and resume.

Forbidden primary experiences:

- static wizard pages;
- a fixed questionnaire;
- transcript-only onboarding;
- CLI-first behavior with a decorative status page;
- raw JSON or schema editors;
- hard-coded screens that duplicate Focusa state machines;
- a backend-complete stage with no generated UI.

## 2. Fixed stack

```text
Generated UI
  A2UI v0.9.1
  @a2ui/web_core/v0_9
  @a2ui/lit/v0_9 permanent renderer
  Focusa Svelte Custom Elements in the trusted catalog

Native live state
  Focusa SQLite canonical events
  stable event ID and sequence
  cursor / Last-Event-ID replay
  existing broadcast live tail
  A2UI snapshots and deltas

External compatibility
  AG-UI adapter after the native Focusa/A2UI path is stable

Contracts
  JSON Schema 2020-12
  OpenAPI 3.0.3
  openapi-typescript + openapi-fetch
  oapi-codegen v2.7.x for UIAI Engine

Browser proof
  UIAI Engine Eval only

Components and API proof
  Vitest
  Svelte Testing Library
  Schemathesis
  A2UI Composer/Theater fixtures
```

Do not build a complete custom Svelte A2UI renderer. Do not place AG-UI on the native Alpha critical path. Do not add Playwright.

## 3. Core API and Operation Registry

Every generated action comes from the generated Focusa Operation Registry and contains:

```text
project/workstream/attachment scope
capability
permission
input/output schema
preview/commit posture
idempotency
optimistic concurrency
Receipt requirement
recovery action
```

Action sequence:

```text
A2UI action
→ resolve UI Action Binding
→ load Operation Descriptor
→ validate input and exact scope
→ validate capability and permission
→ preview when required
→ operator confirmation
→ typed Focusa commit
→ shared Focusa ToolResult/error envelope
→ canonical event
→ Evidence / Receipt when required
→ generated UI delta
```

Do not maintain a second route/action catalog in Svelte, A2UI prompts, Pi, UIAI Engine, or connector code. Do not create a generic generated mutation endpoint.

## 4. Required generated surfaces

```text
Onboarding
  project discovery, quick/full path, resume, import review

Context
  source connection, dropzone, scope preview, progress,
  source health, claim review, contradiction review

Role
  seed, grounded draft, assumptions, redline,
  responsibility/permission separation, approval

Interview
  one Grill question, recommendation, sources,
  answer control, branch progress, autosave, defer, resume

Spec
  Workbench progress, sections, objections, grounding,
  approvals, open full Workbench

Tasks
  plan summary, graph, provider state,
  preview/edit/approval/materialization, first Workpoint

Continuation
  add context, revise role, continue interview, amend spec,
  revise tasks, inspect Receipts, launch next Workpoint
```

## 5. Nontechnical UX

Every surface MUST provide:

- plain language;
- one primary action;
- explanation before input;
- what Focusa already knows;
- one recommendation and source basis;
- consequences and reversibility;
- inline validation;
- autosave state;
- pause and resume;
- progressive disclosure;
- explicit safe recovery;
- keyboard and screen-reader behavior;
- responsive and terminal-safe presentation.

Raw IDs, routes, schemas, stack traces, transport data, and evidence handles belong under **Advanced details**.

UX adaptation MUST use Spec 14 UXP/UFI. It MUST NOT create a second personalization profile or change authority, evidence, approval, scope, or safety.

## 6. Deterministic and generated boundary

Deterministic from canonical state:

- stage, progress, and readiness;
- required fields and validation;
- action bindings;
- scope, capabilities, and permissions;
- approval and Evidence requirements;
- primary action and recovery;
- non-hideable safety state.

AI can generate wording, recommendations, source summaries, question phrasing, and help. AI MUST NOT invent actions, permissions, required fields, completion, Evidence, or authority.

Loading, progress, validation, capability, approval, recovery, and schema-driven input surfaces MUST render without an LLM call.

## 7. Durable stream

```text
client supplies cursor / Last-Event-ID
→ replay missed matching events from SQLite
→ subscribe to broadcast live tail
→ deduplicate by stable ID/sequence
→ emit A2UI snapshot/delta
```

AG-UI translates this stream for external compatibility only.

Required behavior:

- deterministic shell renders immediately;
- progress streams while work runs;
- drafts survive unrelated deltas;
- lagged clients replay rather than lose state;
- manual refresh is recovery-only;
- concurrent Work Surfaces retain separate drafts, scope, bindings, and cursors.

Do not add Redis, Kafka, NATS, a UI event database, or a second message broker for this architecture.

## 8. Model execution

Model-backed Role, Grill Interview, synthesis, and explanatory generation use:

```text
Focusa typed operation
→ Spec 133 governed session
→ Pi RPC AgentExecutionAdapter
→ structured result
→ reducer
→ Evidence / Receipt
→ generated UI
```

Do not add Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, `@ai-sdk/svelte`, Vercel AI Gateway as a required dependency, or another model/tool authority.

## 9. UIAI Engine Eval

All browser, end-to-end, visual, responsive, reconnect, diagnostic, isolation, and browser-accessibility proof uses UIAI Engine Eval.

Focusa MUST NOT introduce Playwright Test, Playwright Library, Playwright CLI, Playwright MCP, `@playwright/test`, `playwright.config.*`, or Playwright fixtures.

Every browser-facing ticket references one or more `uiai.focusa_ui_eval_scenario.v1` scenarios and expected Evidence/Receipt outputs.

## 10. Ticket requirement

```yaml
generated_ui:
  requirement_refs: []
  primitive_owner:
  surface_kind:
  operation_ids: []
  a2ui_catalog_components: []
  read_model_refs: []
  action_binding_refs: []
  capability_refs: []
  durable_event_cursor:
  plain_language_copy:
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

A missing generated-UI or UIAI Eval section blocks the ticket.

## 11. Cross-Functional Alpha

Alpha 0 establishes generated contracts, Operation Registry, action bindings, capability projection, ToolResult mapping, durable native stream, A2UI/Lit, Pi AgentExecutionAdapter, and the first UIAI Eval scenario.

Every Alpha slice is completed through generated UI. AG-UI compatibility proceeds in parallel and does not block the native traversal.

Permanent path:

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

A feature is incomplete when its backend exists but a nontechnical operator cannot understand, complete, recover, and resume it through generated UI.

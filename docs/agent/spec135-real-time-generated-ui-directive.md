# Spec 135 Real-Time Generated UI Directive for Agents

**Authority:** [Spec 135I](../135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md)  
**Applies to:** every agent decomposing, implementing, reviewing, testing, or closing C.R.I.S.T., Project Genesis, onboarding, Mission Canvas, or Spec 135 client work.

---

## Mandatory product rule

Every C.R.I.S.T. and onboarding stage is implemented as real-time generated UI for a nontechnical operator.

Do not implement:

- static wizard pages as the primary experience;
- a large fixed questionnaire;
- transcript-only onboarding;
- CLI-first behavior with a decorative status page;
- raw JSON/schema editors as the default interface;
- hard-coded screen logic that duplicates Focusa core state machines;
- a backend-complete stage with no usable generated UI.

The default experience must let a nontechnical person understand, complete, recover, leave, and resume the full process.

---

## Decided generated-UI stack

Use:

```text
Generated UI protocol
  A2UI v0.9.1

Message/state/catalog processing
  @a2ui/web_core/v0_9

Immediate Cross-Functional Alpha renderer
  @a2ui/lit/v0_9 embedded in Svelte through Web Components

Full production renderer
  Focusa Svelte mappings built on @a2ui/web_core/v0_9

Real-time compatibility protocol
  AG-UI over the existing Focusa API and SSE architecture

Typed client
  openapi-typescript + openapi-fetch

Testing
  A2UI fixtures/Composer/Theater
  Vitest
  Svelte Testing Library
  Playwright
  Schemathesis
```

Do not ask the operator to select another protocol, renderer, form engine, state protocol, or client stack.

---

## Core API boundary

The generated UI is a projection. Focusa core remains canonical.

Every generated action binds to a registered typed Focusa operation with:

```text
project/workstream scope
capability
permission
input schema
preview/commit posture
idempotency
optimistic concurrency
Receipt requirement
recovery action
```

Do not create a generic UI mutation endpoint or execute model-generated code.

Required action sequence:

```text
validate input
→ validate scope and capability
→ preview where required
→ operator confirmation
→ typed Focusa commit
→ canonical event
→ Receipt where required
→ generated UI delta
```

---

## Required stage surfaces

```text
Onboarding
  project discovery, quick/full path, resume, import review.

Context
  source connection, dropzone, scope preview, import progress,
  source health, claim and contradiction review.

Role
  seed, generated draft, grounding, assumptions, redline,
  responsibility/permission separation, approval.

Interview
  one Grill-with-Docs question, recommendation, source basis,
  answer control, branch progress, autosave, defer, resume.

Spec
  Workbench progress, section states, objections, grounding,
  approvals, and launch/open Workbench.

Tasks
  work-plan summary, dependency graph, provider state,
  preview/edit/approval/materialization, first Workpoint.

Operational continuation
  add context, revise role, continue interview, amend spec,
  revise tasks, inspect Receipts, launch next Workpoint.
```

---

## Nontechnical UX requirements

Every generated surface must provide:

- plain language;
- one primary action;
- why the action matters;
- what Focusa already knows;
- a recommended answer/default with sources;
- what happens next;
- inline validation;
- autosave state;
- pause/resume;
- progressive disclosure;
- explicit safe recovery;
- keyboard and screen-reader behavior;
- responsive and terminal-safe presentation.

Raw IDs, routes, schemas, stack traces, evidence handles, and transport details belong under **Advanced details**.

---

## Generation boundary

Generate deterministically from canonical Focusa state:

- stage;
- readiness and progress;
- required fields;
- input types and validation;
- action bindings;
- capabilities and permissions;
- approvals;
- primary next action;
- recovery actions.

AI may generate:

- concise wording;
- recommendations;
- explanations;
- source summaries;
- question phrasing;
- contextual help.

AI may not invent actions, permissions, required fields, completion, evidence, or authority.

---

## Real-time requirements

Use AG-UI lifecycle/activity/tool/state events and A2UI incremental messages.

Required behavior:

- deterministic shell renders immediately;
- progress streams while work runs;
- state changes produce bounded deltas;
- user input survives incoming deltas;
- clients reconnect by cursor or request a fresh snapshot;
- hidden surfaces do not consume unnecessary high-frequency updates;
- manual refresh is recovery-only;
- concurrent Work Surfaces preserve separate drafts, scopes, bindings, and cursors.

---

## Ticket requirement

Every relevant implementation ticket includes:

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

## Cross-Functional Alpha rule

Every Alpha slice is completed through generated UI, not only through CLI or raw API calls.

The permanent path is:

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

A feature is incomplete when its backend exists but a nontechnical operator cannot understand, complete, recover, and resume it through generated UI.

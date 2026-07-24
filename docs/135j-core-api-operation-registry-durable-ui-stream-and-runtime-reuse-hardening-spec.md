# Spec 135J — Core API Operation Registry, Durable UI Stream, and Runtime Reuse Hardening

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135J.  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](135-series-current-manifest.md) governs current contract, streaming, generated-UI, and compatibility decisions.

---

## 0. One-line definition

Generated C.R.I.S.T. UI is a thin projection over one typed Focusa API and one canonical Focusa event history: actions are generated from registered operations, updates replay from SQLite and tail the existing event bus, capabilities and permissions reuse existing authority systems, and recovery reuses the shared ToolResult envelope.

---

## 1. Existing seams to reuse

Focusa already has:

- Axum route families;
- project/workstream/attachment scope and permission checks;
- shared HTTP error-envelope middleware;
- `tool_result_v1` conventions;
- an existing in-process broadcast channel;
- SQLite canonical events with bounded cursor reads;
- Evidence and Receipts;
- Workpoints, Instances, Sessions, and Attachments;
- capability read surfaces.

Generated UI MUST NOT create:

- a second event database;
- a UI workflow authority;
- a manual route/action registry;
- a UI permission system;
- a second error or retry taxonomy;
- a canonical frontend state store;
- direct client access to SQLite;
- route-local copies of core business rules.

---

## 2. Ownership

```text
focusa-core
  canonical reducers, domain services, read models, UiInteractionIntent,
  capabilities, permissions, and authority decisions

focusa-api
  Axum routes, OpenAPI 3.0.3, Operation Registry, UI projections,
  action bindings, durable native event stream, AG-UI compatibility

focusa-client
  generated typed HTTP client, query keys, reconnect, action invocation

A2UI web core and permanent Lit renderer
  trusted rendering, local field drafts, validation presentation, dispatch

Focusa Svelte Custom Elements
  domain-specific catalog components only
```

No UI route or component reproduces a reducer rule.

---

## 3. Generated Focusa Operation Registry

The authoritative registry is generated from Rust route/input/output definitions and Utoipa/OpenAPI metadata.

```yaml
schema: focusa.operation_descriptor.v1
operation_id:
route:
method:
summary:
description:

ownership:
  subsystem:
  core_action_ref:

contracts:
  input_schema_ref:
  output_schema_ref:
  error_schema_ref: focusa.tool_result.v1

scope:
  required_keys: []
  project_scoped:
  workstream_scoped:
  attachment_scoped:

control:
  capability_refs: []
  permission_scopes: []
  mode: read | preview | commit
  confirmation: none | simple | consequential
  idempotency_required:
  optimistic_concurrency_required:
  receipt_required:
  reversible:

ui:
  allowed_in_generated_ui:
  default_label:
  plain_language_description:
  input_presentation_ref:
  result_presentation_ref:
  advanced_only:
  sensitivity:
```

### OpenAPI vendor extensions

```text
x-focusa-subsystem
x-focusa-core-action
x-focusa-scope-keys
x-focusa-capabilities
x-focusa-permissions
x-focusa-mode
x-focusa-confirmation
x-focusa-idempotency
x-focusa-concurrency
x-focusa-receipt
x-focusa-reversible
x-focusa-generated-ui
x-focusa-plain-label
x-focusa-advanced-only
x-focusa-sensitive
```

Generate from one source:

```text
Rust + Serde + Schemars + Utoipa
→ JSON Schema 2020-12
→ OpenAPI 3.0.3
→ Operation Registry
→ TypeScript openapi-typescript/openapi-fetch
→ external adapters generated from published OpenAPI outside Focusa core
→ A2UI catalog schemas and UI action bindings
```

Manual equivalent metadata is forbidden.

---

## 4. Generated UI action binding

```text
Operation Descriptor
+ ProjectRootKey / WorkstreamKey / AttachmentKey
+ capability and permission snapshot
+ canonical revision
→ focusa.ui_action_binding.v1
```

Action execution:

```text
A2UI action
→ validate trusted catalog/action binding
→ load Operation Descriptor
→ validate schema and exact scope
→ validate capability and permission
→ preview when required
→ operator confirmation
→ invoke typed Focusa operation
→ shared Focusa ToolResult/error envelope
→ canonical event
→ Evidence / Receipt when required
→ A2UI delta
```

There is no generic mutation route that bypasses the Operation Registry.

---

## 5. Capability and permission projection

```yaml
schema: focusa.ui_capability_snapshot.v1
project_root:
continuity_id:
attachment_id:
agent_id:
capabilities:
  - capability_id:
    status: available | degraded | unavailable | approval_required
    reason:
    recovery_action_ref:
permissions:
  granted_scopes: []
  missing_scopes: []
providers: []
connectors: []
client_capabilities: []
source_state_revision:
```

This projection composes existing middleware, capability APIs, provider health, connector health, and client capabilities. It is not a second permission registry.

---

## 6. Shared result and recovery envelope

All generated errors, blocked states, retries, and recovery cards derive from the shared Focusa ToolResult/error envelope.

Required consumed fields:

```text
status
canonical
degraded
failure_class
summary/message
retry
recovery_hint
misuse_hint
side_effects
evidence_refs
next_tools
correlation_id
```

### One typed envelope implementation

```text
add shared typed ToolResult/error constructors
→ migrate route families in bounded batches
→ verify compatibility
→ remove duplicate route-local builders
```

PlainLanguageProjection adapts presentation without changing failure semantics.

---

## 7. Durable replayable event stream

### One event history

```text
SQLite canonical events
  durable history, cursor replay, recovery

existing in-process broadcast channel
  low-latency live tail
```

Do not add Redis, Kafka, NATS, AG-UI storage, a UI event log, or another broker.

### Algorithm

```text
client connects with cursor / Last-Event-ID
→ read missed matching events from SQLite
→ emit replay in canonical order
→ subscribe to broadcast live tail
→ deduplicate by stable event ID and sequence
→ continue until disconnect
```

A lagged receiver replays from SQLite before resuming the tail.

### Event envelope

```yaml
schema: focusa.stream_event.v1
event_id:
sequence:
timestamp:
event_type:
schema_version:
scope:
  project_root:
  continuity_id:
  attachment_id:
  work_surface_id:
source_state_revision:
payload_ref:
invalidate: []
correlation_id:
causation_id:
```

Large payloads remain behind handles.

### APIs

```text
GET /v1/events/stream
  cursor, Last-Event-ID, filters, replay, live tail

GET /v1/events/recent
  bounded durable read and recovery

GET /v1/ui/surfaces/:surface_id/stream
  native Focusa/A2UI surface stream over the same history

POST /v1/ag-ui/run
  external compatibility translation over the native stream
```

---

## 8. AG-UI translation boundary

AG-UI is a generated compatibility view:

```text
Focusa run/action event → AG-UI lifecycle/activity/tool event
Focusa surface snapshot → AG-UI STATE_SNAPSHOT
bounded surface update → AG-UI STATE_DELTA
A2UI message → AG-UI CUSTOM focusa.a2ui.message.v0_9
```

AG-UI IDs never replace ProjectRootKey, WorkstreamKey, AttachmentKey, Focusa Session, Workpoint, or canonical event IDs.

AG-UI implementation proceeds after the native replay/A2UI path is stable and MUST NOT block Alpha 0–8 native delivery.

---

## 9. Read-model and UI-intent reuse

```text
canonical state and services
→ bounded subsystem read models
→ Resolved Project Operating Profile
→ UiInteractionIntent
→ Generated Surface Envelope / A2UI messages
```

UiInteractionIntent contains stage, readiness, primary action, required decisions, allowed operation IDs, source/Evidence refs, recovery posture, plain-language semantic inputs, and catalog hints. It contains no client framework objects.

---

## 10. Surface cache and invalidation

Cache deterministic surfaces by:

```text
project/workstream/attachment scope
+ canonical revision
+ surface kind
+ catalog/version
+ workspace profile
+ domain-pack composition
+ UXP/accessibility profile
```

AI-generated wording is cached separately. Registered event-to-read-model keys drive targeted invalidation. Do not regenerate every surface after every event.

TanStack Query remains the web cache/refetch layer and never becomes canonical state.

---

## 11. Speed and reuse decisions

### Do not hand-author A2UI action definitions

Generate action schemas and bindings from the Operation Registry.

### JSON Schema + x-focusa UI annotations

Ordinary strings, numbers, booleans, dates, enums, arrays, and file references map to maintained A2UI inputs and inline validation. Custom Focusa components are limited to genuine domain interactions.

### Deterministic UI without model calls

Loading, progress, validation, capabilities, approvals, recovery, and standard action surfaces render without a model call.

### Scaffold from fixtures

Implement catalog and surfaces against generated schemas, read-model snapshots, and A2UI fixtures while backend operations are implemented. Replace fixtures with live operations without changing contracts.

### Schemathesis stateful preview/commit tests

Use generated OpenAPI workflows for read/preview/commit, idempotency, concurrency, permission, and failure-envelope proof.

### UIAI Engine Eval

All browser, end-to-end, visual, responsive, reconnect, isolation, diagnostic, and browser-accessibility proof uses UIAI Engine Eval. Focusa MUST NOT add Playwright.

---

## 12. API conformance and proof

Required:

- Utoipa/OpenAPI 3.0.3 generation tests;
- JSON Schema 2020-12 tests;
- Operation Registry snapshots;
- TypeScript client and portable OpenAPI/JSON Schema drift gates;
- action-binding tests;
- capability/permission projection tests;
- ToolResult compatibility tests;
- Schemathesis stateful tests;
- durable replay, lag recovery, ordering, deduplication, and isolation tests;
- native A2UI action-to-operation tests;
- AG-UI translation fixtures;
- surface cache/invalidation tests;
- no-direct-SQL-in-UI static gate;
- UIAI Engine Eval scenarios for browser-facing operations.

---

## 13. Cross-Functional Alpha

Alpha 0 delivers:

```text
JSON Schema/OpenAPI 3.0.3
→ generated TypeScript clients and portable OpenAPI/JSON Schema contracts
→ Operation Registry and UI action bindings
→ capability snapshot
→ shared ToolResult mapping
→ durable native event stream
→ A2UI web core and permanent Lit renderer
→ Pi RPC AgentExecutionAdapter
→ first UIAI Engine Eval scenario
→ one real Context generated surface
```

AG-UI compatibility proceeds in parallel after the native stream stabilizes. No Alpha slice introduces a manual action or second state path.

---

## 14. Agent directive

```text
Integrate generated C.R.I.S.T. UI through existing Focusa core and typed APIs.
Generate operations and action bindings from Rust/OpenAPI 3.0.3. Reuse exact
scope, capabilities, permissions, ToolResult envelopes, SQLite events, broadcast
tail, read models, Evidence, Receipts, Workpoints, and Attachments.

Implement the native replayable Focusa/A2UI stream first. AG-UI is external
compatibility and never owns history or blocks native Alpha delivery.

Use maintained A2UI web core and Lit renderer. Add Svelte Custom Elements only
for Focusa domain interactions. Render deterministic states without model calls.
Use Pi RPC/Spec 133 for model work and UIAI Engine Eval for browser proof.
Do not add Playwright, Vercel AI SDK runtime authority, a generic UI mutation
route, or any duplicate store, registry, permission system, or error taxonomy.
```

---

## 15. Acceptance criteria

Spec 135J is accepted when:

1. one Generated Focusa Operation Registry describes every generated-UI operation;
2. action bindings are generated and exactly scoped;
3. no generic mutation escape hatch exists;
4. current core actions/read models remain canonical;
5. capabilities and permissions drive availability;
6. all recovery uses the shared ToolResult envelope;
7. route-local duplicate envelope builders are removed;
8. missed events replay from SQLite before the live tail;
9. event IDs, sequence, cursor, deduplication, ordering, and isolation are proven;
10. AG-UI stores no history and does not block native Alpha;
11. ordinary inputs and actions are generated from schemas;
12. deterministic surfaces need no LLM;
13. cache and targeted invalidation are proven;
14. TypeScript client and portable OpenAPI/JSON Schema drift gates pass;
15. UIAI Engine Eval proves browser-facing flows;
16. Alpha 0 establishes the shared spine used by every later slice.

## 16. Closure blockers

Spec 135J cannot close while generated UI has a manual registry; a generic mutation route exists; UI routes duplicate core rules; permissions/errors have UI-specific stores; event replay is incomplete; AG-UI owns state/history or blocks native delivery; contracts drift across Rust/TypeScript or the portable OpenAPI/JSON Schema boundary; a custom renderer duplicates A2UI; browser proof bypasses UIAI Engine Eval; Playwright exists in Focusa; or the complete native Alpha spine lacks Evidence and Receipt proof.

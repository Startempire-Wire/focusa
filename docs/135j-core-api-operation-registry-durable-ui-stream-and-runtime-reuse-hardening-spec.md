# Spec 135J — Core API Operation Registry, Durable UI Stream, and Runtime Reuse Hardening

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-18  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Amends:** [Spec 109](109-agent-first-api-redesign-ax-spec.md), [Spec 135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md), [Spec 135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md), [Spec 135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md), and [Spec 135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md)  
**Closure relationship:** mandatory companion; Spec 135 cannot close without Spec 135J.  
**Scope:** generated operation registry, OpenAPI-derived UI action bindings, durable replayable SSE, AG-UI translation, canonical event reuse, shared ToolResult/error envelopes, capability and permission projection, read-model reuse, schema annotations, caching, and API integration proof.

---

## 0. One-line definition

The generated C.R.I.S.T. UI must be a thin projection over one typed Focusa API and one canonical event history: action bindings are generated from registered operations, live updates replay from SQLite and tail the existing event bus, permissions and capabilities come from existing authority systems, and recovery comes from the shared result envelope rather than parallel UI-specific logic.

---

## 1. Current runtime seams to reuse

Focusa already has:

- Axum route families;
- project/workstream scope and permission checks;
- a shared HTTP error-envelope middleware;
- `tool_result_v1` response conventions and JSON schema;
- an in-process broadcast channel for live events;
- a canonical SQLite event history with bounded cursor reads;
- Evidence and Receipt systems;
- typed scoped state, Workpoints, Instances, Sessions, and Attachments;
- project and agent capability read surfaces.

The generated UI implementation extends these seams. It must not create:

- a second event database;
- a UI-only workflow engine;
- a manually maintained second route/action registry;
- a UI permission system;
- a second error taxonomy;
- a UI canonical state store;
- direct client access to SQLite;
- route-local copies of Focusa business rules.

---

## 2. Core ownership model

```text
focusa-core
  Canonical reducers, state, domain services, interaction intent,
  read-model builders, capabilities, authority decisions.

focusa-api
  Axum routes, generated OpenAPI, operation registry, UI projections,
  action binding, durable event streaming, AG-UI translation.

focusa-client
  Generated typed HTTP client, query keys, reconnect, action invocation.

A2UI renderers
  Trusted rendering, local field state, validation display, action dispatch.
```

No UI route may directly reproduce a reducer rule already owned by `focusa-core`.

---

## 3. Generated Focusa Operation Registry

The authoritative operation registry is generated from Rust route/input/output definitions and Utoipa/OpenAPI metadata.

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

### 3.1 OpenAPI annotations

Use OpenAPI vendor extensions generated from Rust metadata:

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

The registry, API reference, generated client, and UI action catalog are generated from one source. Agents must not hand-maintain equivalent metadata in Svelte, A2UI prompts, or connector code.

---

## 4. Generated UI action binding

`focusa.ui_action_binding.v1` is generated from the Operation Registry plus resolved project/workstream scope and current capability snapshot.

```text
Operation Descriptor
+ current ProjectRootKey / WorkstreamKey / AttachmentKey
+ capability and permission snapshot
+ current canonical revision
→ UI Action Binding
```

The generated UI may narrow or hide an operation because it is unavailable. It may not broaden its authority.

### 4.1 Action execution

```text
A2UI action event
→ validate catalog/action binding
→ load Operation Descriptor
→ validate scope/capability/permission
→ validate schema
→ preview if required
→ confirm
→ invoke existing typed Focusa route/core action
→ return shared ToolResult envelope
→ persist canonical event/Receipt where required
→ stream UI delta
```

There is no generic generated mutation route that bypasses the Operation Registry.

---

## 5. Capability and permission projection

Create one bounded projection:

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

This projection composes existing permission middleware, capabilities APIs, provider health, connector health, and client capabilities. It does not create a parallel permission registry.

Generated components and actions resolve availability from this snapshot.

---

## 6. Shared result and recovery envelope

All generated UI errors, blocked states, retries, and recovery cards derive from the shared Focusa response envelope and `tool_result_v1` schema.

Required fields consumed by the UI include:

```text
status
canonical
degraded
failure_class
summary / message
retry
recovery_hint
misuse_hint
side_effects
evidence_refs
next_tools
correlation_id
```

### 6.1 One typed envelope implementation

Replace route-local copies of failure-envelope builders through expand-contract migration:

```text
add shared typed ToolResult/Error constructors
→ adapt route families in bounded batches
→ verify response compatibility
→ remove duplicate route-local builders
```

Generated UI must not define a second recovery taxonomy. `PlainLanguageProjection` converts the shared envelope into nontechnical copy while preserving advanced diagnostic details.

---

## 7. Durable replayable event stream

### 7.1 One event history

Use:

```text
SQLite canonical events
  Durable history, replay, cursor recovery.

existing in-process broadcast channel
  Low-latency live tail.
```

Do not add an AG-UI event database, UI event log, Redis stream, or message broker for the initial architecture.

### 7.2 Durable stream algorithm

```text
client connects with cursor / Last-Event-ID
→ read missing matching events from SQLite
→ emit replay events in canonical order
→ subscribe to broadcast live tail
→ deduplicate by stable event ID/sequence
→ continue until disconnect
```

The current behavior that silently drops lagged broadcast events is not sufficient for generated UI. A lagged receiver must replay from the canonical SQLite cursor before resuming the live tail.

### 7.3 Required event envelope

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

Large payloads remain behind stable refs.

### 7.4 Stream APIs

```text
GET /v1/events/stream
  Adds cursor/Last-Event-ID, scope filters, event IDs, sequence, and replay.

GET /v1/events/recent
  Remains the bounded durable read/recovery route.

GET /v1/ui/surfaces/:surface_id/stream
  Filtered generated-surface stream composed from the same event history.

POST /v1/ag-ui/run or equivalent typed AG-UI adapter route
  Translates Focusa events; does not persist a second history.
```

---

## 8. AG-UI translation boundary

AG-UI events are generated views over Focusa events and operation activity.

```text
Focusa run/action event
→ AG-UI lifecycle/activity/tool event

Focusa generated surface state
→ AG-UI STATE_SNAPSHOT

bounded surface update
→ AG-UI STATE_DELTA

A2UI message
→ AG-UI CUSTOM focusa.a2ui.message.v0_9
```

AG-UI `threadId` and `runId` are interaction references. They never replace ProjectRootKey, WorkstreamKey, AttachmentKey, Focusa Session, Workpoint, or canonical event IDs.

---

## 9. Read-model and UI-intent reuse

Generated surfaces are built from bounded read models and pure interaction intent, not route-local database queries.

Required pattern:

```text
canonical state/services
→ bounded subsystem read models
→ Resolved Project Operating Profile
→ UiInteractionIntent
→ Generated Surface Envelope / A2UI messages
```

`UiInteractionIntent` contains:

- current stage and readiness;
- primary action;
- required decisions;
- available operation IDs;
- source/evidence refs;
- recovery posture;
- plain-language semantic inputs;
- component/catalog hints.

It does not contain client framework objects.

---

## 10. Surface cache and invalidation

Cache generated deterministic surfaces by:

```text
project/workstream/attachment scope
+ canonical source-state revision
+ generated-surface kind
+ catalog/version
+ workspace profile
+ domain-pack composition
+ language/accessibility profile
```

AI-generated wording may be cached separately from deterministic surface structure.

Invalidation uses registered event-to-read-model keys. It must not regenerate all C.R.I.S.T. surfaces after every event.

---

## 11. Speed decisions

### 11.1 Generate action catalogs

Do not hand-author A2UI action definitions. Generate catalog action schemas and bindings from the Operation Registry.

### 11.2 Generate ordinary inputs

For ordinary strings, numbers, booleans, dates, enums, arrays, and file references:

```text
JSON Schema + x-focusa UI annotations
→ A2UI basic input component
→ inline validation
```

Create a custom Focusa component only for interactions such as redlines, claim review, source scope, dependency graphs, evidence, Receipts, or Workpoint launch.

### 11.3 Deterministic UI without model calls

Loading, progress, validation, capability, approval, recovery, and standard action surfaces must render without an LLM call.

Use models only for:

- Interview questions and recommendations;
- Role drafting;
- source summaries;
- plain-language explanation where a cached/template projection is insufficient;
- Spec Workbench cognition.

### 11.4 Reuse current client query layer

TanStack Query remains the web cache/refetch layer. AG-UI/A2UI events invalidate or patch the same query/read-model keys; they do not create a competing canonical frontend store.

### 11.5 Scaffold from fixtures

Before backend completion, use generated OpenAPI schemas, A2UI fixtures, AG-UI event fixtures, and snapshot read models to implement and prove components. Replace fixtures with live routes without changing the component contracts.

---

## 12. API conformance and testing

Required:

- Utoipa/OpenAPI generation tests;
- Operation Registry snapshot tests;
- action-binding generation tests;
- capability/permission projection tests;
- shared ToolResult envelope compatibility tests;
- Schemathesis stateful preview/commit tests;
- durable SSE replay and lag recovery tests;
- duplicate-event and ordering tests;
- project/workstream/attachment stream isolation tests;
- AG-UI translation fixture tests;
- A2UI action-to-operation tests;
- surface cache/invalidation tests;
- no-direct-SQL-in-UI-projection static gate.

---

## 13. Cross-Functional Alpha amendment

Alpha 0 must deliver:

```text
OpenAPI operation registry
→ generated TypeScript/openapi-fetch client
→ generated UI action binding
→ capability snapshot
→ shared result envelope mapping
→ one replayable event stream
→ one A2UI surface updated through AG-UI
```

No later Alpha slice may introduce an unregistered manual UI action or a second stream/state path.

---

## 14. Agent decomposition directive

Every decomposing and implementing agent must receive this instruction verbatim or equivalently:

```text
Integrate generated C.R.I.S.T. UI through the existing Focusa core and API.
Generate UI actions from one Rust/OpenAPI Operation Registry. Reuse existing
scope, permissions, capabilities, ToolResult/error envelopes, canonical SQLite
events, broadcast event tail, read models, Evidence, Receipts, Workpoints, and
Attachments. Do not build a UI-only workflow engine, permission registry, event
store, route catalog, error taxonomy, or canonical frontend state.

Upgrade the current SSE path by replaying missed events from SQLite using a
stable event cursor/Last-Event-ID, then tailing the existing broadcast channel.
AG-UI translates this stream and does not own another history.

Generate ordinary A2UI inputs and action schemas from JSON Schema/OpenAPI
metadata. Add custom components only for genuine Focusa domain interactions.
Render deterministic shell, progress, validation, recovery, and capability
states without model calls. Use model generation only where cognition or
plain-language synthesis is actually required.
```

---

## 15. Acceptance criteria

Spec 135J is accepted when:

1. One generated Operation Registry describes every generated-UI operation.
2. A2UI action bindings are generated from that registry and resolved scope/capabilities.
3. No generic mutation escape hatch exists.
4. Generated UI uses existing Focusa core actions and read models.
5. Existing permission and capability systems drive component/action availability.
6. All generated UI recovery derives from the shared ToolResult/error envelope.
7. Route-local duplicate envelope builders are removed through expand-contract migration.
8. The live stream replays missed events from SQLite before tailing broadcast.
9. Stable event IDs, sequence, cursor/Last-Event-ID, deduplication, and ordering are proven.
10. AG-UI persists no second event history.
11. Project/workstream/attachment stream isolation is proven.
12. Ordinary inputs and action schemas are generated from OpenAPI/JSON Schema.
13. Deterministic surfaces do not require an LLM call.
14. Surface caching and targeted invalidation are proven.
15. Schemathesis, SSE replay, envelope, operation-registry, AG-UI, and A2UI binding tests pass.
16. Alpha 0 establishes the complete generated UI/API spine used by all later slices.

---

## 16. Closure blockers

This specification cannot close while:

- generated UI has a manually maintained route/action registry;
- UI action availability duplicates permission logic;
- a UI-specific error taxonomy exists;
- route families continue adding new duplicate envelope builders;
- the live stream silently loses lagged events without replay;
- AG-UI or A2UI state is treated as canonical project state;
- a second event store or broker is introduced without approved evidence;
- generated UI reads SQLite directly;
- ordinary schema-driven fields require custom components;
- deterministic UI unnecessarily waits for a model call;
- an Alpha slice bypasses the Operation Registry or durable stream.

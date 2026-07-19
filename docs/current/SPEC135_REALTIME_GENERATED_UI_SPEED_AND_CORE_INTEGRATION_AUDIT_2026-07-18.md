# Spec 135 Real-Time Generated UI Speed and Core Integration Audit

**Audit date:** 2026-07-18  
**Scope:** Current Focusa code and Spec 135A–135K  
**Status:** implementation reality and mandatory migration guidance

---

## 1. Verdict

Before Specs 135I–135K, the series required dynamic Interview questions, autosave, resumability, Mission Canvas updates, and a Project Genesis UI, but it did not prohibit a static five-stage wizard or require real-time generated UI for every C.R.I.S.T. stage.

The requirement is now explicit:

```text
Every onboarding and C.R.I.S.T. stage
→ real-time generated A2UI surface
→ plain-language nontechnical presentation
→ typed Focusa Operation Registry action
→ canonical Focusa core/API
→ durable replayable event update
```

Specs 135I, 135J, and 135K close the generated-UI, API-integration, and adaptive-usability gaps.

---

## 2. Current code reality

### 2.1 A2UI and AG-UI are not implemented

Current code search finds A2UI/AG-UI only in the new specification documents. There is no runtime package, route, renderer, catalog, or adapter yet.

**Implementation status:** normative target.

### 2.2 OpenAPI generation is not installed

The current Rust workspace uses Serde and Axum, but no Schemars/Utoipa/OpenAPI generation implementation is present in the workspace dependencies.

**Decision:** Alpha 0 begins by establishing generated Rust → OpenAPI/JSON Schema → TypeScript/openapi-fetch contracts before client lanes diverge.

### 2.3 Existing API architecture is the correct foundation

`focusa-api` is already intended as a thin facade:

```text
reads
  snapshot canonical daemon state

writes
  dispatch typed Actions to the daemon event loop
```

Generated UI must preserve this boundary. UI routes must not become an alternate reducer or workflow engine.

### 2.4 Live SSE is low-latency but not durable

The current `/v1/events/stream` implementation:

- subscribes to the in-process broadcast channel;
- emits `focusa_event` JSON;
- sends keepalives;
- silently continues after `RecvError::Lagged`.

This is insufficient for a real-time generated form because a missed event can lose save state, progress, capability, or surface updates.

### 2.5 Canonical event replay already exists

`/v1/events/recent` already provides:

- SQLite canonical event reads;
- bounded limits;
- timestamp cursor;
- `since` and event-type filtering;
- `next_cursor`;
- rehydration metadata.

**Decision:** combine this durable history with the existing broadcast tail. Do not add Redis, Kafka, NATS, a UI event database, or a second AG-UI history for the initial implementation.

### 2.6 Shared error/recovery middleware exists

`middleware/error_envelope.rs` already emits:

- correlation ID;
- status and failure class;
- safe recovery command;
- recovery and misuse hints;
- next tools;
- evidence refs;
- nested `tool_result_v1`.

Generated UI should convert this existing envelope into plain-language recovery cards.

### 2.7 Route-local envelope duplication exists

Multiple route families still define local `*_failure` builders with nearly identical `tool_result_v1` payloads. Examples include events, capabilities, and visual workflow routes.

**Decision:** use expand-contract migration to one shared typed envelope constructor. Do not let new generated-UI routes add another copy.

### 2.8 Capability and permission systems exist

Current API routes already use permission contexts and scoped capability reads.

**Decision:** create `focusa.ui_capability_snapshot.v1` as a bounded projection over existing systems. Do not implement UI permissions in A2UI catalogs or Svelte stores.

### 2.9 UXP/UFI is canonical but not implemented in current runtime code

Spec 14 already defines authoritative UXP/UFI schemas, dimensions, citations, learning rules, transparency, and SQLite storage. Current code search does not show a corresponding runtime implementation.

**Decision:** implement Spec 14 as the only adaptive generated-UI profile. Do not create `SimpleMode`, `ExpertMode`, an expertise score, or another personalization database.

### 2.10 Existing `visual_workflow` routes are evidence routes

`/v1/visual-workflow/evidence/store` and `/v1/visual-workflow/evidence` currently:

- store visual evidence in ECS;
- index handles in Focusa state;
- use labels to encode run/phase/kind;
- duplicate failure-envelope construction;
- allow project/continuity fallback from ambient session state.

They are not a generated workflow or UI protocol.

**Migration decision:**

```text
legacy visual-workflow evidence request
→ typed Workspace Artifact / Evidence capture operation
→ explicit project/workstream/attachment scope
→ ECS handle
→ Evidence link
→ Receipt/event where required
```

Preserve old routes as compatibility aliases during expand-contract migration. Replace label-parsed metadata with typed artifact metadata. Canonical writes must not silently adopt ambient session scope when explicit scope is required.

---

## 3. Highest-leverage implementation accelerators now locked

### 3.1 A2UI instead of a custom generated-UI DSL

Reuse:

- protocol schemas;
- message processor;
- SurfaceModel and data model;
- validation;
- binding;
- incremental updates;
- multi-surface lifecycle;
- action routing;
- basic component catalog;
- Composer/Theater fixtures.

### 3.2 AG-UI middleware instead of a second agent stream

Translate existing Focusa operation and event activity into:

- lifecycle events;
- activity snapshots;
- tool events;
- state snapshots;
- RFC 6902 state deltas;
- custom A2UI messages.

### 3.3 Generated Operation Registry instead of a manual UI action list

Generate operation metadata, action schemas, capability requirements, confirmation posture, and UI action bindings from Rust/OpenAPI.

### 3.4 JSON Schema inputs instead of custom forms

Use ordinary A2UI inputs for standard scalar, enum, date, array, and file-reference fields. Build custom Focusa components only for real domain interactions.

### 3.5 Deterministic surfaces without model calls

Render immediately without an LLM:

- stage shell;
- progress;
- required fields;
- validation;
- permissions/capabilities;
- approval state;
- recovery;
- standard forms;
- known source and task summaries.

Model calls remain for cognition and synthesis, not generic UI mechanics.

### 3.6 Existing UXP/UFI instead of a new nontechnical mode

Use the safe nontechnical default and canonical UXP dimensions for explanation and pacing. Capture only cited observable UFI friction.

### 3.7 Fixtures before live integration

Build A2UI catalogs and stage surfaces against generated schemas and deterministic fixtures while backend routes are being implemented. Replace fixtures with live calls without changing component contracts.

### 3.8 Existing UIAI Test Lab and evidence plane

Use UIAI Engine for:

- screenshots;
- cross-browser visual proof;
- responsive proof;
- browser-context testing;
- generated UI evidence;
- recovery-path verification.

Do not create another browser test/evidence subsystem.

---

## 4. Correct core integration path

```text
Focusa canonical reducer/state
→ subsystem read model
→ Resolved Project Operating Profile
→ UiInteractionIntent
→ Generated Surface Envelope
→ A2UI messages
→ trusted renderer
→ UI Action Binding
→ Focusa Operation Registry
→ existing preview/commit/core action
→ shared ToolResult envelope
→ canonical event / Evidence / Receipt
→ durable SQLite replay + live broadcast
→ AG-UI translation
→ targeted A2UI delta
```

Every layer has one responsibility. No layer may duplicate the authority of the layer beneath it.

---

## 5. Revised fastest Alpha 0 implementation order

Implement in this exact order:

1. Add Schemars/Utoipa and generated OpenAPI/JSON Schema.
2. Generate TypeScript and `openapi-fetch` client.
3. Define Operation Registry annotations and snapshot.
4. Centralize shared ToolResult/error constructors through expand-contract.
5. Add stable event ID/sequence and replayable SSE using SQLite + broadcast.
6. Add capability/permission snapshot.
7. Add `UiInteractionIntent` and one Generated Surface Envelope.
8. Integrate `@a2ui/web_core/v0_9` and maintained Lit renderer.
9. Add AG-UI middleware translation.
10. Bind one Context action through preview/commit.
11. Run Schemathesis, A2UI fixture, Playwright, scope, replay, and recovery proof.

After step 3, UI fixture/catalog work and core read-model work can proceed in parallel. After step 5, every stage can use the same durable stream.

---

## 6. Stage implementation order

After Alpha 0:

```text
Context generated surface
→ Role generated surface
→ Grill Interview generated surface
→ Spec progress/approval generated surface
→ Task-plan generated surface
→ Workpoint/Evidence/Receipt generated continuation
```

This is the shortest path to a full, usable C.R.I.S.T. traversal because each stage reuses the same shell, Operation Registry, action binding, capability snapshot, error mapping, stream, and catalog.

---

## 7. Nontechnical completion standard

Backend completion does not satisfy a C.R.I.S.T. stage.

A stage is complete only when a nontechnical operator can:

- understand what is happening;
- know why Focusa needs input;
- see what Focusa already knows;
- receive a recommendation with sources;
- provide or approve the input;
- see save/progress state;
- recover from a realistic error;
- leave and resume;
- reach the next stage without CLI, raw JSON, route names, schemas, or developer intervention.

---

## 8. Decomposition blockers

Reject decomposition that:

- builds every backend stage before one end-to-end generated path;
- adds static stage pages before the shared surface/action/stream spine;
- creates route-specific frontend calls instead of generated operations;
- adds a second event store;
- copies permission or error logic into the client;
- treats `visual_workflow` routes as a generated UI system;
- creates a second personalization profile;
- makes deterministic UI wait for a model;
- considers CLI proof sufficient for onboarding completion.

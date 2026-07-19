# Spec 135 Real-Time Generated UI Speed and Core Integration Audit

**Audit date:** 2026-07-18  
**Scope:** current Focusa code and Spec 135A–135K  
**Status:** implementation reality and mandatory migration guidance  
**Precedence:** [Spec 135 Series Current Authoritative Delivery Contract](../135-series-current-manifest.md)

---

## 1. Verdict

The Spec 135 series now requires:

```text
every onboarding and C.R.I.S.T. stage
→ real-time generated A2UI surface
→ safe nontechnical presentation
→ generated Focusa Operation Registry action
→ canonical Focusa core/API
→ durable replayable event update
→ UIAI Engine Eval proof when browser-facing
```

The implementation remains incomplete. The current repository contains reusable runtime foundations, but A2UI, generated Operation Registry, replayable SSE, C.R.I.S.T. runtime state, UXP/UFI runtime, generated clients, and UIAI Eval integration remain implementation work.

---

## 2. Current code reality

### 2.1 A2UI and AG-UI are not implemented

A2UI and AG-UI currently exist in specifications only. There is no runtime package, renderer integration, catalog, route family, or compatibility adapter.

**Required implementation:** A2UI web core plus the permanent Lit renderer first. AG-UI proceeds later as an external compatibility adapter and does not block native Alpha traversal.

### 2.2 Generated contracts are not installed

The current Rust workspace uses Serde and Axum but does not yet provide the selected Schemars/Utoipa generation chain.

**Required implementation:**

```text
Rust types
→ JSON Schema 2020-12
→ OpenAPI 3.0.3
→ TypeScript openapi-typescript/openapi-fetch
→ Go oapi-codegen client/models for UIAI Engine
→ Operation Registry and A2UI action bindings
```

### 2.3 Existing API architecture is the correct foundation

`focusa-api` is already intended as a thin facade:

```text
reads
  snapshot canonical daemon state

writes
  dispatch typed Actions to the daemon event loop
```

Generated UI MUST preserve this boundary. UI routes cannot become another reducer, workflow engine, or authority layer.

### 2.4 Live SSE is low-latency but not durable

Current `/v1/events/stream` subscribes to the in-process broadcast channel, emits event JSON, sends keepalives, and can continue after a lagged receiver.

A missed event can lose apparent save state, progress, capability, or surface updates.

### 2.5 Canonical SQLite replay already exists

`/v1/events/recent` already supplies bounded SQLite event reads, filters, cursors, `next_cursor`, and rehydration metadata.

**Required migration:**

```text
client cursor / Last-Event-ID
→ replay missed scoped events from SQLite
→ subscribe to broadcast tail
→ deduplicate by stable event ID and sequence
→ emit A2UI snapshot/delta
```

Do not add Redis, Kafka, NATS, a UI event database, or a second AG-UI history.

### 2.6 Shared ToolResult/error middleware exists

The existing middleware already provides correlation IDs, status, failure classes, recovery and misuse hints, retry posture, next tools, side effects, and Evidence references.

Generated recovery surfaces MUST project this envelope into plain language. Do not create a UI-specific error or retry taxonomy.

### 2.7 Route-local envelope duplication exists

Several route families still define local failure builders.

**Required migration:** expand-contract to one shared typed envelope constructor before new generated-UI routes expand the duplication.

### 2.8 Capability and permission systems exist

Current routes already use permission contexts and scoped capability reads.

**Required implementation:** `focusa.ui_capability_snapshot.v1` is a bounded projection over existing capabilities, permissions, provider health, connector health, and client capability. Do not implement permissions in A2UI catalogs or Svelte stores.

### 2.9 UXP/UFI is specified but not implemented

Spec 14 is the canonical user-experience model. Current runtime code does not yet implement the full UXP/UFI lifecycle.

**Required implementation:** implement Spec 14 as the only adaptive generated-UI profile. Do not create Simple Mode, Expert Mode, expertise scoring, emotion labels, or another personalization database.

### 2.10 Existing `visual_workflow` routes are Evidence routes

The existing visual-workflow routes store ECS evidence and handles. They are not a generated workflow engine.

**Required migration:**

```text
legacy visual-workflow evidence request
→ typed Workspace Artifact / Evidence operation
→ explicit project/workstream/attachment scope
→ ECS handle
→ Evidence link
→ event and Receipt where required
```

Preserve old routes as compatibility aliases during expand-contract. Remove ambient session-scope fallback from authority-bearing writes.

### 2.11 Browser execution and proof already belong to UIAI Engine

UIAI Engine already owns browser sessions, actions, contexts, screenshots, snapshots, diagnostics, FPV, responsive capture, visual comparison, and browser evidence.

**Required implementation:** add versioned UIAI Engine Eval scenario/result contracts and use UIAI Engine Eval for all browser, end-to-end, responsive, visual, reconnect, diagnostic, isolation, and browser-accessibility proof.

Focusa MUST NOT add Playwright or a second browser test runtime.

### 2.12 Model execution already belongs to governed harness sessions

Focusa already has Pi integration and Spec 133 governed session architecture.

**Required implementation:** add `focusa.agent_execution_adapter.v1` with Pi RPC as the reference adapter for Role Composer, Grill Interview, grounded recommendations, synthesis, and generated explanations.

Do not introduce Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, Vercel AI Gateway authority, or another model/tool runtime.

---

## 3. Locked accelerators

### A2UI instead of a custom generated-UI system

Reuse A2UI protocol schemas, `web_core`, SurfaceModel, validation, data binding, incremental updates, action routing, multi-surface lifecycle, basic catalog, Composer, and Theater.

Use the maintained Lit renderer permanently. Author Focusa-specific Svelte controls as Custom Elements. Do not build another complete A2UI renderer.

### Generated Operation Registry

Generate operations, schemas, capabilities, confirmation posture, recovery metadata, and UI action bindings from Rust/OpenAPI.

### Schema-driven ordinary inputs

Use A2UI basic inputs for scalar, enum, date, array, and file-reference fields. Build custom Focusa controls only for domain interactions.

### Deterministic UI without model calls

Render stage shell, progress, required fields, validation, capabilities, approvals, recovery, standard forms, and known summaries without a model call.

### UIAI Engine Eval instead of browser test reinvention

Use UIAI Engine Eval for browser actions, screenshots, responsive states, diagnostics, accessibility snapshots, visual comparison, reconnect, authentication, browser-context isolation, and Evidence generation.

### Existing UXP/UFI

Use the nontechnical baseline and canonical UXP dimensions. Capture only cited observable UFI friction.

### Deterministic fixtures before live integration

Build A2UI catalogs and stage surfaces against generated schemas and fixtures while backend routes are implemented. Replace fixture data with live calls without changing contracts.

---

## 4. Correct core integration path

```text
Focusa canonical primitive and reducer
→ subsystem read model
→ Resolved Project Operating Profile
→ UiInteractionIntent
→ Generated Surface Envelope
→ A2UI messages
→ permanent Lit renderer + Focusa Svelte Custom Elements
→ UI Action Binding
→ generated Operation Registry
→ preview/commit Focusa operation
→ shared ToolResult envelope
→ canonical event / Evidence / Receipt
→ SQLite replay + broadcast live tail
→ targeted A2UI delta
```

AG-UI translates this path for external compatibility. It does not sit between native Focusa events and the first complete product traversal.

---

## 5. Exact fastest Foundation Train

1. Freeze the series at 135K and compile the Delivery Contract.
2. Create the machine-readable feature ledger, DAG, parity, framework, and proof matrices.
3. Add Schemars/Utoipa and generate JSON Schema 2020-12 plus OpenAPI 3.0.3.
4. Generate TypeScript/openapi-fetch and Go/oapi-codegen clients.
5. Generate Operation Registry and UI action bindings.
6. Centralize ToolResult/error constructors through expand-contract.
7. Add stable event IDs and replayable SSE over SQLite plus broadcast.
8. Add capability/permission projection and version handshake.
9. Add Pi RPC AgentExecutionAdapter.
10. Add UiInteractionIntent and Generated Surface Envelope.
11. Integrate A2UI web core and permanent Lit renderer.
12. Register initial Focusa Svelte Custom Elements.
13. Implement first UIAI Engine Eval scenario.
14. Bind one real Context action through preview/commit.
15. Run contract, fixture, scope, replay, recovery, generated-UI, and UIAI Eval proof.

After generated operation contracts stabilize, client, core, Context, Interview, Spec/Task, UIAI, vertical, provider, and hardening lanes proceed in parallel.

AG-UI compatibility proceeds after the native durable stream is stable and does not block the native Alpha.

---

## 6. Stage order

```text
Context generated surface
→ Role generated surface
→ Grill Interview generated surface
→ Spec progress and approval surface
→ Task-plan surface
→ Workpoint / Evidence / Receipt continuation
```

Every stage reuses the same StageShell, Operation Registry, action binding, capability snapshot, ToolResult mapping, durable stream, A2UI catalog, UXP/UFI projection, and UIAI Eval contract.

---

## 7. Greater primitive submission

Implementation order:

```text
general Focusa primitive
→ reducer/state
→ typed API
→ generated TypeScript/Go contracts
→ C.R.I.S.T. projection
→ renderer
→ UIAI Engine Eval
→ Evidence
→ Receipt
```

Reject a PR that implements generally reusable behavior only inside Project Genesis, a route-local UI module, a client store, or an A2UI component.

---

## 8. Nontechnical completion standard

A stage is complete only when a nontechnical operator can:

- understand what is happening;
- understand why input is required;
- see what Focusa already knows;
- receive a recommendation and sources;
- provide or approve input;
- see save and progress state;
- recover from a realistic failure;
- leave and resume exact state;
- continue without CLI, raw JSON, route names, schemas, or developer intervention.

---

## 9. Decomposition blockers

Reject decomposition that:

- builds every backend stage before one complete generated path;
- creates static stage pages before the shared surface/action/stream spine;
- adds route-specific client DTOs or action catalogs;
- adds a second event history;
- copies permission, ToolResult, or retry logic into clients;
- treats visual-workflow evidence routes as generated UI;
- creates a second personalization profile;
- waits for a model to render deterministic UI;
- uses CLI proof for onboarding completion;
- uses browser proof outside UIAI Engine Eval;
- introduces Playwright;
- builds a complete second A2UI renderer;
- makes AG-UI a native Alpha blocker;
- introduces Vercel AI SDK as Focusa runtime authority;
- fails to submit reusable behavior to a greater Focusa primitive;
- omits requirement IDs, Evidence, Receipts, migration, recovery, or UIAI Eval scenarios from the machine-readable delivery graph.

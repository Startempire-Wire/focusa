# Spec 109 — Agent-First API Redesign / AX Compliance

Status: Draft  
Owner: Verious Smith  
Created: 2026-06-25  
Scope: Focusa daemon HTTP API, CLI/Pi tool parity, public stream/card surfaces, menubar/API clients, and UIAI bridge intake surfaces.

## 1. Purpose

Focusa already has the core premise of an agent-first system: durable mission state, typed Workpoints, evidence handles, trajectory, metacognition, bounded reads, route-scoped permissions, and recovery-first envelopes.

This spec makes that premise explicit as an **Agent Experience (AX) API contract**.

The goal is not to make Focusa APIs smaller, prettier, or more human-minimal. The goal is to make them **more directly useful to agents** while remaining safe:

```text
compressed discoverability + constrained power + typed contracts + bounded side effects + recoverable failures
```

## 2. Source Evidence From Current Repo

This spec is grounded in the current repo surfaces:

- `README.md`
  - Focusa is a local-first mission cohesion layer for AI coding agents.
  - Runtime includes Rust daemon, HTTP API, CLI, TUI, Pi extension, and menubar proof surfaces.
  - Focusa exposes Workpoint, Evidence, Trajectory, Context Authority, public stream redaction, and tool envelopes.
- `docs/current/generated/tool-surface-summary.md`
  - Current tool surface: 97 tool contracts, 11 families, 93 API-parity tools, 81 CLI-parity tools, 96 Pi tools, 97 docs-covered tools.
- `docs/current/focusa-tool-contracts.json`
  - Existing tool contracts include name, purpose, family, ontology action, API routes, CLI commands, side-effect profile, parity status, and live checks.
- `docs/23-capabilities-api.md`
  - Capabilities API already states: everything observable, authority centralized, writes validated/audited, local-first, performance-safe, policy-enforced.
- `docs/25-capability-permissions.md`
  - Current permission model separates read, command, and admin permissions; states that permissions grant access, policy grants authority.
- `docs/current/API_REFERENCE_CURRENT.md`
  - Current API reference is generated from route registrations and explicitly says it is an inventory, not a full schema reference.
- `crates/focusa-api/src/server.rs`
  - Router merges many API domains and applies CORS, body limit, JSON guard, mutation rate limit, route scope, auth, and error envelope middleware.
- `crates/focusa-api/src/middleware/error_envelope.rs`
  - Error envelope already includes correlation ID, failure class, recovery hint, misuse hint, next tools, retry posture, and tool_result_v1 details for non-JSON failures.
- `crates/focusa-api/src/middleware/route_scope.rs`
  - Route-scope middleware maps routes to scopes and enforces token permissions when auth is configured.
- `crates/focusa-api/src/middleware/json_guard.rs`
  - Mutation payloads are bounded by JSON depth, array size, object field count, body size, and path traversal guardrails.
- `crates/focusa-api/src/middleware/rate_limit.rs`
  - Mutation requests are route/caller bounded through fixed-window rate limiting.
- `crates/focusa-api/src/routes/commands.rs`
  - Commands are allowlisted, validated, logged, idempotency-key aware, status-queryable, and mapped into daemon actions.
- `crates/focusa-api/src/routes/workpoint.rs`
  - Workpoint routes already include preview/dry_run aliasing, idempotency cache, project-root authority checks, scope mismatch taxonomy, drift checks, mutation previews, recovery hints, and bounded packets.
- `docs/105-agent-dx-ux-merged-scope-spec.md`
  - Existing DX/UX spec already names important gaps: ambiguous materialization, mixed mutation models, durability contract gaps, recovery burden, and response-layout ambiguity.
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`
  - New behavior must move through Idea → New Spec → task decomposition → implementation → tests/proofs → closure.

## 3. AX Rubric Baseline Score

Current Focusa API posture is strong but uneven.

| V4 AX Dimension | Current Score | Current Posture | Required Change |
|---|---:|---|---|
| Capability-first design | 4.5 / 5 | Many powerful primitives exist across Workpoint, Trajectory, Context Cognition, Metacog, Work-loop, Ontology, Traversal, Events, and Commands. | Make canonical operation metadata discoverable from one contract surface. |
| Minimize harmful redundancy | 2.5 / 5 | Several overlapping route families and aliases exist: state/focus, clt/lineage, autonomy/autonomy-status, events/events_sqlite/events_stream, commands plus direct write routes. | Create canonical route map + deprecation/alias metadata. |
| Strict schemas | 2.5 / 5 | Rust structs validate many payloads, but external machine contract is not fully OpenAPI/JSON Schema. | Generate authoritative OpenAPI/JSON Schema from route/tool registry. |
| Preview/commit safety | 3 / 5 | Workpoint preview exists; commands and other write families are not uniformly mode-based. | Require `side_effect_policy=dry_run|preview|commit` semantics on all risky writes. |
| Discoverability | 3.5 / 5 | Tool contract registry and route inventory exist, but capability endpoint returns effective permissions, not full operation metadata. | Add `/v1/agent/capabilities` operation metadata endpoint. |
| Authorization matrix | 3.5 / 5 | Route-scope middleware and permission docs exist; field-level/object-level policies are not consistently discoverable. | Generate operation-level auth matrix from registry. |
| Observability/recovery | 4 / 5 | Error envelopes and tool_result_v1 are strong, but success envelopes are inconsistent. | Standardize compact/standard/debug envelopes for all routes. |
| Idempotency/retries/concurrency | 2.5 / 5 | Workpoint has cache; commands accept idempotency key but do not enforce a global dedupe ledger. | Add global idempotency ledger + state/version conflict checks. |
| Cost/resource budgeting | 3 / 5 | Body, JSON, route, lineage, bloatguard, and resource-mode limits exist, but no uniform request-level budget object. | Add per-operation `budget` schema and enforcement. |
| Trust boundaries | 3.5 / 5 | JSON/path guards and authority checks exist; natural-language slots are not uniformly labeled untrusted. | Add NL trust-boundary metadata to schemas. |
| Low-level/query hardening | 3 / 5 | Traversal and bounded reads exist; proxy/ontology/traverse/sync/debug surfaces need explicit complexity/scope metadata. | Add query-power hardening profile per endpoint. |
| Versioning/contract stability | 2.5 / 5 | `/v1` exists and docs are generated, but operation/schema versions are not in every operation contract. | Add `operation_version`, `schema_version`, deprecation metadata. |

Target after Spec109 implementation: every AX dimension >= 4.5 / 5.

## 4. Problem Statement

Focusa has the right primitives, but the surfaces are not yet standardized as a production-grade AX API.

The main gap is not missing power. The main gap is **contract uniformity**.

Agents should not have to infer:

- which route is canonical vs legacy/alias,
- whether a write is direct, daemon-dispatched, advisory, delayed, or materialized,
- which schema version governs a payload,
- whether `accepted` means persisted, dispatched, queued, previewed, or merely validated,
- whether an operation supports dry-run/preview/commit,
- whether an `idempotency_key` is advisory or enforced,
- which scopes, project-root authority, field permissions, budgets, and confirmation requirements apply,
- whether a failure should be retried, revised, permission-escalated, context-refetched, or stopped.

## 5. Non-Goals

This spec does not require:

- removing Focusa's existing cognitive vocabulary,
- hiding powerful primitives behind human-friendly wrappers,
- replacing existing Workpoint/Trajectory/Metacog/Context Cognition architecture,
- deleting legacy routes immediately,
- changing local-first default deployment philosophy,
- making cloud/team/multi-user sync part of Operator Preview.

This spec does require making the current surfaces more machine-contractual, bounded, and discoverable.

## 6. Core AX Principle

Focusa's API should expose **the most capable safe primitive** for each outcome.

The canonical route for an outcome should be powerful enough for a strong agent to complete meaningful work without artificial human-ramp-up layers, but every such route must be:

- typed,
- bounded,
- scoped,
- permissioned,
- discoverable,
- versioned,
- idempotent where mutating,
- observable,
- recoverable,
- and safe under retry or partial failure.

## 7. Canonical AX Surfaces To Add

### AX-001 — Authoritative Agent Capabilities Endpoint

Add:

```http
GET /v1/agent/capabilities
```

This endpoint returns a compact machine-readable index of operations.

It is not the same as `GET /v1/agents/{agent_id}/capabilities`, which currently reports effective permissions for an agent/token. The new endpoint reports **operation metadata**.

Required compact response:

```json
{
  "schema": "focusa.agent_capabilities.index.v1",
  "api_version": "v1",
  "generated_at": "iso8601",
  "operation_count": 0,
  "families": [],
  "operations": [
    {
      "operation_id": "focusa.workpoint.checkpoint",
      "label": "Workpoint Checkpoint",
      "family": "workpoint",
      "method": "POST",
      "path": "/v1/workpoint/checkpoint",
      "canonical": true,
      "alias_of": null,
      "operation_version": "1.0.0",
      "schema_version": "focusa.workpoint_checkpoint.request.v1",
      "side_effect_profile": "write_workpoint_checkpoint",
      "materialization_mode": "direct_serialized_writer|daemon_dispatch|advisory_read|eventual_dispatch",
      "supports_side_effect_policy": ["dry_run", "preview", "commit"],
      "requires_idempotency_key": true,
      "requires_if_match_version": true,
      "requires_preview_token": false,
      "permissions_required": ["workpoint:write"],
      "confirmation_required": false,
      "budget_profile": "standard_mutation",
      "response_detail_supported": ["compact", "standard", "debug"],
      "request_schema_ref": "/v1/agent/schemas/focusa.workpoint_checkpoint.request.v1",
      "response_schema_ref": "/v1/agent/schemas/focusa.workpoint_checkpoint.response.v1",
      "error_taxonomy_ref": "/v1/agent/error-taxonomy",
      "examples_ref": "/v1/agent/examples/focusa.workpoint.checkpoint",
      "docs_ref": "docs/focusa-tools/tools/focusa_workpoint_checkpoint.md",
      "deprecation": null
    }
  ]
}
```

### AX-002 — Schema Index

Add:

```http
GET /v1/agent/schemas
GET /v1/agent/schemas/{schema_id}
```

Schemas must be JSON Schema compatible and align with generated OpenAPI.

Every schema must define:

- required fields,
- optional fields,
- enums,
- defaults,
- max sizes,
- context refs,
- natural-language fields labeled as untrusted,
- budget support,
- response detail support,
- examples,
- error cases.

### AX-003 — OpenAPI Export

Add or generate:

```http
GET /v1/openapi.json
```

Also add repo artifact:

```text
docs/current/generated/openapi.json
```

The current `docs/current/API_REFERENCE_CURRENT.md` remains useful as a route inventory, but it is not enough for agent execution because it lacks full request/response schemas.

### AX-004 — llms.txt Context Index

Add:

```text
llms.txt
```

Purpose: concise LLM reading order for Focusa docs.

It must not be treated as an authoritative machine contract. It should point agents to:

1. README,
2. current API reference,
3. generated OpenAPI,
4. capabilities endpoint,
5. tool surface summary,
6. core glossary,
7. golden workflow,
8. Workpoint docs,
9. Context Authority docs,
10. troubleshooting/error docs.

## 8. Standard Request Envelope

All non-read operations must accept the AX request envelope by Spec109 completion.

```json
{
  "intent": "clear outcome description",
  "inputs": {},
  "context_refs": [],
  "constraints": {},
  "side_effect_policy": "dry_run|preview|commit",
  "idempotency_key": "client-generated-uuid",
  "if_match_version": "state-or-resource-version",
  "preview_token": "required-for-sensitive-commit-if-issued",
  "budget": {
    "max_runtime_ms": 30000,
    "max_records": 500,
    "max_cost_usd": 0.25,
    "max_external_calls": 10
  },
  "response_detail": "compact|standard|debug"
}
```

Compatibility rule: existing route-specific bodies may be accepted during migration, but canonical docs and examples must prefer the AX envelope or an explicit route-specific schema that maps losslessly into it.

## 9. Standard Response Envelope

All routes must return a compatible AX response envelope.

### 9.1 Compact response

```json
{
  "request_id": "req_...",
  "operation_id": "focusa.workpoint.checkpoint",
  "operation_version": "1.0.0",
  "schema_version": "focusa.workpoint_checkpoint.response.v1",
  "status": "success|partial|failed|blocked|pending",
  "materialization_status": "none|validated|previewed|accepted_pending|dispatched|materialized_canonical",
  "result": {},
  "warnings": [],
  "recoverable_errors": [],
  "next_recommended_call": {}
}
```

### 9.2 Standard response

Adds:

```json
{
  "request_id": "req_...",
  "operation_id": "...",
  "status": "...",
  "authority": {
    "project_root": "...",
    "continuity_id": "...",
    "scope_status": "verified|unverified|mismatch|unsafe"
  },
  "side_effects": [],
  "evidence_refs": [],
  "audit_event": {},
  "retry": {
    "safe": true,
    "posture": "safe_retry|do_not_retry_unchanged|operator_required|stop",
    "reason": "..."
  },
  "next_tools": []
}
```

### 9.3 Debug response

Adds:

```json
{
  "actions_planned": [],
  "actions_taken": [],
  "omitted": [],
  "bounds": {},
  "route_scope": {},
  "permission_checks": [],
  "query_plan": {},
  "diagnostics": {},
  "docs": [],
  "compatibility": {}
}
```

## 10. Standard Error Taxonomy

Every failure must use a machine-actionable error type.

```json
{
  "error": {
    "type": "validation_error|authorization_denied|scope_mismatch|conflict|budget_exceeded|unsafe_request|dependency_failure|partial_success|requires_confirmation|schema_version_unsupported|resource_exhausted|not_found|daemon_unavailable|persistence_failed|reducer_rejected|deprecation_blocked",
    "message": "Human-readable summary",
    "field_errors": [],
    "recovery_hints": [],
    "next_recommended_call": {},
    "retry_posture": "safe_retry|do_not_retry_unchanged|operator_required|stop"
  }
}
```

Existing `failure_class`, `recovery_hint`, `misuse_hint`, `next_tools`, `retry`, and `tool_result_v1` fields should be mapped into this taxonomy instead of removed abruptly.

## 11. Materialization Contract

Every mutating operation must declare exactly one materialization mode:

| Mode | Meaning |
|---|---|
| `validated_only` | Request validated; no state changed. |
| `previewed` | Plan/diff/cost/risk returned; no state changed. |
| `accepted_pending` | Accepted into a queue/store but not applied yet. |
| `daemon_dispatch` | Dispatched to daemon reducer/event loop; result may require status polling. |
| `direct_serialized_writer` | Route owns serialized state mutation and persistence path. |
| `materialized_canonical` | State was applied and persisted; response is read-after-write consistent. |

The terms `accepted`, `dispatched`, `completed`, `ok`, and `success` must not be used without a `materialization_status` field.

## 12. Preview / Commit Semantics

For destructive, expensive, public, external, sync, export, proxy, work-loop, token, device-pairing, restore, and irreversible operations:

- `dry_run` validates inputs, permissions, budgets, and likely effects; no durable state changes.
- `preview` returns plan, diff, affected resources, risk, cost, confirmation requirement, and optional `preview_token`; no durable state changes.
- `commit` executes; requires `idempotency_key` and may require `preview_token`.

Preview response shape:

```json
{
  "status": "success",
  "materialization_status": "previewed",
  "result": {
    "preview_token": "prev_...",
    "would_create": [],
    "would_update": [],
    "would_delete": [],
    "would_emit": [],
    "would_call_external": [],
    "authority_scope": {},
    "budget_estimate": {},
    "risk": "low|medium|high|blocked",
    "safe_to_apply": true,
    "requires_confirmation": false
  }
}
```

## 13. Idempotency, Retries, and Concurrency

### AX-005 — Global Idempotency Ledger

Add a shared idempotency ledger for all non-read operations.

Required fields:

```json
{
  "idempotency_key": "...",
  "operation_id": "...",
  "request_hash": "sha256:...",
  "resource_scope": {
    "project_root": "...",
    "continuity_id": "...",
    "work_item_id": "..."
  },
  "first_seen_at": "iso8601",
  "last_seen_at": "iso8601",
  "status": "in_progress|completed|failed|expired",
  "result_ref": "..."
}
```

Rules:

- Same key + same operation + same request hash returns prior result.
- Same key + different request hash returns `conflict`.
- Idempotency must survive daemon restart for production-hardening routes.
- In-memory caches are allowed only for preview/operator-preview routes and must be marked as such.

### AX-006 — Version Checks

Mutating operations must accept one of:

- `if_match_version`,
- `state_revision`,
- `resource_version`,
- `workpoint_updated_at`,
- or explicit `no_version_required` operation metadata.

Conflict response:

```json
{
  "error": {
    "type": "conflict",
    "message": "Resource changed since caller read it",
    "recovery_hints": ["Fetch current resource, merge, retry with new if_match_version"],
    "next_recommended_call": {
      "method": "GET",
      "path": "/v1/workpoint/current"
    }
  }
}
```

## 14. Budgeting and Resource Controls

Current body/JSON/rate/lineage/resource-mode controls are good foundations. Spec109 standardizes them into operation budgets.

Every operation metadata record must define:

```json
{
  "budget_profile": "read_small|read_bounded|mutation_standard|mutation_sensitive|external_proxy|export_long_running",
  "server_limits": {
    "max_runtime_ms": 0,
    "max_records": 0,
    "max_body_bytes": 0,
    "max_json_depth": 0,
    "max_json_array_items": 0,
    "max_json_object_fields": 0,
    "max_external_calls": 0,
    "max_cost_usd": 0
  },
  "client_budget_supported": true
}
```

If a client budget exceeds server limits, server limits win. If the request would exceed budget, return `budget_exceeded` with a lower-cost next call.

## 15. Canonical Route Standardization

### AX-007 — Canonical Route Map

Create:

```text
docs/current/generated/agent-canonical-route-map.json
```

Each route must be classified as:

- `canonical`,
- `alias`,
- `legacy`,
- `deprecated`,
- `internal`,
- `debug_only`,
- `public_surface`,
- `dangerous_sensitive`.

Examples requiring review:

| Current overlap | Required decision |
|---|---|
| `/v1/state/current` vs `/v1/focus/*` | Pick canonical read/write path and alias/deprecate the other. |
| `/v1/lineage/*` vs `/v1/clt/*` | Pick canonical lineage/CLT vocabulary or expose explicit alias metadata. |
| `/v1/autonomy` vs `/v1/autonomy/status` | Normalize canonical route and compatibility path. |
| `/v1/events/recent`, `events_sqlite`, `events_stream`, `sse` | Clarify canonical event stream/readback surfaces. |
| `/v1/commands/submit` vs direct mutation routes | Declare canonical write model per route family. |
| `/proxy/*` vs `/v1/proxy/*` | Declare proxy threat model and canonical path. |

### AX-008 — Deprecation Metadata

Every non-canonical route must expose:

```json
{
  "canonical": false,
  "alias_of": "operation_id",
  "deprecation": {
    "status": "active_alias|deprecated|removed_pending_major",
    "replacement_operation_id": "...",
    "replacement_path": "...",
    "remove_no_earlier_than": "v2"
  }
}
```

## 16. Operation Metadata Registry

Extend `docs/current/focusa-tool-contracts.json` or create a sibling generated file:

```text
docs/current/generated/agent-operation-contracts.json
```

Required operation fields:

```json
{
  "operation_id": "focusa.workpoint.resume",
  "operation_version": "1.0.0",
  "schema_version": "focusa.workpoint_resume.request.v1",
  "family": "workpoint",
  "purpose": "Resume canonical continuation packet for verified project scope",
  "method": "POST",
  "path": "/v1/workpoint/resume",
  "canonical": true,
  "route_class": "agent_canonical",
  "tool_names": ["focusa_workpoint_resume"],
  "cli_commands": ["focusa workpoint resume"],
  "side_effect_profile": "read_or_checkpoint_resume_semantics",
  "materialization_mode": "advisory_read",
  "permissions_required": ["workpoint:read"],
  "object_authorization": {
    "project_root_required": true,
    "continuity_id_required": true,
    "tenant_boundary": "local_project_root"
  },
  "field_permissions": {
    "read": ["*"],
    "write": []
  },
  "budget_profile": "read_bounded",
  "request_schema_ref": "...",
  "response_schema_ref": "...",
  "examples_ref": "...",
  "error_taxonomy_ref": "...",
  "live_check": "...",
  "docs_ref": "...",
  "changelog_ref": "..."
}
```

## 17. Auth Matrix Standard

The current route-scope middleware is useful, but it must become generated/validated from the operation registry instead of hand-maintained separately.

Every operation must declare:

- required scopes,
- route-scope middleware mapping,
- tenant/local-project boundary,
- object-level authorization rule,
- field-level read policy,
- field-level write policy,
- confirmation requirement,
- audit event emitted,
- whether local no-token mode may access it,
- whether non-loopback requires token.

Spec109 hard rule:

```text
If operation metadata and route_scope middleware disagree, CI fails.
```

## 18. Low-Level / Query / Proxy Hardening

Any route exposing traversal, ontology, proxy, sync, export, debug pressure controls, ECS rehydration, or external model/provider calls must declare a `query_power_profile`:

```json
{
  "query_power_profile": {
    "raw_eval_allowed": false,
    "read_only_by_default": true,
    "tenant_bound": true,
    "project_root_bound": true,
    "allowlisted_fields": [],
    "max_complexity": 0,
    "supports_explain": true,
    "supports_dry_run": true,
    "redaction_profile": "private|public_safe|debug_sensitive"
  }
}
```

Rules:

- Avoid raw unrestricted eval.
- Avoid unbounded full-tree traversal by default.
- Cold/full payload reads must be explicit opt-in and reflected in response `bounds`.
- Proxy routes must surface external-call budget, provider/model target, redaction profile, and whether prompt/context contains untrusted content.

## 19. Natural-Language Trust Boundaries

Any schema field that accepts natural language must mark it as untrusted.

Examples:

- `intent`,
- `reason`,
- `justification`,
- `current_ask`,
- `mission`,
- `summary`,
- `draft`,
- `latest_action`,
- `payload.content`.

Required schema annotation:

```json
{
  "x-focusa-trust": {
    "input_kind": "natural_language",
    "trusted": false,
    "may_override_policy": false,
    "prompt_injection_risk": "low|medium|high",
    "server_policy_precedence": true
  }
}
```

## 20. Public Stream / Arena / UIAI Bridge Compliance

Public and bridge-facing surfaces must be AX-compliant because they are where Focusa becomes credible externally.

### Public stream/card rules

Any public card emitted through `FOCUSA_PUBLIC_STREAM=1`, Arena, or public proof surfaces must include:

- schema version,
- operation/source tool,
- redaction status,
- evidence refs safe for public display,
- omitted/private fields count,
- proof confidence,
- replay/refetch path if public-safe,
- no secrets, tokens, private project paths, raw prompts, private transcript segments, or unredacted PII.

### UIAI bridge intake rules

UIAI diagnostics/evidence intake must declare:

- source session identity,
- screenshot/log/proof artifact type,
- redaction status,
- browser/page authority scope,
- evidence hash/ref,
- whether evidence is actual/partial/surrogate/blocked/missing,
- linked Workpoint or trajectory target,
- follow-up operation.

## 21. AX Evaluation and Simulation

Add an AX simulation test suite.

Repo target:

```text
tests/spec109_ax_api_contract_test.py
```

Required eval scenarios:

1. Fresh agent discovers capabilities and chooses correct Workpoint resume route.
2. Agent attempts write without scope; receives deterministic blocked response and exact next call.
3. Agent dry-runs Workpoint checkpoint, then commits with idempotency key.
4. Agent retries same idempotency key and receives prior result, not duplicate mutation.
5. Agent sends same idempotency key with different body and receives conflict.
6. Agent requests full lineage/tree without explicit budget and receives bounded response with next cursor.
7. Agent calls deprecated/alias route and receives canonical replacement metadata.
8. Agent attempts proxy/external call without budget and receives budget/confirmation requirement.
9. Agent gets validation error and can automatically produce corrected second call from `field_errors`.
10. Agent attempts completion claim with partial/surrogate evidence and closure is blocked per Spec107.

Metrics:

- first-call success rate,
- second-call recovery success rate,
- tokens needed to discover operation,
- invalid route guesses,
- unsafe retries prevented,
- duplicate side effects prevented,
- budget overruns prevented,
- schema parity failures,
- docs/runtime drift count.

## 22. Acceptance Criteria

Spec109 is accepted when:

1. `docs/109-agent-first-api-redesign-ax-spec.md` exists.
2. A task decomposition exists before implementation.
3. `/v1/agent/capabilities` returns operation metadata, not only permissions.
4. `/v1/agent/schemas` and `/v1/openapi.json` exist or are generated in `docs/current/generated/`.
5. `docs/current/API_REFERENCE_CURRENT.md` remains a route inventory, but authoritative machine schemas come from OpenAPI/JSON Schema.
6. Every route in `docs/current/API_REFERENCE_CURRENT.md` is classified in a canonical route map.
7. All non-read operations declare materialization mode, side-effect profile, idempotency requirement, version-check policy, budget profile, and permission scopes.
8. All sensitive operations support dry-run/preview/commit or are explicitly exempted with rationale.
9. Command idempotency is enforced, not merely echoed.
10. Workpoint idempotency is promoted from route-local cache to the shared idempotency ledger or marked operator-preview only.
11. Success and failure responses align with the standard AX envelopes.
12. Error taxonomy is generated/tested across at least one happy-path and one blocked-path test per critical family.
13. Route-scope middleware is validated against the operation registry in CI.
14. Natural-language fields are schema-annotated as untrusted.
15. Low-level/proxy/query routes declare query-power hardening metadata.
16. Public stream/Arena/UIAI bridge cards declare redaction/evidence/public-safety metadata.
17. AX simulation tests report first-call success, recovery quality, token usage, and unsafe-action prevention.
18. Completion claims cite actual evidence per Spec107.

## 23. Rollout Plan

### Phase A — Contract generation and route classification

- Create operation metadata registry.
- Generate canonical route map.
- Generate OpenAPI/JSON Schema skeletons.
- Add `/v1/agent/capabilities` compact endpoint.
- Add route-scope registry parity test.

### Phase B — Envelope standardization

- Add response envelope helpers.
- Wrap critical success responses: health/info/project/workpoint/trajectory/commands/work-loop/metacognition.
- Map current error envelopes to Spec109 taxonomy.
- Add compact/standard/debug response detail.

### Phase C — Mutation safety standardization

- Add global idempotency ledger.
- Add if-match/version conflict checks.
- Add side_effect_policy parsing.
- Add preview_token for sensitive routes.
- Require budgets for proxy/export/sync/work-loop/token/device-pairing/restore routes.

### Phase D — AX eval and public-proof compliance

- Add AX simulation tests.
- Add public stream/card schema.
- Add UIAI bridge evidence-intake compliance schema.
- Add llms.txt.
- Update docs and README pointers.

## 24. Implementation Priority Backlog

### P0

- AX operation metadata registry.
- Canonical route map.
- `/v1/agent/capabilities`.
- OpenAPI/JSON Schema generation.
- Standard response envelope helper.
- Typed error taxonomy mapping.
- Command idempotency enforcement.
- Route-scope vs metadata CI parity gate.

### P1

- Global idempotency ledger.
- Preview/commit mode standardization for critical writes.
- Version conflict checks.
- Budget object support.
- Low-level/proxy query hardening metadata.
- Natural-language trust annotations.
- Deprecated/alias route response metadata.

### P2

- Full public card schema.
- UIAI evidence-intake schema.
- AX simulation dashboards.
- Token-usage and first-call success reporting.
- llms.txt and docs reading-order hardening.
- Backward-compatible SDK/tooling examples.

## 25. Canonical Rule

> Focusa should expose powerful agent primitives directly — but every primitive must be discoverable, typed, scoped, budgeted, idempotent where mutating, observable, recoverable, and safe under retry.

Spec109 does not make Focusa less powerful. It makes Focusa's power easier for agents to use correctly.

# API Route Permission Matrix

Status: enforced canonical authorization contract (Workforce Full AC2). This matrix defines route scopes consumed by the daemon-owned `can(principal, capability, context)` gate.

## Scope vocabulary

| Scope | Meaning | Examples |
| --- | --- | --- |
| `public:health` | unauthenticated health only | `GET /v1/health` |
| `read:*` | all read-only state/metadata routes | local trusted use; broad read token |
| `state:read` | Focus/ASCC/state read | current state, stack, CLT state |
| `state:write` | Focus/ASCC/state mutation | focus update/push/pop/set-active |
| `workpoint:read` | Workpoint read/resume/drift-check | current/resume/status |
| `workpoint:write` | Workpoint checkpoint/evidence mutation | checkpoint/link evidence |
| `trajectory:read` | Trajectory view/assess/resume | view/gap/projection |
| `trajectory:write` | Trajectory candidate/checkpoint mutation | define-goal/checkpoint |
| `project:read` | Project identity/verify | identity/verify |
| `metacog:read` | Metacog retrieval/readbacks | retrieve/recent/doctor |
| `metacog:write` | Metacog capture/reflect/adjust/evaluate | learning writes |
| `prediction:read` | Prediction recent/stats | prediction reads |
| `prediction:write` | Prediction record/evaluate/outcome | prediction writes/evaluations |
| `telemetry:read` | Telemetry/status reads | memory/events/tokens/tools |
| `telemetry:write` | Telemetry ingest | trace/tool/cost/activity posts |
| `ontology:read` | Ontology/traverse/context refs | ontology and traversal reads |
| `events:read` | Event log reads/streams | events, SSE |
| `attachments:write` | Attachment attach/detach | attach/detach |
| `sync:admin` | Sync/peer/token management | peer registration/tokens/sync |
| `proxy:invoke` | Provider/proxy invocation | chat completion proxies |
| `work_loop:read` | Work-loop status/writer | status/health/writer |
| `work_loop:control` | Continuous work mutation | control/context/checkpoint/select-next |
| `admin:service` | daemon/service/admin operations | restart/kickstart/cleanup/release proofs |
| `admin:*` | all scopes | owner-local only |

## Route family matrix

| Family / route prefix | Methods | Intended scope | Privacy class | Notes |
| --- | --- | --- | --- | --- |
| `/v1/health` | GET | `public:health` | P1 | Only route that should remain public when auth is configured. |
| `/v1/info`, `/v1/env` | GET | `telemetry:read` | P1/P2 | `env` must not disclose secret values. |
| `/v1/state`, `/v1/ascc`, `/v1/focus` reads | GET | `state:read` | P2 | Focus State can contain project-sensitive summaries. |
| `/v1/focus/*`, `/v1/ascc/update-delta` | POST | `state:write` | P2 | Mutation routes. |
| `/v1/project/*` | GET/POST | `project:read` | P1/P2 | Verify/identity should be read-style; no task mutation. |
| `/v1/workpoint/*` reads | GET/POST resume/status/drift | `workpoint:read` | P2 | Resume may expose continuation summaries. |
| `/v1/workpoint/*` writes | POST checkpoint/link/evidence | `workpoint:write` | P2 | Canonical continuity mutations. |
| `/v1/trajectory/view`, `/assess`, `/resume` | GET/POST | `trajectory:read` | P2 | Advisory projections. |
| `/v1/trajectory/define-goal`, `/checkpoint` | POST | `trajectory:write` | P2 | Candidate/checkpoint writes. |
| `/v1/metacognition/*` reads | GET/POST retrieve/doctor/recent | `metacog:read` | P2/P3 | Retrieval can expose learned project context. |
| `/v1/metacognition/*` writes | POST capture/reflect/adjust/evaluate | `metacog:write` | P2/P3 | Learning-store mutations. |
| `/v1/predictions/recent`, `/stats` | GET | `prediction:read` | P2 | Prediction context can reveal project state. |
| `/v1/predictions`, `/evaluate`, `/capture-outcome` | POST | `prediction:write` | P2 | Forecast/evaluation mutations. |
| `/v1/ontology/*`, `/v1/traverse*`, `/v1/reflex/*` | GET/POST read-style | `ontology:read` | P1/P2 | Bounded traversal; full payload opt-ins remain sensitive. |
| `/v1/telemetry/*` reads | GET | `telemetry:read` | P1/P2 | Tool/token/event summaries. |
| `/v1/telemetry/*` writes | POST | `telemetry:write` | P1/P2 | Ingest endpoints should be bounded. |
| `/v1/events*`, SSE | GET | `events:read` | P2/P3 | Event payloads may be sensitive. |
| `/v1/attachments/*` | POST/GET | `attachments:write` for attach/detach; `read:*` for list | P2/P3 | Needs size/path constraints. |
| `/v1/sync*`, `/v1/tokens*`, peer routes | GET/POST | `sync:admin` | P3/P4 | Token/peer management is admin-sensitive. |
| `/proxy/*` | POST | `proxy:invoke` | P3 | Provider request/response boundary; never public unauthenticated. |
| `/v1/work-loop/*` status | GET | `work_loop:read` | P2 | Writer ownership/readiness. |
| `/v1/work-loop/*` controls | POST | `work_loop:control` | P2/P3 | Continuous execution mutation. |
| `/v1/release/*`, cleanup/daemon/service-adjacent commands | GET/POST | `admin:service` | P2/P3 | Service/release proof and operations. |

## Canonical decision contract

1. Global bearer authentication resolves the principal from daemon-owned state.
2. `can(principal, capability, context)` is the sole allow/deny authority for consequential routes.
3. Context binds request, Workstream, Workset, WorkItem, CallGraph frame, risk, and entitlement posture.
4. `x-focusa-permissions` is non-authoritative requested-scope metadata. It cannot add a grant; requested/effective mismatch is denied and audited as `CLIENT_SCOPE_ELEVATION_DENIED`.
5. Local loopback and the configured daemon admin token receive server-derived `admin:*` plus `risk:high`. Paired-device grants derive only from the stored device-token record.
6. Historic paired-device `read`/`write` scopes migrate to bounded `read:*`/`write:*`. Historic device `admin`/`admin:*` values do not activate administrator authority.
7. Every decision is persisted unchanged in `capability_authorization_audits`; audit failure fails the request closed.
8. Entitlement acceptance is proven by an `EntitlementGateAccepted` request extension, not a client claim or middleware-order assumption.

Route-local `permission_context(...).allows(...)` calls are compatibility shims only. They cannot grant or independently deny after the canonical middleware decision.

## Acceptance criteria for full enforcement

- Every registered route has a declared scope in a machine-readable inventory.
- Every mutation route checks a non-read scope when token auth is enabled.
- Health remains public; no other route is public on non-loopback deployments.
- Tests verify paired read/write tokens remain bounded and spoofed `admin:*` metadata cannot elevate.
- Allow/deny and the durable audit row are the same versioned decision.
- Tool/CLI docs mention required scopes for admin/service/proxy/control routes.

## Enforcement update

`crates/focusa-api/src/middleware/route_scope.rs` invokes the canonical core gate for every non-public route, using `request_principal` server-derived grants and the exact route context. `crates/focusa-api/src/middleware/entitlement.rs` supplies daemon-owned entitlement proof. Public health and bounded pre-auth pairing bootstrap remain explicit exceptions; the outer auth layer still governs configured admin-token posture.

Focused proof: `tests/workforce_full_ac2_canonical_can_gate_test.py`, the core authorization matrix/catalog/audit tests, API package tests, and `docs/evidence/workforce-full/ac2-proof.txt`.

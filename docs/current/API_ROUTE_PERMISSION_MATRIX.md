# API Route Permission Matrix

Status: security design baseline. This matrix defines the intended route scopes for Focusa API authorization. It is not a complete enforcement proof yet; current enforcement is partial and concentrated in `capabilities_extra.rs` plus global bearer-token auth.

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

## Enforcement target

1. Global bearer auth remains the outer gate.
2. Non-loopback startup without auth remains forbidden.
3. Each route family should call a scope guard equivalent to `require_scope(headers, state, "scope")`.
4. Default token scopes under auth should be read-only, not mutation-capable.
5. `admin:*` should be owner-local only and never required for normal read tooling.

## Current enforcement status

- `capabilities_extra.rs` already uses scoped permission helpers for many capability/gate/cache/export/reference routes.
- Most core routes still rely on global bearer auth and local-loopback posture, not route-level scopes.
- This matrix is the baseline for implementing route-level scoped authorization and for generating tests.

## Acceptance criteria for full enforcement

- Every registered route has a declared scope in a machine-readable inventory.
- Every mutation route checks a non-read scope when token auth is enabled.
- Health remains public; no other route is public on non-loopback deployments.
- Tests verify default authenticated token context cannot mutate state.
- Tool/CLI docs mention required scopes for admin/service/proxy/control routes.

## Enforcement update

`crates/focusa-api/src/middleware/route_scope.rs` now enforces this route-family scope baseline whenever `FOCUSA_AUTH_TOKEN` is configured. Default authenticated tokens remain read-oriented via `permission_context`; mutation routes require explicit write/control/admin scopes in `x-focusa-permissions`. `tests/security_api_route_scope_dynamic_test.sh` starts a temporary auth-enabled daemon and verifies unauthenticated reads return 401, default-token writes return 403, and `telemetry:write` can write telemetry.

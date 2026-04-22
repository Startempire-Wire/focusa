# docs/12-api.md — Local API (HTTP) Specification (MVP)

## Purpose
Provide a stable local API for:
- CLI control
- Menubar UI visibility
- Adapter integration (proxy mode)

## Server
- Default bind: `127.0.0.1:8787`
- Configurable via env/config.
- No auth in MVP (localhost only); add token later.

## Content Types
- JSON request/response for commands.
- Optional SSE or WS for event streaming.

## Endpoints (MVP)

### Health
GET `/v1/health`
Response:
- `{ "ok": true, "version": "...", "uptime_ms": 12345 }`

### Status
GET `/v1/status`
Response includes:
- active focus frame summary
- stack depth
- background worker status
- last event timestamp
- prompt stats (last assembled size)

### Focus Stack
GET `/v1/focus/stack`
POST `/v1/focus/push`
POST `/v1/focus/pop`
POST `/v1/focus/set-active`

Canonical `GET /v1/focus/stack` response shape:
- `stack`: object (`FocusStackState`), **not** a plain array
  - `root_id`
  - `active_id`
  - `frames[]`
  - `stack_path_cache[]`
  - `version`
- `active_frame_id`: uuid|null (alias of `stack.active_id`)

### Focus Gate
GET `/v1/focus-gate/candidates`
POST `/v1/focus-gate/ingest-signal`
POST `/v1/focus-gate/surface`
POST `/v1/focus-gate/suppress`
POST `/v1/focus-gate/pin`
POST `/v1/gate/signal` (alias of ingest-signal)

Exact request schemas:
- `POST /v1/focus-gate/ingest-signal` and `POST /v1/gate/signal`
  - `kind: "user_input_received"|"tool_output_captured"|"error_observed"|"repeated_warning"`
  - `summary: string`
  - `frame_context: uuid` (optional)
- `POST /v1/focus-gate/surface`
  - `candidate_id: uuid`
  - `boost: number` (optional, default `1.0`)
- `POST /v1/focus-gate/suppress`
  - `candidate_id: uuid`
  - `scope: string` (optional, default `"session"`)
- `POST /v1/focus-gate/pin`
  - `candidate_id: uuid`

Candidates response includes:
- id
- label
- source
- current surface pressure
- last_seen_at

### Prompt Assembly
POST `/v1/prompt/assemble`

Exact request schema (required unless marked optional):
- `turn_id: string` (required, non-empty)
- `raw_user_input: string` (required)
- `format: "string"|"messages"` (optional; default `messages`)
- `budget: integer` (optional)

Response:
- Canonical: `assembled`, `stats`, `handles_used[]`
- Compatibility (migration): `assembled_prompt`, `context_stats`

### Turn Lifecycle (Adapter)
POST `/v1/turn/start`
POST `/v1/turn/append` (optional)
POST `/v1/turn/complete`

Exact request schemas:
- `POST /v1/turn/start`
  - `turn_id: string` (required)
  - `harness_name: string` (required)
  - `adapter_id: string` (required)
  - `timestamp: RFC3339 datetime string` (required)
- `POST /v1/turn/append`
  - `turn_id: string` (required)
  - `chunk: string` (required)
- `POST /v1/turn/complete`
  - `turn_id: string` (required)
  - `assistant_output: string` (required)
  - `artifacts: string[]` (required; can be empty)
  - `errors: string[]` (required; can be empty)
  - Idempotency: duplicate `turn_id` returns `{ status: "accepted", duplicate: true }` and must not emit a second `turn_completed` event.

### ASCC
GET `/v1/ascc/frame/:frame_id`
POST `/v1/ascc/update-delta`

### Focus
GET `/v1/focus/stack`
GET `/v1/focus/frame/current`
- Query `frame_id: uuid` (optional)
- Query `session_key: string` (optional)
- Returns one scoped frame plus `matched_by` and `active_frame_id`; keeps full stack endpoint available for deep/debug reads.

### ECS
GET  `/v1/ecs/handles`
POST `/v1/ecs/store`
GET  `/v1/ecs/resolve/:handle_id`
GET  `/v1/ecs/content/:handle_id`
POST `/v1/ecs/rehydrate/:handle_id`

Exact request schemas:
- `GET /v1/ecs/handles`
  - Query `limit: integer` (optional, capped server-side to `512`; returns most recent handles)
  - Query `summary_only: boolean` (optional; when `true`, omits blob-heavy metadata like `sha256`, `size`, `session_id`)
  - Response includes `count` = total handle count before limiting
- `POST /v1/ecs/store`
  - `kind: "log"|"diff"|"text"|"json"|"url"|"file_snapshot"|"other"`
  - `label: string`
  - `content_b64: base64 string`
- `POST /v1/ecs/rehydrate/:handle_id`
  - Query `max_tokens: integer` (optional, default `300`)

### Memory
GET `/v1/memory/semantic`
POST `/v1/memory/semantic/upsert`
GET `/v1/memory/procedural`
POST `/v1/memory/procedural/reinforce`

Exact request schemas:
- `GET /v1/memory/semantic`
  - Query `limit: integer` (optional, capped server-side to `512`; returns most recent records)
  - Query `summary_only: boolean` (optional; when `true`, omits heavier metadata like `source`, `confidence`, `tags`, `ttl`)
  - Response includes `count` = total semantic record count before limiting
- `POST /v1/memory/semantic/upsert`
  - `key: string`
  - `value: string`
  - `source: "user"|"worker"|"manual"` (optional, default `user`)
- `POST /v1/memory/procedural/reinforce`
  - `rule_id: string`

### Reflection Loop (Overlay)
POST `/v1/reflect/run`
GET `/v1/reflect/history?limit=50&mode=scheduled&stop_reason=low_confidence&since=2026-03-01T00:00:00Z&until=2026-03-02T00:00:00Z&cursor_before=2026-03-01T12:00:00Z`
- `mode` filter accepts `manual|scheduled`; invalid values return `400 { code: "invalid_mode_filter", ... }`
- `since` / `until` must be RFC3339; invalid values return `400 { code: "invalid_time_filter", ... }`
- history response includes `applied_filters` echo (`limit`, `requested_limit`, `mode`, `stop_reason`, `since`, `until`, `cursor_before`)
- history limit is bounded server-side to `1..200`
- history response includes `next_cursor` for paging older results
GET `/v1/reflect/status`
  - telemetry includes `latest_window_key`, `latest_window_run_count`, `last_scheduler_run_at`, `stop_reason_counts`
GET `/v1/reflect/scheduler`
  - includes telemetry counters: `latest_window_key`, `latest_window_run_count`, `last_scheduler_run_at`
POST `/v1/reflect/scheduler`
POST `/v1/reflect/scheduler/tick`

Exact request schema (`POST /v1/reflect/run`):
- `mode: "manual"|"scheduled"` (required)
- `idempotency_key: string` (optional but recommended)
- `window: string` (optional, e.g. `"1h"`)
- `budget: integer` (optional)
- `context: object` (optional)

Response includes:
- `iteration_id`
- `window`
- `observations[]`
- `risks[]`
- `recommended_actions[]`
- `confidence`
- `stop_reason` (`single_iteration_complete|no_evidence_delta|repeated_recommendation_set|low_confidence`)

Scheduler config schema (`GET/POST /v1/reflect/scheduler`):
- `enabled: bool`
- `interval_seconds: integer`
- `max_iterations_per_window: integer`
- `cooldown_seconds: integer`
- `low_confidence_threshold: float (0..1)`
- `no_delta_min_event_delta: integer`

Scheduler tick (`POST /v1/reflect/scheduler/tick`) returns:
- accepted reflection result envelope, or
- `{ status: "skipped", reason: "scheduler_disabled|cooldown_active|window_iteration_cap_reached" }`

### Events
GET `/v1/events/recent?limit=200`
GET `/v1/events/stream` (SSE optional)

## Error Model
All errors return:
- HTTP status code
- JSON `{ code, message, details?, correlation_id? }`

## Determinism Requirement
API commands must be idempotent where applicable:
- repeated `status` reads do not mutate state
- `turn/complete` with same turn_id must not double-apply (use turn_id dedupe)

## Canonical fixtures
Request fixtures live at `docs/fixtures/api/`:
- `turn_start.request.json`
- `prompt_assemble.request.json`
- `gate_signal.request.json`
- `ecs_store.request.json`
- `memory_reinforce.request.json`

These fixtures are the source of truth for smoke tests and contract probes.

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

Request/Response must include:
- `stack` array
- `active_frame_id`

### Focus Gate
GET `/v1/focus-gate/candidates`
POST `/v1/focus-gate/ingest-signal`
POST `/v1/focus-gate/surface`
POST `/v1/focus-gate/suppress`

Candidates response includes:
- id
- label
- source
- current “surface pressure” (internal numeric allowed but do not expose as “attention”)
- last_seen_at

### Prompt Assembly
POST `/v1/prompt/assemble`
Request:
- `turn_id`
- `raw_user_input`
- `format`: `"string"|"messages"`
- `budget`: token budget target
Response:
- `assembled`
- `stats`: approximate token counts
- `handles_used[]`

### Turn Lifecycle (Adapter)
POST `/v1/turn/start`
POST `/v1/turn/append` (optional)
POST `/v1/turn/complete`

### ASCC
GET `/v1/ascc/frame/:frame_id`
POST `/v1/ascc/update-delta`

### ECS
POST `/v1/ecs/store`
GET  `/v1/ecs/resolve/:handle_id`

### Memory
GET `/v1/memory/semantic`
POST `/v1/memory/semantic/upsert`
GET `/v1/memory/procedural`
POST `/v1/memory/procedural/reinforce`

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

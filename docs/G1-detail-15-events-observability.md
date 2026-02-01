# docs/15-events-observability.md — Event Schema, Logging, and Tracing

## Why Events Matter
Events are the canonical truth for:
- debugging
- UI visualization later
- auditing state changes
- acceptance testing

Every meaningful state mutation MUST emit an event.

## Event Record (JSON)
Fields:
- `id`: string (uuidv7 recommended)
- `ts`: RFC3339 string
- `type`: string enum
- `origin`: `"cli"|"gui"|"adapter"|"worker"|"daemon"`
- `correlation_id`: string (turn_id or request id)
- `payload`: object (type-specific)
- `severity`: `"debug"|"info"|"warn"|"error"`

## Event Types (MVP)

### Focus Stack
- `focus.frame_pushed`
- `focus.frame_popped`
- `focus.active_changed`

Payload includes:
- frame_id
- parent_id
- label/goal
- stack_depth

### Focus Gate
- `gate.signal_ingested`
- `gate.candidate_created`
- `gate.candidate_surfaced`
- `gate.candidate_suppressed`

Payload includes:
- candidate_id
- source
- hint_kind (novelty/risk/persistence/etc.)
- pressure (internal metric ok)

### Prompt
- `prompt.assembled`
Payload:
- format
- token_estimate
- included_slots[]
- handles_used[]
- budget_target

### ASCC
- `ascc.delta_applied`
Payload:
- frame_id
- sections_updated[]
- anchor_id (turn_id)

### ECS
- `ecs.artifact_stored`
- `ecs.handle_resolved`
Payload:
- handle_id
- kind
- size_bytes
- sha256

### Memory
- `memory.semantic_upserted`
- `memory.rule_reinforced`
- `memory.decay_tick`

### Worker
- `worker.job_enqueued`
- `worker.job_started`
- `worker.job_completed`
- `worker.job_failed`

### Adapter/Turn
- `turn.started`
- `turn.completed`
- `turn.failed`

## Storage
- append-only JSONL at `~/.focusa/log/events.jsonl`
- rotation policy:
  - size cap (config, default 50MB)
  - keep last N rotated files (default 5)

## Tracing
MVP: structured logs + correlation_id propagation.
Later: integrate OpenTelemetry (not required).

## UI Hook
Menubar UI reads:
- `/v1/status`
- `/v1/events/recent`
Optionally subscribes to `/v1/events/stream` for live updates.

---

# UPDATE

# docs/15-events-observability.md (UPDATED) — Replay Invariant

## Replayability Invariant (Declared)

> Focusa state MUST be reconstructible by replaying events in order.

MVP does not require replay tooling, but:
- reducer transitions must be pure
- events must be complete

This enables:
- debugging
- audits
- future time-travel UI

# docs/G1-14-reflection-loop.md — Reflection Loop Overlay (Policy-Safe Meta-Cognition)

## Purpose
Define a controlled reflection loop that lets a model periodically review work quality and focus trajectory **without** violating Focusa principles (advisory, deterministic, human-aligned).

## Boundary
Reflection loop is an **overlay workflow**, not autonomous control mode.

Must not:
- auto-switch active focus frame
- auto-write memory without policy path
- execute tools outside normal permission model
- recurse indefinitely

## Inputs
- Current Focusa state (`/v1/status`, focus stack)
- Recent events (`/v1/events/recent`)
- Optional operator prompt/context

## Outputs
Per iteration, produce structured result:
- `iteration_id`
- `window` (time range)
- `observations[]`
- `risks[]`
- `recommended_actions[]` (advisory)
- `confidence` (0..1)
- `stop_reason`

## Cadence Modes
- `manual` — one-shot run
- `scheduled` — periodic, bounded by policy
  - daemon loop triggers `/v1/reflect/scheduler/tick` using configured `interval_seconds`

## Safety Controls (required)
1. **Budget cap** per iteration (`max_tokens`, `max_steps`)
2. **Rate cap** (`min_interval_seconds`)
3. **Loop cap** (`max_iterations` per window)
4. **Stop conditions**:
   - low confidence (`stop_reason: "low_confidence"`, threshold-configured)
   - no new evidence delta (threshold-configured min event delta)
   - repeated recommendation set (`stop_reason: "repeated_recommendation_set"`)

## Idempotency
Reflection run request supports idempotency key:
- duplicate key in same window returns prior result
- no duplicate persistence side effects

## API Contract (overlay)
- `POST /v1/reflect/run`
  - request: `{ mode, idempotency_key?, window?, budget?, context? }`
  - response: reflection result envelope
- `GET /v1/reflect/history?limit=...`
  - recent reflection iterations
- `GET /v1/reflect/status`
  - scheduler/guardrail status

## CLI Contract (overlay)
- `focusa reflect run [--window 1h] [--budget 800] [--idempotency-key k]`
- `focusa reflect history [--limit 20]`
- `focusa reflect status`

## Persistence
Reflection iterations are persisted as events/results with:
- `trace_id`
- `correlation_id`
- `idempotency_key`
- `event_time` + `ingest_time`

## Acceptance Gates
- R1: duplicate run with same idempotency key does not create new iteration
- R2: stop conditions trigger deterministically for repeated no-delta windows
- R3: API/CLI responses follow global JSON error envelope rules
- R4: reflection recommendations remain advisory unless explicitly applied through existing action paths

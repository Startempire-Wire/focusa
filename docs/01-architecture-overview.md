# docs/01-architecture-overview.md — MVP Architecture Overview

## System Boundary
Focusa is a local “cognitive proxy” that:
- intercepts prompts and responses,
- manages focus state and context fidelity,
- injects minimal structured context and references,
- maintains lightweight memory,
- exposes local observability (API/GUI/CLI).

Focusa MUST remain backend-agnostic:
- It cannot depend on internal APIs of Letta or other harnesses.
- It can implement adapter “drivers” that speak generic I/O protocols (stdin/stdout, HTTP).

## High-Level Diagram

User/Harness (Letta/Claude/Codex/Gemini)
  |
  | (prompt/request)
  v
Focusa Proxy Layer
  |  - Focus Stack (HEC)
  |  - Focus Gate (salience)
  |  - ASCC (structured checkpoints)
  |  - ECS (artifact offloading)
  |  - Memory (semantic/procedural)
  |  - Prompt Assembly
  |
  v
Model Endpoint / Harness Backend

## Planes (MVP)

### 1) Cognitive Control Plane
- Focus Stack (HEC): frames, push/pop, current focus.
- Focus Gate: organic surfacing of priority candidates.

### 2) Context Fidelity Plane
- ASCC: structured summaries updated incrementally.
- ECS: externalize large blobs; represent them as handles.

### 3) Memory Plane (minimal)
- Semantic memory: small facts/preferences.
- Procedural memory: rules derived from repeated patterns.

### 4) Background Cognition Plane
- One async worker that:
  - performs classification,
  - updates ASCC deltas,
  - produces salience hints to Focus Gate.

### 5) Interfaces
- CLI: primary control for engineers.
- Local API: status, stack, memory, events.
- Menubar UI: passive observability + basic controls.

## Determinism & Safety Rules
1. Focus Gate is **advisory** only.
2. Prompt Assembly is **deterministic** given state + inputs.
3. Any large data MUST be externalized when above threshold.
4. No component may introduce blocking latency to request/response path beyond a strict budget (see performance budgets).
5. All state mutations must be logged as events.

## Performance Budgets (MVP)
- Hot path (proxy request processing):
  - < 20ms additional overhead on prompt assembly on typical machines.
- Background tasks:
  - async; never block hot path.
- Storage:
  - local file store operations should be batched where possible.

## Data Persistence (MVP)
- All persistence is local-first.
- Canonical storage:
  - SQLite (append-only events + versioned snapshots + telemetry/UXP/UFI indices)
  - filesystem ECS objects (blobs)
- Must survive daemon restart.
- Multi-device sync is supported via event exchange (no silent merges).
  See: `docs/43-multi-device-sync.md`

## Configuration (MVP)
Single config file + env overrides:
- store paths
- adapter selection
- thresholds (token & size)
- API listen address
- logging verbosity

## Deliverables (MVP)
- Rust daemon with local API + event log.
- Rust CLI controlling daemon.
- Generic proxy adapter (stdin/stdout wrapper) with at least one harness path validated (Letta usage).
- Minimal menubar app showing status, focus, background activity.

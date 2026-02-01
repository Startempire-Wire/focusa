# docs/03-runtime-daemon.md — Daemon Runtime, State, and Persistence

## Daemon Overview
`focusa-daemon` is a local resident process that owns:
- authoritative state
- persistence
- API surface
- background worker scheduling
- event emission/logging

It MUST be safe to run continuously.

## Process Model
- single daemon process
- one Tokio runtime
- state is mutated via an internal reducer (event-driven model)

### Internal Loop
- inbound requests:
  - API commands (CLI/GUI)
  - adapter events (proxy layer)
- state updates:
  - apply reducer action -> new state
  - emit event records
- outbound:
  - respond to API
  - stream events (optional)

## State Model (MVP)

### `AppState`
- `focus_stack: FocusStackState`
- `focus_gate: FocusGateState`
- `ascc: AsccState`
- `ecs: EcsState`
- `memory: MemoryState`
- `workers: WorkerState`
- `adapters: AdapterState` (minimal bookkeeping)
- `metrics: MetricsState` (counters, last activity timestamps)

### Persistence Rules
Persist these on mutation (debounced/batched):
- focus_stack
- ascc checkpoints
- ecs index (artifact metadata)
- memory (semantic/procedural)
- worker metadata (optional)
- event log index (for replay/inspection)

Persist mechanism:
- local directory (default: `~/.focusa/`)
- JSON files for state + append-only JSONL event log
- ECS artifacts stored as files under `~/.focusa/ecs/`

## Event Log (MVP)
Append-only JSONL:
- every state mutation must emit an event
- events have:
  - id (monotonic or UUIDv7)
  - timestamp
  - type
  - payload
  - correlation_id (request/turn id)
  - origin (cli/gui/adapter/worker)

Event log is bounded:
- keep last N MB or last N days (config)
- older logs can be compacted later (non-MVP)

## Concurrency & Locking
Use a single state owner model:
- `Arc<RwLock<AppState>>` OR actor-style channel ownership
Preferred for MVP:
- single owner task with `mpsc` command channel (actor)
- avoids lock contention and simplifies determinism

## Command Handling (Reducer)
Define `Action` enum:
- Focus actions:
  - `PushFrame`
  - `PopFrame`
  - `SetActiveFrame`
- Gate actions:
  - `IngestSignal`
  - `SurfaceCandidate`
  - `SuppressCandidate`
- ASCC actions:
  - `UpdateCheckpointDelta`
- ECS actions:
  - `StoreArtifact`
  - `ResolveHandle`
- Memory actions:
  - `UpsertSemantic`
  - `ReinforceRule`
  - `DecayTick`
- Worker actions:
  - `WorkerEnqueue`
  - `WorkerComplete`

Reducer:
- pure function: `(state, action) -> (state', events[])`
- side effects done outside reducer:
  - file writes
  - process I/O
  - network calls

## Background Scheduling
A single periodic tick:
- every `T` seconds:
  - run decay tick
  - flush pending persistence batch
  - check worker queue

Workers must not block hot path.

## Startup & Recovery
On start:
1) load config
2) ensure directories exist
3) load state snapshots
4) open event log
5) start API server
6) start worker scheduler

On crash/restart:
- state reloaded from snapshots
- event log is for inspection, not required replay in MVP

## Shutdown
- flush persistence
- stop API
- close event log cleanly

---

# UPDATE

# docs/03-runtime-daemon.md (UPDATED) — Sessions & Identity

## Session Identity Layer (MVP)

### Purpose
Ensure clean isolation across:
- multiple harness runs
- terminal restarts
- concurrent projects

### SessionId
- Generated UUIDv7 at proxy/session start
- Required field on:
  - Turn
  - Event
  - WorkerJob
  - ASCC anchor
  - Focus Gate signals

### WorkspaceId (Optional, MVP-light)
- String or hash (e.g. cwd hash)
- Used for grouping sessions
- Does NOT merge state automatically

### AppState (Updated)
Add:
- `current_session_id: SessionId`
- `sessions: HashMap<SessionId, SessionMeta>`

### SessionMeta
Fields:
- `session_id`
- `created_at`
- `adapter_id`
- `workspace_id?`
- `status: active | closed`

### Persistence
- Session metadata persisted to:
  `~/.focusa/state/sessions.json`

### Invariants
1. All state mutations must include session_id
2. Reducer rejects cross-session writes
3. Events without session_id are invalid

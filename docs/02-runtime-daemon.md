# docs/02-runtime-daemon.md — Focusa Runtime & Daemon (MVP)

## Purpose

The Focusa Runtime (daemon) is the **single authoritative execution context** for all cognitive state.

It is responsible for:
- maintaining Focus State
- enforcing Focus Stack invariants
- running the Intuition Engine
- applying the Focus Gate
- coordinating the Expression Engine
- persisting state safely
- remaining fast and invisible

The daemon is **not** an agent, planner, or orchestrator.

---

## Core Invariants

1. Exactly one daemon instance owns mutable state per session
2. All state transitions are deterministic
3. All mutations emit events
4. No background task may block the hot path
5. No subsystem may bypass Focus Gate or Focus Stack

---

## Runtime Model

### Architecture Style
- Single-writer reducer loop
- Event-driven
- Async I/O
- Local-only

### Execution Flow (Per Turn)

```
Harness Input
     ↓
Session Validation
     ↓
Intuition Engine (async signal updates)
     ↓
Focus Gate (candidate surfacing)
     ↓
Focus Stack validation
     ↓
Focus State update
     ↓
Expression Engine
     ↓
Model Invocation
     ↓
State Persistence + Events
```

---

## Session Management

### Session Definition
A **Session** is an isolated execution context representing one continuous Focusa run.

### Session Properties
- session_id (UUIDv7)
- adapter_id
- workspace_id (optional)
- created_at
- last_activity
- status: active | closed

### Rules
- All state belongs to exactly one session
- Cross-session access is forbidden by default
- Restarting the daemon restores session state

---

## State Ownership

The daemon owns:

- Focus Stack
- Focus State (per frame)
- Focus Gate state
- Intuition Engine buffers
- Reference Store metadata
- Memory (semantic / procedural)
- Event log
- Beads mappings

No other component may mutate these directly.

---

## Event System

### Purpose
Events provide:
- auditability
- replayability
- debugging
- UI updates

### Event Properties
- event_id
- timestamp
- session_id
- event_type
- payload
- correlation_id

### Event Categories
- focus.*
- intuition.*
- gate.*
- reference.*
- expression.*
- session.*
- error.*

---

## Persistence

### Persisted Data
- Focus Stack
- Focus State
- Reference Store handles
- Memory
- Sessions
- Events

### Storage Model
- SQLite (canonical) for:
  - append-only events
  - versioned snapshots
  - UXP/UFI + telemetry indices
- File-backed Reference Store (ECS objects)

JSON/JSONL remain supported for export/import, but are not the canonical store.

### Guarantees
- Crash-safe writes
- Restart-safe recovery
- Deterministic reload

---

## Failure Handling

### Rules
- No silent failure
- No partial state writes
- No undefined behavior

### On Error
- Emit error event
- Preserve last valid state
- Surface failure via CLI / UI

---

## Performance Constraints

- Hot-path operations must complete <20ms typical
- Intuition Engine runs async only
- Expression Engine is deterministic and bounded
- Disk I/O batched where possible

---

## What the Runtime Is Not

- Not multi-writer
- Not distributed
- Not autonomous
- Not self-modifying
- Not cloud-connected

---

## Acceptance Criteria

- Long sessions remain stable
- No state corruption on restart
- No blocking from background tasks
- Event log fully reconstructs state

---

## Summary

The Focusa Runtime is the **stable ground of cognition**.  
It does not think, decide, or act — it **maintains coherence**.

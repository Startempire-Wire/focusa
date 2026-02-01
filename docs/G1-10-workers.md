# docs/10-workers.md — Background Workers & Async Cognition (MVP)

## Purpose
Background workers implement **System-1–like cognition**:
- fast
- asynchronous
- non-blocking
- heuristic-driven
- advisory only

They must **never** block prompt assembly or harness I/O.

In MVP, there is **one worker pipeline** with clearly defined job types.

---

## Worker Model

### Design Constraints
- Runs inside daemon process
- Uses async task queue
- Limited concurrency (default: 1–2 workers)
- Strict time budget per job
- Can be paused/disabled via config

### Worker Responsibilities (MVP)
- classify signals
- extract ASCC deltas
- propose Focus Gate candidates
- propose memory updates (advisory only)

Workers do **not**:
- mutate focus stack directly
- assemble prompts
- execute tools
- call the harness/model

---

## Worker Architecture

### WorkerQueue
- async channel (`mpsc`)
- bounded size (default 100 jobs)
- backpressure: drop low-priority jobs if queue full

### WorkerJob
Fields:
- `id`
- `kind: WorkerJobKind`
- `created_at`
- `priority: Low | Normal | High`
- `payload_ref: Option<HandleRef>`
- `frame_context: Option<FrameId>`
- `correlation_id`
- `timeout_ms`

### WorkerJobKind (MVP)
- `classify_turn`
- `extract_ascc_delta`
- `detect_repetition`
- `scan_for_errors`
- `suggest_memory`

---

## Job Execution Rules

### Scheduling
- jobs enqueued by daemon reducer
- high-priority jobs first
- max execution time per job (default 200ms)
- if timeout exceeded → cancel and emit failure event

### Safety
- workers must be panic-isolated
- failure does not affect daemon state
- emit `worker.job_failed` with error details

---

## Job Definitions

### classify_turn
Input:
- turn transcript (via handle or small text)
Output:
- tags (file paths, errors, tools, intent shifts)
- emit `gate.signal_ingested`

### extract_ascc_delta
Input:
- delta turn content
- current ASCC checkpoint
Output:
- structured delta proposal
- reducer applies merge rules (worker does not mutate state)

### detect_repetition
Input:
- recent signals / candidates
Output:
- repetition hint → Focus Gate

### scan_for_errors
Input:
- tool outputs / assistant output
Output:
- error signals with severity

### suggest_memory
Input:
- repeated stable patterns
Output:
- candidate memory suggestion (not applied automatically)

---

## Worker → Reducer Interaction
Workers return **results**, not state changes.

Reducer decides:
- whether to accept results
- whether to emit Focus Gate signals
- whether to enqueue follow-up jobs

---

## Events
Workers must emit:
- `worker.job_enqueued`
- `worker.job_started`
- `worker.job_completed`
- `worker.job_failed`

Each event includes:
- job id
- kind
- duration_ms
- correlation_id

---

## Persistence
Workers have **no persistent state**.
All persistence handled by reducer.

---

## Acceptance Tests
- Worker failure does not crash daemon
- Worker never blocks prompt assembly
- Jobs respect timeout
- Queue backpressure works

# docs/10-workers.md — Background Workers & Async Cognition (MVP)

## Purpose
Background workers implement **System-1–like secondary cognition**:
- fast
- asynchronous
- non-blocking to prompt assembly
- bounded by explicit time budgets
- advisory only

They must **never** block prompt assembly or harness I/O.

Workers may use either:
- local heuristic execution
- bounded LLM-backed execution

The execution method does not change worker authority.
Workers remain advisory result producers whose outputs must still flow through reducer/governance acceptance paths.

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
- perform direct canonical/reducer-bypass writes
- gain authority merely because an LLM was used during execution

Workers may call bounded model-backed extraction/evaluation paths when configured, but such calls remain subordinate to the same advisory-only contract.

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
- each job must have an explicit time budget
- 200ms remains the default local/heuristic budget unless overridden by job/config
- bounded LLM-backed jobs may use a larger configured timeout, but only with cancellation/failure handling and without turning workers into critical-path prompt assembly
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
- candidate classification result for downstream gate/reducer handling

### extract_ascc_delta
Input:
- delta turn content
- current ASCC checkpoint
Output:
- structured delta proposal
- reducer applies merge rules (worker does not mutate state)
- extraction may be heuristic or LLM-assisted, but output remains proposal-level rather than canonical truth

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
- whether any worker output is promoted, persisted, suppressed, or discarded

This remains true for both heuristic and LLM-backed worker execution.

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
- LLM-backed worker execution still returns advisory outputs only
- Timeout/fallback behavior is observable in traces/events
- Reducer remains the only authority that can turn worker output into lasting state change

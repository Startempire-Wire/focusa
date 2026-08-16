# Background execution and completion notification — first-class feature

Status: implemented (core + daemon + CLI + Pi extension), landing
2026-08-16. Supersedes ad-hoc `setsid nohup … > log &` wrappers.

## Why

The TBQ rule (terminal never blocks) required agents to dispatch
long-running work into the background. Shell wrappers were brittle: no
job identity, no durable completion record, no notification loop back to
the originating agent surface. This feature makes background execution a
typed, ledger-backed, observable core capability.

## Architecture

```
focusa bg run --name X -- <command…>
    │  1. POST /v1/background-jobs        → durable job row (queued)
    │  2. spawn child (detached pgid)     → status running + pid
    │  3. wait on child (the CLI IS the monitor)
    │  4. POST /v1/background-jobs/{id}/complete
    ▼
daemon: records completion durably (background_jobs table),
        THEN broadcasts focusa.stream_event.v1 envelope
        {event_type: "background_job_completion", …} on events_tx
    ▼
SSE /v1/events/stream (broadcast-channel surface, routes/sse.rs)
    ▼
Pi extension handleSSEEvent → uiCtx.notify("[bg] <name> <status> …")
focusa bg wait → long-poll ledger (500ms poll, bounded timeout)
focusa bg status → ledger row + monitor-lost reaping (/proc pid check)
```

## Surfaces

| Surface | Route / command |
| --- | --- |
| Core types | `focusa_core::background_jobs` (record, status, completion envelope) |
| Ledger | `focusa_core::background_job_store` (SQLite `background_jobs`) |
| Daemon | `routes/background_jobs.rs`: create/update/complete/get/list/wait |
| CLI | `focusa bg run --name X [--cwd D] -- <command…>`, `bg status --job`, `bg wait --job [--timeout-ms]`, `bg list` |
| Extension | `background_job_completion` SSE case → uiCtx.notify + metacog banner |

## Invariants

1. Completion is recorded durably BEFORE the SSE broadcast (mirrors #311).
2. The CLI monitor owns the lifecycle; a dead monitor is detected by
   `bg status` (pid liveness) and recorded as `monitor_lost` — never
   silently "running" forever.
3. Job output streams to the job's log file (reported in every envelope);
   the ledger stores the log path, not the log contents.
4. Waiters and the agent surface read the SAME completion envelope —
   no per-consumer reconstruction.
5. `focusa bg run` is the monitor process: dispatch it with
   `setsid nohup focusa bg run … &` for terminal detachment; the job
   record survives independent of the caller.

## Interaction audit (checked at landing)

- SSE path: `complete_job` → `events_tx` → `routes/sse.rs`
  `/v1/events/stream` (the only live stream route; `routes/events.rs`
  JSONL tail is deprecated and unregistered) → extension
  `handleSSEEvent`. Envelope carries `event_type` (the #45 lesson).
- Route precedence: static `/v1/background-jobs/wait` wins over the
  `{job_id}` param route in axum.
- No overlap with silent-session completion (#311): separate tables,
  same broadcast channel, distinct `event_type`.
- Extension typecheck green (tsc) with the new SSE case.
EOF

## Primitive workflow (agent default)

The bg loop is a Focusa PRIMITIVE: agents dispatch terminal-blocking
work through typed tools, not shells.

- Pi tools: `focusa_bg_run`, `focusa_bg_run_many` (parallel
  orchestration), `focusa_bg_status` — strict schemas, discovery via
  focusa_tool_search/describe.
- Delivery: the `background_job_completion` SSE envelope carries the
  bounded `output_tail`; the extension writes the completion + tail into
  the agent front terminal (notify banner + `pi.appendEntry` entry).
- Orchestration: `focusa_bg_run_many` fans out independent jobs in
  parallel; each job delivers its own completion — the multi-pipeline
  speed primitive.
- AGENTS.md (root + repo) mandates this workflow: raw shells only
  during cold-start recovery; tail-polling is banned.

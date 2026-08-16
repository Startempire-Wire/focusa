# focusa bg — background execution primitive (TBQ workflow)

## Purpose

The canonical non-blocking dispatch for terminal-blocking queries
(builds, tests, migrations, long scans, remote waits). Jobs are
first-class: durable ledger rows, detached execution, and a completion
notification (with a bounded output tail) delivered to the agent's
front terminal — no polling.

## CLI

```bash
focusa bg run --name <job> [--cwd <dir>] -- <command...>
focusa bg status --job <id>     # instant single query
focusa bg list                  # recent jobs
focusa bg wait --job <id> [--timeout-ms N]   # bounded blocking join
```

`focusa bg run` is the monitor: create row → spawn detached (pgid 0) →
wait → durable completion record → SSE broadcast
(`background_job_completion` with `output_tail`, bounded 4KB).
Dispatch it with `setsid nohup focusa bg run ... &` for terminal
detachment.

## Pi tools (typed primitives)

- `focusa_bg_run {name, command, cwd?}` — dispatch one job.
- `focusa_bg_run_many {jobs: [{name, command, cwd?}]}` — parallel
  orchestration; each job delivers its own completion notification.
- `focusa_bg_status {job_id?}` — instant ledger snapshot (never poll).

## Anti-patterns (banned)

- Repeated `tail` checks / `sleep N; tail` chains (tail-is-sleep).
- Raw `setsid nohup ... > log &` while the daemon is up.
- Treating the completion envelope as advisory: the SSE notification +
  output_tail IS the terminal delivery path.

## Recovery

Monitor-lost jobs are detected by `bg status` (pid liveness) and marked
`monitor_lost`. `focusa rebuild-state` recovers canonical state from the
event chain when a snapshot is unavailable.

# Background Execution and Completion Notification (TBQ Discipline)

**Status:** mandatory agent rule (AGENTS.md) + implemented machinery (#311)

## Rule

The operator terminal must never stop flowing. Any terminal-blocking query
(TBQ) — builds, test suites, migrations, long scans, waits for remote jobs —
MUST be dispatched to an asynchronous background process, and the agent must
continue other work immediately. `sleep`-polling loops are only the
last-resort status check; the completion-reporting surface below makes them
unnecessary.

Blocking is allowed only for sub-second commands and commands with an
explicit short bound whose output is required immediately.

## Dispatch pattern (canonical)

```bash
# Write the job as a script file, then detach it fully:
setsid nohup bash /path/to/job.sh > /path/to/job.log 2>&1 < /dev/null &
disown 2>/dev/null || true
```

Rules learned the hard way:

- Run the redirect through a script file — inline heredocs + `&` in tool
  sessions silently lose the redirect and the output goes to a dead socket.
- Prefer stdin for large SQL payloads (exec argv is capped at 128 KB per
  argument — `Argument list too long`).
- Never run two prune generations in parallel — they fight over the same
  write lock and pollute each other's logs.
- Every long job gets: a script file, a dedicated log, a completion marker
  line (`echo "JOB COMPLETE $(date +%F-%T)"`), and a bounded failure mode
  (retries with a stall exit code, never an infinite loop).

## Completion notification (#311)

When the work runs as a Focusa Silent Session, completion is pushed — not
polled:

1. The daemon's 30-second sweeper detects settled sessions
   (`completed|failed|cancelled`), records a deduped completion event
   (`silent_session_completion_events`, UNIQUE(session_id, run_id, status)),
   and broadcasts `type: silent_session_completed` over the existing SSE
   channel.
2. The Pi extension listens and shows one `uiCtx.notify` in the originating
   terminal: "Silent session <id> <status>: <summary>" (warning for
   failed/cancelled).
3. Scripts and humans block cleanly with
   `focusa silent wait --session-id <id> --timeout-secs 300`.
4. Missed events recover via
   `GET /v1/silent-sessions/completions?since_seq=<n>`.

Full design: `docs/159-silent-session-completion-notification.md`.

## Background job registry (session 2026-08-15)

| Job | Script | Log | State |
| --- | --- | --- | --- |
| Live DB maintenance (stop → prune junk → indexes → VACUUM → restart) | `/tmp/focusa-maintenance3.sh` | `/tmp/focusa-maintenance3.log` | running (daemon briefly down during window) |
| Gate chain v7 (retention + #311 tests/checks/clippy) | remote build host | `/tmp/focusa311-build7.log` | running |
| Extension typecheck (deployed line) | — | `/tmp/focusa311-extcheck.log` | green (`EXT-TYPECHECK-OK`) |

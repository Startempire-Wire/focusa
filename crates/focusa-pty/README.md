# focusa-pty — governed persistent Pi PTY runtime

One persistent Pi process per governed Attachment, backed by a real PTY
library (`portable-pty`). Ordinary child-process pipes are never used.

## PTY-004 — identity + registry
- `identity` — exact AttachmentKey/WorkSurfaceId mirror of the renderer contract.
- `registry` — one process per governed Attachment; duplicate attach is
  idempotent; view switches never kill the process (registry outlives views).

## PTY-005 — persistent Pi PTY process
- Spawn fails BEFORE process creation on scope mismatch or missing Attachment.
- One persistent process per Attachment with resize/input/output/interrupt/
  detach/close/restart and stale-output rejection.

## PTY-006 — ordered output events
- Partial reads preserve bytes; every event carries the exact identity, run
  generation, and monotonic sequence; a stale generation cannot impersonate
  the current process.

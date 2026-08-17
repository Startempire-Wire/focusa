# Silent Session Completion Notification (#311)

**Issue:** Startempire-Wire/focusa#311
**Goal:** terminal-blocking queries run asynchronously, and their completion
is *reported* back into the originating terminal — the agent is told, not
reminded. No sleep-polling.

## Problem

Silent Sessions produced durable receipts/evidence but no push on run
settlement. Agents polled projections (or files), wasting turns and racing
stale reads. tmux has no agent-facing completion signal either.

## Design

### 1. Durable completion-event ledger (`focusa-core`)

`crates/focusa-core/src/silent_session_completion_events.rs`:

- Table `silent_session_completion_events(seq AUTOINCREMENT, session_id,
  run_id, generation, status, summary, evidence_refs JSON, created_at,
  UNIQUE(session_id, run_id, status))` — created on demand
  (`ensure_schema`), plus session/created indexes.
- `record_completion_event` — `INSERT OR IGNORE`; returns whether the event
  was new (dedupe = exactly-one-event per settled run).
- `latest_completion(session_id)`, `recent_completions(since_seq, limit)`
  (backfill cursor), `is_terminal_lifecycle` (`completed|failed|cancelled`).

### 2. API (`crates/focusa-api/src/routes/silent_sessions_wait.rs`)

- `GET /v1/silent-sessions/wait?session_id=&timeout_ms=&since_seq=`
  server long-poll: returns the durable completion event the moment the
  session settles; detects already-terminal-but-unrecorded sessions and
  records them on the fly; returns `{status:"waiting", timed_out:true,
  current:{lifecycle_state}}` on budget exhaustion.
- `GET /v1/silent-sessions/completions?since_seq=&limit=` — backfill for
  missed events (at-least-once with dedupe ids).
- `POST /v1/silent-sessions/sweep-completions` — forces one detection sweep.
- `sweep_completions(db_path, events_tx)` — scans `runtime_silent_sessions`
  for terminal lifecycle rows without a recorded event, records them, and
  broadcasts `{"schema":"focusa.silent_session_completion.v1",
  "type":"silent_session_completed", ...}` on the existing SSE channel.

### 3. Daemon

30-second completion sweeper in `main.rs` (spawned beside the daily
retention sweep). Push delivery therefore needs no client at all; the
wait/backfill endpoints exist for humans, scripts, and missed-event
recovery.

### 4. CLI

`focusa silent wait --session-id <id> [--timeout-secs 300]` — blocking
long-poll for shells and humans; exits when the session settles or the
budget expires. Also `focusa events prune` (see doc 158) for ledger
retention.

### 5. Pi extension

`apps/pi-extension/src/session.ts` handles the
`silent_session_completed` SSE event with a single `uiCtx.notify`
(`info`, or `warning` for failed/cancelled): the background terminal's
completion is written back into the originating terminal. Fail-silent per
§30 — a notification must never crash Pi. Applied to both the deployed
0.9.152 extension and the canonical `apps/pi-extension` tree.

### 6. Herdr

No integration exists (source-verified). Herdr (herdr.dev) is an
agent-aware terminal runtime with a socket API (agents can wait on pane
state), working/blocked/idle pane marking, and a 654-plugin marketplace.
A Focusa↔Herdr adapter (run silent sessions in Herdr panes; subscribe to
Herdr transitions) is an optional transport on top of this one — see the
research note on #311.

## Acceptance mapping

1. Exactly one completion event per settled run — UNIQUE + INSERT OR IGNORE.
2. `focusa silent wait` returns promptly with final status; headless.
3. Single Pi notify per completion; no polling timer added for it.
4. Events carry durable receipt/evidence refs; backfillable.

## Verification

- `cargo test -p focusa-core silent_session_completion_events` — idempotent
  record, latest lookup, backfill cursor, terminal-state detection.
- `cargo check -p focusa-api -p focusa-cli` + clippy `--all-targets -D
  warnings` for focusa-core/api/cli — clean.
- Extension typecheck clean on the deployed line (TS).
- Deployment note: activates with the next canonical release-pipeline
  deploy; no local release artifacts per policy. The sweeper is additive
  and no-op on old data.

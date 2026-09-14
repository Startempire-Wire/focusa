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
    │  1. POST /v1/background-jobs        → durable row + creator pid (queued)
    │  2. bind lifecycle monitor pid      → status running before child spawn
    │  3. spawn/wait child (CLI monitor)  → launch failures complete normally
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
2. The CLI monitor owns the lifecycle. Every created row records its creator
   PID while queued, then the exact lifecycle-monitor PID before child spawn.
   `bg status`, `bg list`, and `bg wait` reconcile a dead/stale queued creator
   as terminal `failed` with `failure_class=launch_failed`; a dead running
   monitor becomes terminal `monitor_lost`. Both receive `completed_at` and
   broadcast the normal completion envelope after durable settlement.
3. Job output streams to the job's log file (reported in every envelope).
   The completing monitor also sends the bounded `output_tail`, which the
   ledger stores durably before broadcast. Consumers never depend on the
   daemon sharing the monitor's filesystem or `/tmp` namespace. Legacy
   monitors without the durable tail retain direct log-path fallback and,
   on Linux while the monitor is alive, a bounded `/proc/<pid>/root` fallback
   for `PrivateTmp` cross-version interoperability.
4. Waiters and the agent surface read the SAME completion envelope —
   no per-consumer reconstruction.
5. `focusa bg run` is the monitor process. Canonical non-blocking dispatch
   uses `focusa bg run --detach`; raw `setsid`/`nohup` job wrappers are not a
   supported dispatch surface. `--cwd` is applied to the child command in
   both foreground-monitor and detached-monitor modes; recording a directory
   without executing there is forbidden.

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

## v2 upgrade — zero-poll delivery + visual TUI surfaces (2026-08-22)

Gap found in the field: the extension `handleSSEEvent` never implemented
the `background_job_completion` case (docs claimed it; code had no case),
`bg run` always blocked as monitor (forcing `setsid nohup … &` wrappers),
and the human had no persistent view of running jobs. v2 closes all three.

### Design

1. **Non-blocking dispatch — `bg run --detach`.**
   Parent CLI creates the durable job row, then re-execs itself
   (`current_exe`) as a detached process-group-0 monitor with
   `--internal-monitor`, prints `job <id> dispatched log <path>`, and
   exits immediately. The detached monitor performs the existing
   update-running → wait → POST /complete flow. Terminal NEVER blocks;
   no shell wrappers needed. Monitor-less drift is still covered by the
   existing `bg status` pid-liveness reaping (`monitor_lost`).

2. **Started broadcast — `background_job_started`.**
   `update_job` (status→running) broadcasts a
   `focusa.stream_event.v1` envelope `{event_type:
   "background_job_started", job_id, name, command, cwd, pid}` on the
   same `events_tx`. Surfaces see dispatch latency, not just completion.
   Invariant 1 (durable before broadcast) is preserved: upsert_job
   commits first, broadcast second.

3. **Agent front-terminal delivery (zero-poll).**
   Extension `handleSSEEvent` normalizes `evt.event_type ?? evt.type`
   and adds both bg cases:
   - `background_job_started` → footer status line updates.
   - `background_job_completion` → `uiCtx.notify("[bg] <name> <status>
     exit N · <first tail line>", ok ? "info" : "error")` + durable
     `pi.appendEntry("focusa-bg-completion", envelope)`. The agent reads
     completions from the pushed notification — polling/tailing banned.

4. **Human-visible progress (TUI).**
   - Footer status (`ctx.ui.setStatus("focusa-bg", …)`): `⚙ bg: N
     running · last <name> ✓/✗`, cleared when idle and no recent jobs.
   - Persistent widget (`ctx.ui.setWidget("focusa-bg", …)` above editor):
     live list of running jobs + last 3 completions with exit codes,
     themed via `theme.fg("success"/"error"/"muted")`.
   - State seeded from `GET /v1/background-jobs?limit=…` when SSE
     connects; updated incrementally from SSE events only. No polling.

### Acceptance

- AC1 `bg run --detach --name x -- true` returns <1s, job completes,
  ledger row `completed`, `bg status --job` shows it.
- AC2 SSE carries `background_job_started` then
  `background_job_completion` for one job, in order.
- AC3 extension typecheck green; completion notify text contains name +
  exit code + bounded tail line.
- AC4 widget renders ≥1 running job during a sleep job and clears to
  recent-completions view after.

## v2.1 hardening — durable receipt required (2026-08-28)

A Pi tool must never infer dispatch from successfully starting a local CLI
process. `focusa_bg_run` and every lane of `focusa_bg_run_many` invoke
`focusa bg run --detach --json` and report success only after parsing a
non-empty durable `job_id` and `log_path` from
`focusa.background_job_dispatch.v1` (legacy text receipts remain readable).
Quoted command strings are passed as one platform-shell payload, never split
on spaces.

The detached CLI monitor reuses the job row created by its parent through
hidden internal binding arguments; it must not create a second queued row.
`focusa_bg_status` normalizes API bases once and fails closed on HTTP errors,
missing jobs, invalid JSON, or malformed ledger envelopes. Entitlement and
daemon failures remain visible tool failures and never produce “dispatched”
wording. The daemon must declare and merge the background-job router; the
isolated same-row e2e is the registration and monitor-parity release proof.

## v2.2 — exact Workstream delivery and durable Pi rendering

Background execution is daemon-global; Pi progress is not. A completion may
render only in the native Pi session that dispatched it.

1. After project verification, the Pi extension promotes a private per-session
   bootstrap runtime into a verified `AttachmentKey` using
   `AsyncLocalStorage.enterWith()`. It re-keys the same runtime object; it never
   mutates the process-global bootstrap key.
2. The extension publishes that non-secret typed key as
   `FOCUSA_ATTACHMENT_KEY_V1`. A child `focusa bg run` validates it and includes
   it in the create request. Manual callers without the variable remain valid
   unscoped producers.
3. New records use `focusa.background_job.v2` and persist optional
   `attachment_json`. Migration is additive; v1 rows remain readable and
   explicitly unscoped. Started/completion SSE envelopes copy the attachment
   from the durable row.
4. Pi compares every ScopeRef, continuity, instance, session, and attachment
   field. Missing or foreign attachments are inert: no footer, widget,
   notification, or transcript entry. It never infers scope from `cwd`.
5. Running/recent visual state lives in `AttachmentRuntimeState`, never module
   globals. This preserves two-session isolation through switch, resume, and
   rehydration.
6. `pi.appendEntry("focusa-bg-completion", …)` stores a bounded, ANSI-sanitized
   `focusa.pi_background_completion_entry.v1`. The registered entry renderer
   makes new and restored receipts visible. `sendMessage()` is forbidden, so
   progress never enters model context.

Acceptance requires producer/store tests, Pi exact-attachment and restored
renderer tests, two-session runtime promotion/isolation, cross-version v1 row
migration, and a live scoped completion after canonical installation.

## v2.3 — typed pre-start launch settlement (#502)

A successfully created background-job row may never remain permanently
`queued` because log opening, monitor spawn, child spawn, or the transition to
`running` failed. The create request records the creator PID. A detached parent
rebinds the same row to the exact monitor PID without changing status. The
monitor blocks on a one-use inherited PID-checked pipe until that rebind is
acknowledged; only then may it record `running`, open the log, or spawn the
child.

Every pre-start failure uses the existing `/complete` route with terminal
`status=failed`, `failure_class=launch_failed`, exit code 126, and a bounded
stage-tagged diagnostic. The daemon commits that row before broadcasting the
ordinary `background_job_completion` SSE. It preserves the original CLI error;
a settlement failure is appended rather than substituted or hidden.

Rows from interrupted/legacy creators are reconciled by `bg status`, `bg list`,
and `bg wait`: a queued row with a dead creator PID settles immediately; a
queued row lacking monitor identity settles after a 30-second grace period.
PID liveness uses `/proc` on Linux, signal-zero probing on other Unix systems,
and a bounded process-handle probe on Windows. Ambiguous permission/probe
failures treat the process as live (fail closed against false settlement).

Acceptance requires both direct and detached unspawnable-command e2e proof:
one durable row, terminal `failed`, typed `launch_failed`, exit 126,
`completed_at`, bounded diagnostic, and the matching completion envelope. No
failure may be relabeled `monitor_lost`, and no test may blanket-skip Windows.

## v2.4 — daemon-owned stale-row repair with v1/v2 compatibility

New rows use `focusa.background_job.v3` and may carry an OS process-start token
alongside the PID. This prevents PID reuse from making an unrelated process look
like the lifecycle owner. The migration only adds the nullable
`process_start_token` column; v1 and v2 records remain readable unchanged.

The daemon reconciles nonterminal rows once at startup and again before job-list
responses. A queued legacy row without any PID settles after the existing
30-second grace period. A row with a PID settles only when that process is gone
or its available start token mismatches. If the platform cannot verify the start
token for a live process, reconciliation leaves the row untouched rather than
risking a false completion.

Queued rows settle as typed `launch_failed`; running rows settle as
`monitor_lost`. Both receive one completion time, failure code, and bounded
reason. Existing CLI-side reconciliation remains as a compatibility fallback
for older daemons, while the daemon is the owner for current installations.

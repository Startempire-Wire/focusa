# Spec 133a — Silent Session runner delivery (gap closure for #679)

Status: implementation plan, source-verified against `8cfb52eac`.
Parent spec: `docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md`.

## 1. Verified finding

`#679` is not a mis-set flag. The delivery half of Spec 133 §7 is unwired:

- `crates/focusa-core/src/silent_sessions/runner_client.rs` implements the protected
  local protocol against a Unix socket, and is **re-exported but never called**
  (`runner_client::*` appears only in `silent_sessions/mod.rs:70`).
- No non-test code constructs a `SilentSessionScheduler`
  (`silent_session_scheduler.rs` is declared in `lib.rs:190` and used only by tests
  and `runtime_entrypoint_classification.rs`).
- No `Command::new` exists anywhere in the Silent Session path, so nothing ever
  launches `focusa-session-runner`.
- `silent_session_scheduler.rs` is a dispatch/admission overlay
  (resource + writer + entitlement admission); it computes candidates but performs
  no delivery.
- Start/input/restart handlers durably append the request and answer
  `status: "queued"` (`silent_sessions_control.rs:383`, `silent_sessions_input.rs:358`).

Consequence: the reducer can reach `Queued` but nothing advances `Queued → Launching`,
and `202 queued` is emitted without a POSIX (or Windows) runner ever existing. That is
precisely the behaviour `#679` reports, and exactly what Spec 133 §13.3 forbids.

## 2. Normative requirements being satisfied

| Spec | Requirement | Current state |
|---|---|---|
| §7 | Daemon → protected local protocol → Session Runner → adapter/backend | runner exists, never launched |
| §13.1 | Per-run owned process tree; POSIX process group/session; Windows Job Objects | not implemented daemon-side |
| §13.2 | Launch from a typed `LaunchManifest`, never a shell string | manifest type exists, unused |
| §13.3 | Launcher defect prevention; no silent hang; LowMem resolved before spawn | not wired |
| §14.4 | `windows_job_conpty` must exist and pass runtime tests before Windows is advertised; until then negotiate `unsupported`, never silently fall back | descriptor only |

## 3. Architecture of the fix

1. `RunnerSupervisor` (new, `focusa-core::silent_sessions::runner_supervisor`)
   - resolves the sibling executable (`focusa-session-runner[.exe]`) next to the
     running daemon, with an explicit override for packaged installs;
   - builds a typed `LaunchManifest` (executable, argv, cwd, safe env,
     secret refs, stdio modes, backend id, resource policy);
   - launches with exclusive, single-owner semantics:
     - Unix: `setsid`/process group, pgid recorded, signal by group;
     - Windows: Job Object with kill-on-close, ConPTY when interactive;
   - supervises: heartbeat, restart budget (§13.6), orphan adoption (§13.7),
     reboot recovery (§13.8).
2. `RunnerTransport` trait (new) with two implementations:
   - `UnixSocketTransport` (existing protocol, `cfg(unix)`);
   - `NamedPipeTransport` (`cfg(windows)`, `tokio::net::windows::named_pipe`),
     same framing and nonce semantics.
   `runner_client.rs` becomes transport-agnostic; no `UnsupportedPlatform` path on
   a supported OS.
3. `SilentSessionDispatcher` (new): the one place that advances
   `Queued → Launching → Running`, delivers input, and records delivery receipts.
   It is invoked from daemon bootstrap (tick) and from the request path, so an
   accepted request is proven delivered or truthfully failed.
4. Capability negotiation: `GET /v1/silent-sessions/capabilities` reports each
   backend as `supported | unsupported(reason)` per host. Windows reports
   `windows_job_conpty: supported` only after its runtime suite passes; otherwise
   `unsupported` per §14.4 — never a silent untracked process.

## 4. Surface matrix (all surfaces exercise the same dispatcher)

| Surface | Entry | Must prove |
|---|---|---|
| HTTP | `POST /v1/silent-sessions/{id}/start|input|restart` | real PID + delivery receipt, not `queued` alone |
| CLI | `focusa silent-session start/input/observe` | same receipt, truthful status |
| Pi tool | `focusa_silent_sessions` (start/input/preflight) | envelope parity, #524 preflight fixed |
| Menubar / canvas | capability + status projection | reports negotiated backend, no false `running` |
| Remote authorized client | same routes under scope | identical authority + evidence |

## 5. OS matrix and proof

| OS | Transport | Tree ownership | Runtime proof |
|---|---|---|---|
| Linux | Unix socket | process group + optional cgroup v2 | installed-daemon E2E (below) |
| macOS | Unix socket | process group | CI runner E2E |
| Windows | named pipe | Job Object + ConPTY | Windows CI E2E before advertising support |

Proof required for closure (Spec 133 §13 + #679 acceptance): real installed
start, input delivery, cold resume, no dual writer, negative cases (denied start,
dead runner, wrong owner), and consumer parity across HTTP, CLI and Pi. Source and
static checks alone are explicitly insufficient.

## 6. Phases

- **P1** transport trait + client parity (compiles on all targets, unit-tested).
- **P2** `RunnerSupervisor` + manifest launch + supervision on Unix; wire dispatcher.
- **P3** surface parity (HTTP/CLI/Pi) + capability negotiation.
- **P4** Windows named-pipe transport + Job/ConPTY backend + Windows CI suite.
- **P5** installed-daemon E2E proof, then close #679 with evidence.

Phases P1–P3 are verifiable on this host. P4 requires a Windows runner; until it
passes, Windows stays `unsupported` by negotiation, which is the spec's own rule.

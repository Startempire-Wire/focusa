# `focusa_silent_sessions`

**Family:** `work_loop`  
**Label:** Focusa Silent Sessions

## Purpose

List, start, reopen, tail, send input to, or safely kill tmux-backed Focusa SilentSessions running in the background.

## Why it exists

Operators need a `focusa_` tool surface to see background autonomous coding sessions, reopen them with tmux, and stop them when needed without manually remembering tmux commands.

## Actions

- `list` — show all `focusa-silent-*` tmux sessions with attached/window/created/activity metadata.
- `start` — create a detached tmux session with a Focusa-governed Pi command; requires `approved=true`.
- `reopen` — return `tmux attach -t <session>`, `tmux attach -d -t <session>` for detach-others recovery, and recent pane output.
- `tail` — capture recent pane output using `capture-pane -p -J` so wrapped lines are readable.
- `health` — read pane metadata plus tmux activity/log mtime to classify the session as `running`, `stale`, `degraded`, `dead`, or `unknown`.
- `send` — send literal operator steering text to the session and press Enter; requires `approved=true`.
- `interrupt` — send `C-c` to the active pane for a hung/runaway agent; requires `approved=true`.
- `restart` — kill the existing named tmux session if present and start it again from stored metadata (`root_dir`, `command`, `mission`, `work_item_id`, `run_as_user`) unless caller supplies overrides; requires `approved=true`.
- `kill` — terminate the tmux session; requires `approved=true` and `force=true`.

## Tmux control model

SilentSessions intentionally use a small, memorable tmux subset inspired by common tmux cheat-sheet operations:
Reference reviewed: https://tmuxcheatsheet.com/

- `tmux list-sessions` for inventory.
- `tmux new-session -d -s <name> -n agent -c <root>` for detached background work with a stable window name.
- `tmux attach -t <name>` to reopen.
- `tmux attach -d -t <name>` when the operator wants to detach other clients and take over the session.
- `tmux capture-pane -p -J -S -<lines>` for readable tails.
- `tmux list-panes -F ...` for read-only health/pane metadata.
- `stat -c '%U:%G:%u' <root_dir>` before `start` to detect the target project-root owner.
- `as-user <owner> 'tmux ...'` when Pi is root and the project root belongs to a non-root owner, so background sessions run as the project owner instead of creating root-owned project files.
- `tmux pipe-pane -o` to persist pane output to `/tmp/focusa-silent-<session>-<run_as_user>.log` for unattended audit/recovery.
- Before enabling `pipe-pane`, logs rotate at 5 MiB with three backups (`.1` through `.3`) so repeated starts/restarts do not append forever.
- `/tmp/focusa-silent-<session>.json` stores best-effort per-session metadata (`root_dir`, `root_owner`, `run_as_user`, `permission_posture`, `command`, `mission`, `work_item_id`, `log_path`, `log_max_bytes`, `log_backups`) so later `list`, `tail`, `health`, `send`, `interrupt`, `restart`, and `kill` use the same execution identity and restart contract.
- `/tmp/focusa-silent-registry.json` stores a bounded registry of SilentSession metadata keyed by normalized session name. `list`, `reopen`, `tail`, `health`, `start`, `restart`, and `kill` surface `registry`/`registry_metadata` so recovery does not depend on reconstructing flags from transcript memory.
- `tmux send-keys -l -- <text>` followed by `Enter` for literal steering input.
- `tmux send-keys C-c` for approved interruption without destroying the session.
- `tmux kill-session -t <name>` only for explicit stop/kill/restart.

## Safety

`kill`, `send`, `interrupt`, and `restart` are process-control actions and require explicit approval flags. `start` also requires approval because it creates a background process. `reopen` and `tail` are read-only; they return exact tmux commands because tool calls cannot take over the operator terminal interactively.

## Permission posture

SilentSessions are not bound to one hardcoded user. On `start`/`restart`, the tool resolves `root_dir`, detects the filesystem owner, and if Pi is running as root in a non-root-owned project tree, starts tmux through `as-user <owner>`. Structured results include `root_owner`, `run_as_user`, `permission_posture`, `log_path`, and `ownership_warning` when a root-run session under `/home` could create root-owned files.

## LowMem posture

Default `start` activates LowMem via `/v1/resource/mode` before launching the agent command. Public Focusa tools stay callable; LowMem changes fidelity and budgets only.

## Expected result

The tool returns a visible text summary plus structured details for session names, tmux attach commands, detach-others attach commands, captured tail output, tmux version, session metadata, persistent `log_path`, `log_rotated`, `log_max_bytes`, `log_backups`, `log_stats`, `activity_age_seconds`, `stale_after_seconds`, `root_owner`, `run_as_user`, `permission_posture`, `registry`, `registry_metadata`, mutation approval posture, `evidence_capture_suggestion` for copy-ready proof capture, and recovery hints. Failure responses should include `failure_class`, `status`, `canonical/degraded` when applicable, `retry` posture, side effects, and next tools so agents can recover without guessing. Tmux process-control failures use `failure_class=process_control_failed` with list/health/tail recovery instead of ambiguous retry.

## Examples

```text
focusa_silent_sessions action="list"
focusa_silent_sessions action="start" session_name="focusa-c7e1" work_item_id="focusa-c7e1.2" approved=true
focusa_silent_sessions action="reopen" session_name="focusa-c7e1"
focusa_silent_sessions action="tail" session_name="focusa-c7e1" lines=120
focusa_silent_sessions action="health" session_name="focusa-c7e1"
focusa_silent_sessions action="send" session_name="focusa-c7e1" command="Steer: prioritize failing validation first" approved=true
focusa_silent_sessions action="interrupt" session_name="focusa-c7e1" approved=true
focusa_silent_sessions action="restart" session_name="focusa-c7e1" approved=true
focusa_silent_sessions action="kill" session_name="focusa-c7e1" approved=true force=true
```

## Contract summary

- Family: Work Loop.
- Side effects: `process_control`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: none; local/Pi-only surface.
- CLI commands: `stat`, `as-user`, `tmux list-sessions`, `tmux new-session`, `tmux attach-session`, `tmux capture-pane`, `tmux list-panes`, `tmux pipe-pane`, `tmux send-keys`, `tmux kill-session`
- Parity: `pi_only`; exemptions: `pi_only`.
- Core surface: Pi-local tmux SilentSession controller.
- Live check: contract_static plus optional tmux list-sessions/stat owner probe; kill/send/start require explicit approval flags.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`; wraps `tmux` local commands.

## Durable observability and operator-control contract (Spec 132 D7)

This contract is the binding seam for the daemon-independent compatibility
surface. A foreground Pi process is never the liveness authority: a
`SilentSession` continues independently and its records remain readable after
Pi restarts.

### Stable identity and scope

Every session has a stable `session_id` (ULID/UUID), a normalized display name,
`project_root`, `continuity_id`, optional `work_item_id`, and an owner/scope
record. A restart creates a new `run_id` under the same `session_id`; it does
not silently create a new logical session. Requests must carry the session
identity or an unambiguous normalized name and must pass the project-root and
operator-authorization checks.

### Versioned envelopes

Successful calls return `focusa.silent_session.<action>.v1` with:

```json
{
  "session_id": "…",
  "run_id": "…",
  "status": "running",
  "project_root": "~/projects/example",
  "continuity_id": "…",
  "cursor": "42",
  "output": [],
  "retention": {"max_bytes": 5242880, "backups": 3},
  "authorization": {"approved": true, "operator_scope": "project"},
  "recovery": {"next_tools": ["focusa_silent_sessions"]}
}
```

`output` is bounded and sanitized. `tail` accepts a cursor and returns the
next cursor; `follow` is a bounded stream that terminates on completion,
failure, cancellation, or disconnect. Control calls return `accepted`,
`performed`, `run_id`, and an auditable event reference rather than claiming
that a process changed when the control command only queued input.

Failure envelopes use `tool_result_v1` and include `failure_class`, `status`,
`retry`, `side_effects`, `evidence_refs`, `next_tools`, and one actionable
recovery hint. They never expose raw command lines, credentials, ANSI/OSC
sequences, or unbounded npm/agent output.

### Lifecycle and retention

The only canonical lifecycle values are `starting`, `running`,
`waiting-input`, `blocked`, `completed`, `failed`, and `dead`. `stale` and
`degraded` are diagnostic annotations, not replacement statuses. A durable
registry record, append-only event log, bounded output log, and checkpoint
cursor are written under the Focusa runtime home (not `/tmp`); logs rotate at
5 MiB with three backups and rotation is itself an auditable event. Restart
recovery reopens the same `session_id`, selects the latest run, and reports
missing/corrupt records instead of silently reconstructing state.

### Operator controls and safety

`list`, `health`, `tail`, and `follow` are read-only. `start`, `send`,
`interrupt`, `restart`, and `kill` require explicit approval bound to the
project/workpoint scope; `kill` additionally requires force and records the
reason. `reopen`/attach is read-only planning and never steals a terminal
without an explicit detach-others choice. Authorization failure, a trust
prompt, shell-quoting failure, LowMem activation failure, or daemon error is a
visible blocked/warning result and must not silently terminate the worker.

The effective start configuration is retained in a typed, redacted record:
`root_dir`, `project_root`, `continuity_id`, `work_item_id`, provider/model,
thinking mode, command arguments, LowMem request/result, run-as user, resource
limits, and operator approval. Commands are passed as argument vectors or
literal tmux input; nested shell interpolation is prohibited. Recovery is
explicit: inspect `health`, then `tail` from the last cursor, then `reopen`,
`restart`, `interrupt`, or `kill` according to the reported status and
approval posture.

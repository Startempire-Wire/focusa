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
- `health` — read pane metadata with `tmux list-panes` to classify the session as `running`, `degraded`, `dead`, or `unknown`.
- `send` — send literal operator steering text to the session and press Enter; requires `approved=true`.
- `interrupt` — send `C-c` to the active pane for a hung/runaway agent; requires `approved=true`.
- `restart` — kill the existing named tmux session if present and start it again with the same Focusa-governed defaults or supplied command; requires `approved=true`.
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
- `/tmp/focusa-silent-<session>.json` stores best-effort session metadata (`root_dir`, `root_owner`, `run_as_user`, `permission_posture`, `log_path`) so later `list`, `tail`, `health`, `send`, `interrupt`, and `kill` use the same execution identity.
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

The tool returns a visible text summary plus structured details for session names, tmux attach commands, detach-others attach commands, captured tail output, tmux version, session metadata, persistent `log_path`, `root_owner`, `run_as_user`, `permission_posture`, mutation approval posture, and recovery hints. Failure responses should include `failure_class`, `status`, `canonical/degraded` when applicable, `retry` posture, side effects, and next tools so agents can recover without guessing.

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

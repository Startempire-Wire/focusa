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
- `send` — send literal operator steering text to the session and press Enter; requires `approved=true`.
- `kill` — terminate the tmux session; requires `approved=true` and `force=true`.

## Tmux control model

SilentSessions intentionally use a small, memorable tmux subset inspired by common tmux cheat-sheet operations:
Reference reviewed: https://tmuxcheatsheet.com/

- `tmux list-sessions` for inventory.
- `tmux new-session -d -s <name> -n agent -c <root>` for detached background work with a stable window name.
- `tmux attach -t <name>` to reopen.
- `tmux attach -d -t <name>` when the operator wants to detach other clients and take over the session.
- `tmux capture-pane -p -J -S -<lines>` for readable tails.
- `tmux send-keys -l -- <text>` followed by `Enter` for literal steering input.
- `tmux kill-session -t <name>` only for explicit stop/kill.

## Safety

`kill` and `send` are process-control actions and require explicit approval flags. `start` also requires approval because it creates a background process. `reopen` and `tail` are read-only; they return exact tmux commands because tool calls cannot take over the operator terminal interactively.

## LowMem posture

Default `start` activates LowMem via `/v1/resource/mode` before launching the agent command. Public Focusa tools stay callable; LowMem changes fidelity and budgets only.

## Expected result

The tool returns a visible text summary plus structured details for session names, tmux attach commands, detach-others attach commands, captured tail output, tmux version, session metadata, mutation approval posture, and recovery hints. Failure responses should include `failure_class`, `status`, `canonical/degraded` when applicable, `retry` posture, side effects, and next tools so agents can recover without guessing.

## Examples

```text
focusa_silent_sessions action="list"
focusa_silent_sessions action="start" session_name="focusa-c7e1" work_item_id="focusa-c7e1.2" approved=true
focusa_silent_sessions action="reopen" session_name="focusa-c7e1"
focusa_silent_sessions action="tail" session_name="focusa-c7e1" lines=120
focusa_silent_sessions action="send" session_name="focusa-c7e1" command="Steer: prioritize failing validation first" approved=true
focusa_silent_sessions action="kill" session_name="focusa-c7e1" approved=true force=true
```

## Contract summary

- Family: Work Loop.
- Side effects: `process_control`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: none; local/Pi-only surface.
- CLI commands: `tmux list-sessions`, `tmux new-session`, `tmux attach-session`, `tmux capture-pane`, `tmux send-keys`, `tmux kill-session`
- Parity: `pi_only`; exemptions: `pi_only`.
- Core surface: Pi-local tmux SilentSession controller.
- Live check: contract_static plus optional tmux list-sessions probe; kill/send/start require explicit approval flags.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`; wraps `tmux` local commands.

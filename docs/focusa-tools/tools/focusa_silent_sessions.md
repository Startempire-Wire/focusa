# `focusa_silent_sessions`

**Family:** `work_loop`  
**Label:** Focusa Silent Sessions

## Purpose

List, start, reopen, tail, send input to, or safely kill tmux-backed Focusa SilentSessions running in the background.

## Why it exists

Operators need a `focusa_` tool surface to see background autonomous coding sessions, reopen them with tmux, and stop them when needed without manually remembering tmux commands.

## Actions

- `list` — show all `focusa-silent-*` tmux sessions.
- `start` — create a detached tmux session with a Focusa-governed Pi command; requires `approved=true`.
- `reopen` — return `tmux attach -t <session>` and recent pane output.
- `tail` — capture recent pane output.
- `send` — send a command line to the session; requires `approved=true`.
- `kill` — terminate the tmux session; requires `approved=true` and `force=true`.

## Safety

`kill` and `send` are process-control actions and require explicit approval flags. `reopen` does not mutate the session; it returns the exact attach command because tool calls cannot take over the operator terminal interactively.

## LowMem posture

Default `start` activates LowMem via `/v1/resource/mode` before launching the agent command. Public Focusa tools stay callable; LowMem changes fidelity and budgets only.

## Expected result

The tool returns a visible text summary plus structured details for session names, tmux attach commands, captured tail output, mutation approval posture, and recovery hints. Failure responses should include `failure_class`, `status`, `canonical/degraded` when applicable, `retry` posture, side effects, and next tools so agents can recover without guessing.

## Examples

```text
focusa_silent_sessions action="list"
focusa_silent_sessions action="start" session_name="focusa-c7e1" work_item_id="focusa-c7e1.2" approved=true
focusa_silent_sessions action="reopen" session_name="focusa-c7e1"
focusa_silent_sessions action="tail" session_name="focusa-c7e1" lines=120
focusa_silent_sessions action="kill" session_name="focusa-c7e1" approved=true force=true
```

## Source

Defined in `apps/pi-extension/src/tools.ts`; wraps `tmux` local commands.

# `focusa_silent_sessions`

Legacy/non-durable tmux compatibility wrapper for listing, starting, reopening, tailing, sending input to, or safely killing Pi-local Focusa SilentSessions. It is not the canonical Spec133 daemon-native control plane. Use it when Legacy/non-durable tmux compatibility wrapper for explicitly managing Pi-local background SilentSessions; not the canonical Spec133 daemon-native control plane. Default launcher requires explicit model and bounded timeout validation before command execution. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Legacy/non-durable tmux compatibility wrapper for explicitly managing Pi-local background SilentSessions; not the canonical Spec133 daemon-native control plane. Default launcher requires explicit model and bounded timeout validation before command execution.
- Capability family: `work_loop`; namespace: `focusa.work_loop`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `action` (optional; string | string | string | string | string | string | string | string | string): SilentSession action. list is default; kill/send/start/interrupt/restart require approved=true.
- `session_name` (optional; string): SilentSession name or suffix. Names are normalized under focusa-silent-* prefix.
- `root_dir` (optional; string): Working directory for a new SilentSession; defaults to current Pi cwd.
- `command` (optional; string): Custom shell command for start or input line for send. Omit for default Focusa-governed Pi autopilot command.
- `model` (optional; string): LLM model identifier. Required when using the default start command because implicit fallback is disabled.
- `timeout_seconds` (optional; integer; min=30, max=3600): Runtime timeout in seconds for the default start command.
- `mission` (optional; string): Mission prompt for default start command.
- `work_item_id` (optional; string): Optional bead/work item id to anchor the SilentSession.
- `lowmem` (optional; boolean): Activate LowMem at start; default true.
- `lines` (optional; number): Tail lines for durable output; default 80, max 400.
- `cursor` (optional; string): Byte cursor returned by a prior tail/follow call.
- `approved` (optional; boolean): Required true for start/send/kill because those mutate background process state.
- `force` (optional; boolean): Required true with approved=true to kill a SilentSession.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_silent_sessions`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_silent_sessions.md

## Anti-examples

- control mutations without writer/preflight authority
- fresh direct questions that do not continue work

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `process_control`, `process_control`
- Read-only: `false`; destructive: `false`; idempotent: `true`; open-world: `false`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_work_loop_status` (likely_next)
- `focusa_work_loop_checkpoint` (likely_next)
- `focusa_resource_mode` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_work_loop_status`, `focusa_work_loop_checkpoint`, `focusa_resource_mode`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Runbooks: `runbook:work_loop`
- Pi: `focusa_silent_sessions`; MCP: `focusa.silent.sessions`; OpenAI: `focusa_silent_sessions`.
- CLI: `tmux list-sessions`, `tmux new-session`, `tmux attach-session`, `tmux capture-pane`, `tmux list-panes`, `tmux pipe-pane`, `tmux send-keys`, `tmux send-keys C-c`, `tmux kill-session`.
- REST: Pi-local only.
- Specification: contract registry.
- Descriptor digest: `sha256:cc5070d4611a41b9d6b21cf348311d2029a489ed22f91bca8a845f3315e3117a`.

# `focusa_work_loop_control`

**Family:** `work-loop`  
**Label:** Work Loop Control

## Purpose

Control continuous work loop: on, pause, resume, stop.

## When to use

Use `focusa_work_loop_control` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_work_loop_control` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_work_loop_control action="pause" preflight=true reason="operator requested release check"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_work_loop_writer_status`](./focusa_work_loop_writer_status.md)
- [`focusa_work_loop_status`](./focusa_work_loop_status.md)
- [`focusa_work_loop_context`](./focusa_work_loop_context.md)
- [`focusa_work_loop_checkpoint`](./focusa_work_loop_checkpoint.md)
- [`focusa_work_loop_select_next`](./focusa_work_loop_select_next.md)

## Contract summary

- Family: Work Loop.
- Side effects: `control_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/work-loop/enable`, `POST /v1/work-loop/pause`, `POST /v1/work-loop/resume`, `POST /v1/work-loop/stop`
- CLI commands: none.
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Work-loop state/writer controller.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

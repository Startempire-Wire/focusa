# `focusa_work_loop_select_next`

**Family:** `work-loop`  
**Label:** Work Loop Select Next

## Purpose

Ask daemon to defer blocked work and select next ready work item.

## When to use

Use `focusa_work_loop_select_next` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_work_loop_select_next` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_work_loop_select_next parent_work_item_id="focusa-kgd1"
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
- [`focusa_work_loop_control`](./focusa_work_loop_control.md)
- [`focusa_work_loop_context`](./focusa_work_loop_context.md)
- [`focusa_work_loop_checkpoint`](./focusa_work_loop_checkpoint.md)

## Contract summary

- Family: Work Loop.
- Side effects: `select_next_work`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/work-loop/select-next`
- CLI commands: none.
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Work-loop state/writer controller.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

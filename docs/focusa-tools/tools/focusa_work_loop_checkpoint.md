# `focusa_work_loop_checkpoint`

**Family:** `work-loop`  
**Label:** Work Loop Checkpoint

## Purpose

Create a manual continuous-loop checkpoint.

## When to use

Use `focusa_work_loop_checkpoint` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_work_loop_checkpoint` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_work_loop_checkpoint summary="Tool docs generated; validation pending."
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_work_loop_writer_status`](./focusa_work_loop_writer_status.md)
- [`focusa_work_loop_status`](./focusa_work_loop_status.md)
- [`focusa_work_loop_control`](./focusa_work_loop_control.md)
- [`focusa_work_loop_context`](./focusa_work_loop_context.md)
- [`focusa_work_loop_select_next`](./focusa_work_loop_select_next.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

# `focusa_workpoint_resume`

**Family:** `workpoint`  
**Label:** Workpoint Resume

## Purpose

Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action.

## When to use

Use `focusa_workpoint_resume` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_workpoint_resume` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_workpoint_resume mode="operator_summary"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_workpoint_checkpoint`](./focusa_workpoint_checkpoint.md)
- [`focusa_workpoint_link_evidence`](./focusa_workpoint_link_evidence.md)
- [`focusa_active_object_resolve`](./focusa_active_object_resolve.md)
- [`focusa_evidence_capture`](./focusa_evidence_capture.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

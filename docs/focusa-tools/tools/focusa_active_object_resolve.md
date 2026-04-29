# `focusa_active_object_resolve`

**Family:** `workpoint`  
**Label:** Focusa Active Object Resolve

## Purpose

Resolve likely active object references from the current Workpoint and optional hint without inventing canonical refs.

## When to use

Use `focusa_active_object_resolve` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_active_object_resolve` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_active_object_resolve hint="apps/pi-extension/src/tools.ts"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_workpoint_checkpoint`](./focusa_workpoint_checkpoint.md)
- [`focusa_workpoint_resume`](./focusa_workpoint_resume.md)
- [`focusa_workpoint_link_evidence`](./focusa_workpoint_link_evidence.md)
- [`focusa_evidence_capture`](./focusa_evidence_capture.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

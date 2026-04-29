# `focusa_metacog_reflect`

**Family:** `metacognition`  
**Label:** Metacog Reflect

## Purpose

Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes.

## When to use

Use `focusa_metacog_reflect` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_metacog_reflect` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_metacog_reflect turn_range="last_20" failure_classes=["instruction_following"]
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_metacog_capture`](./focusa_metacog_capture.md)
- [`focusa_metacog_retrieve`](./focusa_metacog_retrieve.md)
- [`focusa_metacog_plan_adjust`](./focusa_metacog_plan_adjust.md)
- [`focusa_metacog_evaluate_outcome`](./focusa_metacog_evaluate_outcome.md)
- [`focusa_metacog_recent_reflections`](./focusa_metacog_recent_reflections.md)
- [`focusa_metacog_recent_adjustments`](./focusa_metacog_recent_adjustments.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

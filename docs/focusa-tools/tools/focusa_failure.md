# `focusa_failure`

**Family:** `focus-state`  
**Label:** Record Failure

## Purpose

Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars.

## When to use

Use `focusa_failure` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_failure` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_failure failure="Skill loader rejected Focusa skill: SKILL.md lacked required description frontmatter." recovery="Added YAML name/description to all Focusa skill copies."
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_scratch`](./focusa_scratch.md)
- [`focusa_decide`](./focusa_decide.md)
- [`focusa_constraint`](./focusa_constraint.md)
- [`focusa_intent`](./focusa_intent.md)
- [`focusa_current_focus`](./focusa_current_focus.md)
- [`focusa_next_step`](./focusa_next_step.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

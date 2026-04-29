# `focusa_tree_head`

**Family:** `tree-lineage`  
**Label:** Tree Head

## Purpose

Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work.

## When to use

Use `focusa_tree_head` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_tree_head` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_tree_head
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_tree_path`](./focusa_tree_path.md)
- [`focusa_tree_snapshot_state`](./focusa_tree_snapshot_state.md)
- [`focusa_tree_restore_state`](./focusa_tree_restore_state.md)
- [`focusa_tree_diff_context`](./focusa_tree_diff_context.md)
- [`focusa_tree_recent_snapshots`](./focusa_tree_recent_snapshots.md)
- [`focusa_tree_snapshot_compare_latest`](./focusa_tree_snapshot_compare_latest.md)

## Source

Defined in `apps/pi-extension/src/tools.ts`.

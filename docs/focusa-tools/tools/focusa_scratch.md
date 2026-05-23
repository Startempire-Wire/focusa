# `focusa_scratch`

**Family:** `focus-state`  
**Label:** Scratchpad

## Purpose

Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done.

## When to use

Use `focusa_scratch` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_scratch` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_scratch tag="reasoning" note="Need to compare README against released runtime before editing."
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_decide`](./focusa_decide.md)
- [`focusa_constraint`](./focusa_constraint.md)
- [`focusa_failure`](./focusa_failure.md)
- [`focusa_intent`](./focusa_intent.md)
- [`focusa_current_focus`](./focusa_current_focus.md)
- [`focusa_next_step`](./focusa_next_step.md)

## Contract summary

- Family: Focus State.
- Side effects: `local_note`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: none; local/Pi-only surface.
- CLI commands: none.
- Parity: `local_only`; exemptions: `local_scratchpad_only`.
- Core surface: FocusState reducer/update.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

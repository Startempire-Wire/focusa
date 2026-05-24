# `focusa_state_hygiene_plan`

**Family:** `diagnostics-hygiene`  
**Label:** Focus State Hygiene Plan

## Purpose

Create a proposal-style hygiene plan; does not mutate Focus State.

## When to use

Use `focusa_state_hygiene_plan` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_state_hygiene_plan` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_state_hygiene_plan reason="old next steps may be stale after release"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

`details.plan` is proposal-only and includes `exact_duplicate_groups`, `exact_stale_candidates`, `target_frame_id`, `actions`, and `apply_requires_approval=true`. Use it to review precise stale signals before the approval-gated non-destructive apply note.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_tool_doctor`](./focusa_tool_doctor.md)
- [`focusa_state_hygiene_doctor`](./focusa_state_hygiene_doctor.md)
- [`focusa_state_hygiene_apply`](./focusa_state_hygiene_apply.md)

## Contract summary

- Family: Diagnostics / Hygiene.
- Side effects: `read_only`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: none; local/Pi-only surface.
- CLI commands: none.
- Parity: `pi_only`; exemptions: `approval_placeholder`, `domain_cli_only`.
- Core surface: Local diagnostic/hygiene composition.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

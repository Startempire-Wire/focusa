# `focusa_state_hygiene_apply`

**Family:** `diagnostics-hygiene`  
**Label:** Focus State Hygiene Apply

## Purpose

Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed `/v1/focus/update`.

## When to use

Use `focusa_state_hygiene_apply` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_state_hygiene_apply` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_state_hygiene_apply approved=false reason="review hygiene plan first"
focusa_state_hygiene_apply approved=true reason="duplicate review complete"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_tool_doctor`](./focusa_tool_doctor.md)
- [`focusa_state_hygiene_doctor`](./focusa_state_hygiene_doctor.md)
- [`focusa_state_hygiene_plan`](./focusa_state_hygiene_plan.md)

## Contract summary

- Family: Diagnostics / Hygiene.
- Side effects: `write_focus_state_note` when `approved=true`; no deletion.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/focus/update`.
- CLI commands: none.
- Parity: `pi_only`; exemptions: `domain_cli_only`.
- Core surface: Reducer-backed Focus State note append through `/v1/focus/update`.
- Live check: contract_static plus bounded hot-path live checks; approved apply writes an auditable Focus State note.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

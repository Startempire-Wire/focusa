# `focusa_workpoint_link_evidence`

**Family:** `workpoint`  
**Label:** Workpoint Link Evidence

## Purpose

Attach a stable evidence reference or verification result to the active canonical Workpoint.

## Project folder semantics

After compaction/model switch, pass `project_root` and `continuity_id` from the canonical WorkpointResumePacket when Pi's ambient cwd is broad (for example `/root`). The tool builds a `FocusaSessionIdentity` from that explicit project context before trajectory clarity and evidence-link calls.

## When to use

Use `focusa_workpoint_link_evidence` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_workpoint_link_evidence` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance.

## Example usage

```text
focusa_workpoint_link_evidence target_ref="docs/focusa-tools" result="43 one-tool docs generated" evidence_ref="docs/focusa-tools/tools/focusa_workpoint_link_evidence.md" project_root="/home/wirebot/focusa" continuity_id="spec96-lowmem-surgical"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

## Recovery notes

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, call `focusa_workpoint_resume` or a relevant read tool before continuing.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_workpoint_checkpoint`](./focusa_workpoint_checkpoint.md)
- [`focusa_workpoint_resume`](./focusa_workpoint_resume.md)
- [`focusa_active_object_resolve`](./focusa_active_object_resolve.md)
- [`focusa_evidence_capture`](./focusa_evidence_capture.md)

## Contract summary

- Family: Workpoint.
- Side effects: `evidence_link`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/workpoint/evidence/link`
- CLI commands: `focusa workpoint evidence-link`
- Parity: `full`.
- Core surface: Workpoint reducer/state.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.

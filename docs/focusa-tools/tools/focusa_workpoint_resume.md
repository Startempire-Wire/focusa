# `focusa_workpoint_resume`

**Family:** `workpoint`  
**Label:** Workpoint Resume

## Purpose

Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action.

Authority boundary: a canonical Workpoint is immediate continuation authority only when scoped to the exact `project_root + continuity_id`. See [`docs/current/AUTHORITY_MODEL.md`](../../current/AUTHORITY_MODEL.md).

## When to use

Use `focusa_workpoint_resume` when its specific Focusa state or workflow surface is the narrowest tool that matches the current need. Prefer this tool over raw transcript memory when the result should survive compaction, be inspectable, or guide a later agent turn.

## When not to use

Do not use `focusa_workpoint_resume` to dump unbounded logs, bypass operator steering, or create parallel memory outside Focusa. If the tool returns `pending`, `blocked`, `degraded`, or `canonical=false`, treat that as a recovery state and follow the returned next-step guidance. If it returns `status=rejected_current_ask_scope_conflict` or `action_authority_for_current_ask=false`, hard-stop: the packet is not executable next-action authority across the project mismatch.

## Project folder semantics

`project_root` is the project folder/container holding related files, and `continuity_id` is the stable logical session/workstream identity. Broad roots (`/`, `/root`, `/home`, `/tmp`, `/var`, `/usr`, `/opt`) are unsafe and return `rejected_unsafe_project_root`/`scope_mismatch` instead of canonical packets. Cross-project packets return `rejected_scope_mismatch`; same-root/different-continuity packets return `rejected_continuity_mismatch`. When a `FocusaSessionIdentity` envelope is supplied, its `project_root`, `continuity_id`, `session_frame_key`, and ProjectIdentity are authoritative over flat legacy fields. `session_id` is temporal metadata across compaction, model switch, fork, or process restart. Trajectory/goals/work-item/frame tags can raise `identity_confidence_percent` only after the hard gates match.

## Example usage

```text
focusa_workpoint_resume mode="operator_summary"
```

## Expected result

The tool should return a visible summary plus structured details. For Pi tools, inspect `details.tool_result_v1` when available for `status`, `failure_class`, `canonical`, `degraded`, `retry`, `side_effects`, `evidence_refs`, and `next_tools`.

When no canonical project-bound packet is available, the Pi tool returns `details.recovery_packet` with `authority=operator_and_current_project_context`, `safe_next_action`, `next_tools`, and `do_not_use` guidance. Treat this as a checkpoint-first recovery contract, not as continuation truth.

## Recovery notes

- `failure_class=hot_path_timeout` or `status=timeout_preserved`: the Pi tool may preserve a noncanonical fallback packet even without a local packet; retry after `focusa_tool_doctor`/`focusa_resource_mode`, or checkpoint the current mission before trusting state.

- If Focusa is unavailable, run `focusa_tool_doctor` or check `/v1/health`.
- If the result is non-canonical/degraded, follow `details.recovery_packet.safe_next_action`: create a fresh `focusa_workpoint_checkpoint` from the current operator ask/project state before treating continuation as canonical.
- If writer ownership is involved, call `focusa_work_loop_writer_status` or use work-loop preflight first.

## Related tools

- [`focusa_workpoint_checkpoint`](./focusa_workpoint_checkpoint.md)
- [`focusa_workpoint_link_evidence`](./focusa_workpoint_link_evidence.md)
- [`focusa_active_object_resolve`](./focusa_active_object_resolve.md)
- [`focusa_evidence_capture`](./focusa_evidence_capture.md)

## Contract summary

- Family: Workpoint.
- Side effects: `read_only`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/workpoint/resume`
- CLI commands: `focusa workpoint resume`
- Parity: `full`.
- Core surface: Workpoint reducer/state.
- Live check: contract_static plus bounded hot-path live checks; degraded results remain noncanonical and nonblocking.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Defined in `apps/pi-extension/src/tools.ts`.


## Workpoint Resume Packet v2

`focusa_workpoint_resume` returns `schema_version="focusa.workpoint_resume_packet.v2"` plus `resume_packet_v2` when the daemon can render the structured packet. The v2 packet contains:

- `packet_id`, `generated_at`, `resume_source`, `canonical`, `degraded`, and `confidence`.
- Top-level `project_identity` and `session_identity` so project authority is explicit.
- `rendered_summary` for compact prompt injection.
- Rich `resume_summary` with one-line summary, current action, safest next action, warnings, do-not-use guidance, and context sufficiency.
- `workpoint` continuation data with status, mission, active object refs, blockers, drift boundaries, hooks, evidence refs, and next action.
- `trajectory` with high/mid/low hierarchy; similarity grouping is advisory only.
- `traversal_slices` with tags, window tags, and `rehydrate_refs` for bounded `focusa_traverse` follow-up.
- `tool_affordances.best_next`, `.recovery`, `.do_not_use`, `api_provenance` with freshness/tool-result metadata, `next_tools`, and `failure_class`.

Authority boundary: project/session authority remains safe `project_root + continuity_id`; shared high-level trajectory similarity and broad folders such as `/root` must never merge sessions.

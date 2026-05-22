# focusa_trajectory_view

## Purpose

Read the per-project Trajectory Intelligence view before acting. This is the north-star orientation tool: it shows ProjectIdentity, high/mid/low trajectory goals, desired end state, current verified state, active gap, evidence, drift boundaries, context sufficiency, and next Workpoint candidate.

## When to use

- Start/resume a project session.
- After compaction/model switch/fork.
- Before choosing a next action when goal/state/gap is unclear.
- After operator steering, Workpoint transitions, evidence updates, failures, degradation, or handoff.
- When scope mismatch, stale context, or drift risk is possible.

## Example usage

```json
{
  "project_root": "<focusa-repo>",
  "session_id": "pi-session",
  "continuity_id": "logical-workstream-id",
  "mode": "summary"
}
```

## Expected result

Returns `tool_result_v1` details with `/v1/trajectory/view` response:

- `project_identity.status`, `authority_boundary=project_root_plus_continuity_id`, `continuity_id`, temporal `session_id`, and fingerprint.
- `trajectory.definition_status`.
- high-level, mid-level, and low-level trajectory goals; desired end state; current state; short-term goal; active gap.
- `trajectory.similarity_group` with advisory group keys and `must_not_merge_sessions=true`.
- `trajectory.lifecycle.clarity_gate` and `intelligence_view.clarity_gate` with `clear|provisional|unclear|conflicted` status and `proceed|verify_first|operator_input` guidance.
- evidence refs and blockers.
- `intelligence_view.context_sufficiency`.
- `do_not_use` stale/mismatched refs.
- advisory `next_workpoint_candidate`.
- Pi lifecycle refresh snapshots (`lastTrajectoryClarity`) are updated at session start/resume, compaction, steering, failure/degradation, and fork handoff.

## Recovery notes

- `failure_class=scope_mismatch` or `status=degraded`: verify ProjectIdentity before trusting context.
- Same high-level trajectory similarity is advisory only; distinct mid/low goals or continuity IDs remain separate sessions.
- `definition_status=unclear`: define/confirm goal before proceeding.
- `recommended_action=verify_first`: verify local evidence or Workpoint before acting.
- `recommended_action=operator_input`: ask only for missing trajectory facts, not broad instructions.

## Related tools

- `focusa_workpoint_resume`
- `focusa_active_object_resolve`
- `focusa_evidence_capture`
- `focusa_tool_doctor`

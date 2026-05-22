# focusa_trajectory_propose_workpoint

## Purpose

Turn an active trajectory gap into an advisory Workpoint candidate. The candidate must be explicitly checkpointed through Workpoint tooling before it is canonical, and the proposal itself never selects work-loop items or executes actions.

## When to use

- Project start/resume when trajectory is unclear.
- After operator steering changes project goal/state.
- Before compaction/model switch/handoff.
- Before converting trajectory gap into Workpoint continuation.

## Example usage

```json
{
  "project_root": "<focusa-repo>",
  "session_id": "pi-session",
  "continuity_id": "logical-workstream-id",
  "target_ref": "optional/file/or/object",
  "action_type": "trajectory_gap_followup"
}
```

## Expected result

Returns `tool_result_v1` details backed by the `/v1/trajectory/*` endpoint. The result is project-scoped, bounded, and explicit about `canonical`, `degraded`, `advisory_only`, `no_execution_side_effects`, `workpoint_candidate.action_intent`, target refs, verification hooks, blockers, `do_not_drift`, `checkpoint_required`, and recovery posture.

## Recovery notes

Use `details.tool_result_v1.failure_class` plus status/canonical/degraded fields for recovery decisions.

- Scope mismatch: verify ProjectIdentity before trusting context.
- Advisory candidate: do not treat as canonical Workpoint until `focusa_workpoint_checkpoint` accepts it.
- Blockers present: run `focusa_trajectory_assess` or resolve active objects before checkpointing.
- Work-loop authority: never call work-loop selection/execution from a trajectory proposal.
- Unclear trajectory: use `focusa_trajectory_define_goal` or request only missing goal facts.
- Degraded daemon: fall back to Workpoint resume + Focus Slice, then retry trajectory view.

## Related tools

- `focusa_trajectory_view`
- `focusa_workpoint_resume`
- `focusa_workpoint_checkpoint`
- `focusa_active_object_resolve`

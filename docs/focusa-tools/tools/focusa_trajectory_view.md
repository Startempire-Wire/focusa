# focusa_trajectory_view

## Purpose

Read the per-project Trajectory Intelligence view before acting. This is the north-star orientation tool: it shows ProjectIdentity, HLT/MLG/STG/Waypoints, desired end state, current verified state, active gap, evidence, drift boundaries, context sufficiency, and next Workpoint candidate.

Authority boundary: HLT/MLG/STG/Waypoints orient execution; Workpoint is immediate continuation authority. See [`docs/current/AUTHORITY_MODEL.md`](../../current/AUTHORITY_MODEL.md).

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
  "mode": "summary",
  "allow_prior_project_trajectory": true
}
```

## Expected result

Returns `tool_result_v1` details with `/v1/trajectory/view` response:

- `project_identity.status`, `authority_boundary=project_root_plus_continuity_id`, `continuity_id`, temporal `session_id`, and fingerprint.
- `trajectory.definition_status`.
- high-level, mid-level, and low-level trajectory goals; desired end state; current state; short-term goal; active gap.
- Optional `allow_prior_project_trajectory=true` returns a same-project prior trajectory as an advisory reload fallback when continuity changed; refresh short-term goal/current state as needed.
- When ProjectIdentity is verified but no durable trajectory exists, `trajectory.bootstrap_default=true` returns an explicit noncanonical advisory goal instead of an empty/NOT SET projection; define/confirm the project goal before treating it as canonical.
- `trajectory.similarity_group` with advisory group keys and `must_not_merge_sessions=true`.
- `trajectory.lifecycle.clarity_gate` and `intelligence_view.clarity_gate` with `clear|provisional|unclear|conflicted` status and `proceed|verify_first|operator_input` guidance.
- evidence refs and blockers.
- `intelligence_view.context_sufficiency` with `score`, `proceed_posture`, `missing_facts`, `stale_refs`, and `conflicting_signals`.
- `intelligence_view.relevance_rationale`, `current_state_delta`, `learning_refs`, `prediction_refs`, and `ask_operator_if`.
- `do_not_use` stale/mismatched refs.
- advisory `next_workpoint_candidate`.
- Pi lifecycle refresh snapshots (`lastTrajectoryClarity`) are updated at session start/resume, compaction, steering, failure/degradation, and fork handoff.

## Recovery notes

- Pi tool calls default to `mode=summary` for bounded hot-path orientation.
- The Pi wrapper gives `/trajectory/view` a route-specific 4–5s hot timeout budget before preserving cached advisory clarity.
- `failure_class=hot_path_timeout` or `status=timeout_preserved`: cached clarity can be returned as advisory only; retry after `focusa_resource_mode`/`focusa_tool_doctor` before treating it as current.
- User-facing timeout text may say `preserved cached advisory ...; cause=timeout`; this is a degraded orientation fallback, not a task failure.
- `failure_class=scope_mismatch` or `status=degraded`: verify ProjectIdentity before trusting context.
- Same high-level trajectory similarity is advisory only; distinct mid/low goals or continuity IDs remain separate sessions unless the caller explicitly opts into prior-project reload fallback.
- `trajectory.bootstrap_default=true`: bootstrap projection is advisory only; use `focusa_trajectory_define_goal` or checkpoint the current explicit mission before durable work.
- `definition_status=unclear`: define/confirm goal before proceeding.
- `recommended_action=verify_first`: verify local evidence or Workpoint before acting.
- `recommended_action=operator_input`: ask only for missing trajectory facts, not broad instructions.

## Related tools

- `focusa_workpoint_resume`
- `focusa_active_object_resolve`
- `focusa_evidence_capture`
- `focusa_tool_doctor`

## Contract summary

- Family: Trajectory.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `GET /v1/trajectory/view`
- CLI commands: `focusa trajectory view`
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Spec96 per-project Trajectory Intelligence projection.
- Live check: contract_static plus /v1/trajectory/view safe probe and ProjectIdentity status.
- Contract source: `docs/current/focusa-tool-contracts.json`.

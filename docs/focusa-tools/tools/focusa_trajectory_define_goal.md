# focusa_trajectory_define_goal

## Purpose

Create a project-scoped trajectory goal candidate. It validates long-term goal and desired end state, records provenance and lifecycle status in the response, and does not mutate task/execution authority. Root goal supersession requires operator confirmation or durable supersession evidence.

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
  "long_term_goal": "Stable project goal",
  "desired_end_state": "Evidence-backed desired end state",
  "operator_confirmed": true,
  "required_checks": ["test or live proof command"],
  "not_done_if": ["required checks have not run"]
}
```

## Expected result

Returns `tool_result_v1` details backed by the `/v1/trajectory/*` endpoint. The result is project-scoped, bounded, and explicit about `canonical`, `degraded`, `advisory_only`, `trajectory_candidate.definition_status`, `root_goal_change_allowed`, lifecycle `source_precedence`, `next_tools`, and recovery posture. Persisted trajectories include the Spec96 definition-of-done proof contract: `desired_end_state`, `required_evidence_refs`, `required_checks`, `acceptance_risks`, and `not_done_if`.

## Recovery notes

Use `details.tool_result_v1.failure_class` plus status/canonical/degraded fields for recovery decisions.

- Scope mismatch: verify ProjectIdentity before trusting context.
- Advisory candidate: do not treat as canonical Workpoint until `focusa_workpoint_checkpoint` accepts it.
- Unclear trajectory: use `focusa_trajectory_define_goal` or request only missing goal facts.
- Supersession rejected: provide `operator_confirmed=true` or `supersession_evidence_refs` from durable proof.
- Degraded daemon: fall back to Workpoint resume + Focus Slice, then retry trajectory view.

## Related tools

- `focusa_trajectory_view`
- `focusa_workpoint_resume`
- `focusa_workpoint_checkpoint`
- `focusa_active_object_resolve`

## Contract summary

- Family: Trajectory.
- Side effects: `advisory_projection`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/trajectory/define-goal`
- CLI commands: `focusa trajectory define-goal`
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Spec96 per-project Trajectory Intelligence projection.
- Live check: contract_static plus /v1/trajectory/view safe probe and trajectory endpoint smoke test.
- Contract source: `docs/current/focusa-tool-contracts.json`.

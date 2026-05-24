# focusa_trajectory_resume

## Purpose

Resume per-project trajectory orientation after compaction/model switch/session resume, then combine it with Workpoint resume before acting.

## When to use

- Project start/resume when trajectory is unclear.
- After operator steering changes project goal/state.
- Before compaction/model switch/handoff.
- Before converting trajectory gap into Workpoint continuation.

## Example usage

```json
{
  "project_root": "<focusa-repo>",
  "session_id": "pi-session"
}
```

## Expected result

Returns `tool_result_v1` details backed by the `/v1/trajectory/*` endpoint. The result is project-scoped, bounded, and explicit about `canonical`, `degraded`, `advisory_only`, `next_tools`, and recovery posture.

## Recovery notes

- `failure_class=hot_path_timeout` or `status=timeout_preserved`: the Pi tool preserves a degraded noncanonical fallback candidate/checkpoint/resume packet; use it only as advisory orientation, then retry after `focusa_tool_doctor`/`focusa_resource_mode`.

Use `details.tool_result_v1.failure_class` plus status/canonical/degraded fields for recovery decisions.

- Scope mismatch: verify ProjectIdentity before trusting context.
- Advisory candidate: do not treat as canonical Workpoint until `focusa_workpoint_checkpoint` accepts it.
- Unclear trajectory: use `focusa_trajectory_define_goal` or request only missing goal facts.
- Degraded daemon: fall back to Workpoint resume + Focus Slice, then retry trajectory view.

## Related tools

- `focusa_trajectory_view`
- `focusa_workpoint_resume`
- `focusa_workpoint_checkpoint`
- `focusa_active_object_resolve`

## Contract summary

- Family: Trajectory.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/trajectory/resume`
- CLI commands: `focusa trajectory resume`
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Spec96 per-project Trajectory Intelligence projection.
- Live check: contract_static plus /v1/trajectory/view safe probe and trajectory endpoint smoke test.
- Contract source: `docs/current/focusa-tool-contracts.json`.

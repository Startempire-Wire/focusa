# Focusa Workpoint Runbook

## Resume

1. Verify `project_root + continuity_id`.
2. Call `focusa_trajectory_view` for destination/gap context.
3. Call `focusa_workpoint_resume`; trust canonical packets over transcript tails.
4. If not found, checkpoint before important work.

## Execute and prove

1. Resolve ambiguous objects with `focusa_active_object_resolve`.
2. Keep one bounded current action and explicit drift boundaries.
3. Capture stable evidence with `focusa_evidence_capture` or link existing proof.
4. Checkpoint exact mission, blockers, evidence, and next action before compaction/model switch/release.

## Scope rules

- Workpoint authority is exact project/workstream scope.
- Worktrees are typed working subpaths under the same project authority.
- `canonical=false` is degraded fallback, never completion truth.

## Done condition

Acceptance evidence is linked, blockers are resolved or explicitly deferred, and the next canonical packet can resume without transcript inference.

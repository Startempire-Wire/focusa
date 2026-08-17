# Focusa Core Agent Runbook

## Fast start

1. Detect the intended project with `focusa_project_identity` and verify it with `focusa_project_verify`.
2. When the safe operator-named folder has no valid marker, call `focusa_project_bootstrap` in `preview` mode, apply only with explicit confirmation, then verify identity again.
3. Resume or complete `focusa_project_genesis`, `focusa_trajectory_view`, and `focusa_workpoint_resume`; do not infer authority from transcript memory.
4. Discover the narrowest capability with `focusa_tool_search`, then cold-load its schema with `focusa_tool_describe`.
5. Use `docs/contracts/spec141/generated-capability-v2/pi-tools.json` or `docs/focusa-tools/tools/` for all 115 Focusa Pi tools.
6. Capture stable evidence and checkpoint before compaction, model changes, or release work.

## Authority and recovery

- Project authority is `project_root + continuity_id`; a worktree is a typed working subpath, not a new project.
- Treat `blocked`, `pending`, `degraded`, `canonical=false`, or scope conflict as recovery states.
- Use `focusa_tool_doctor`, `focusa_project_verify`, and `focusa_workpoint_checkpoint` rather than guessing.
- Operator steering always overrides predictions and default sequences.

## Done condition

The requested outcome is proven by stable evidence, linked to the canonical Workpoint, and reflected in Trajectory state without cross-project drift.

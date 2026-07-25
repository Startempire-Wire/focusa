# Focusa Core Agent Runbook

## Fast start

1. Verify the project with `focusa_project_identity` and `focusa_project_verify`.
2. Resume `focusa_trajectory_view` and `focusa_workpoint_resume`; do not infer authority from transcript memory.
3. Discover the narrowest capability with `focusa_tool_search`, then cold-load its schema with `focusa_tool_describe`.
4. Use `docs/contracts/spec141/generated-capability-v2/pi-tools.json` or `docs/focusa-tools/tools/` for all 112 Focusa Pi tools.
5. Capture stable evidence and checkpoint before compaction, model changes, or release work.

## Authority and recovery

- Project authority is `project_root + continuity_id`; a worktree is a typed working subpath, not a new project.
- Treat `blocked`, `pending`, `degraded`, `canonical=false`, or scope conflict as recovery states.
- Use `focusa_tool_doctor`, `focusa_project_verify`, and `focusa_workpoint_checkpoint` rather than guessing.
- Operator steering always overrides predictions and default sequences.

## Done condition

The requested outcome is proven by stable evidence, linked to the canonical Workpoint, and reflected in Trajectory state without cross-project drift.

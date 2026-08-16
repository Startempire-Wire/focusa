# Focusa Work-loop Runbook

## Read path

1. `focusa_work_loop_writer_status`
2. `focusa_work_loop_status`
3. `focusa_workpoint_resume`

## Mutation path

1. Run `focusa_work_loop_control` with `preflight=true` unless the operator explicitly authorized mutation.
2. Preserve writer lease, project scope, root work item, and current ask.
3. Apply pause/resume/stop only through daemon authority.
4. Checkpoint before risky continuation; use `focusa_work_loop_select_next` only for blocked work.

## Silent Sessions

For autonomous execution, load `skill:focusa-silent-sessions`; the daemon owns session/run/generation state and mutation approvals.

## Recovery

- Writer mismatch: pause and reconcile—never steal authority.
- Stale state: use the hygiene doctor; apply only with explicit approval.
- Compaction/transport exhaustion: use canonical Workpoint/session transfer and governed rollover.

## Done condition

One canonical writer, scoped work item, current operator ask, checkpointed next action, and evidence-backed completion state are visible.

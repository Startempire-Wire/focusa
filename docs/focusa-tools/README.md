# Focusa Tool Docs

One doc per current `focusa_*` tool. Family pages are indexes only.

- [Focus State](./focus-state.md)
- [Workpoint](./workpoint.md)
- [Work Loop](./work-loop.md)
- [Metacognition](./metacognition.md)
- [Tree Lineage](./tree-lineage.md)
- [Diagnostics Hygiene](./diagnostics-hygiene.md)
- [Predictive Power](./predictive-power.md)
- [Trajectory](./trajectory.md)
- [Project Identity](./project-identity.md)
- [Stability audit 2026-05-22](./stability-audit-2026-05-22.md)

## Current counts

- 59 Pi tools documented with one page per tool.
- Prediction tools are first-class (`record`, `recent`, `evaluate`, `stats`).
- Tool contracts are validated by `node scripts/validate-focusa-tool-contracts.mjs` and live-proofed by `node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures`.
- Tool result envelopes include `failure_class`; degraded/noncanonical outputs are recovery states, not success; bounded `reflex_suggestions` point to Spec97 recovery primitives when applicable.
- Trajectory tools are first-class per-project north-star tools: view, define_goal, assess, propose_workpoint, checkpoint, resume.
- Spec97 Reflex Primitive docs are public and read-only through [`focusa_reflex_primitives`](tools/focusa_reflex_primitives.md), `GET /v1/reflex/primitives`, and `surface=reflex_primitives` traversal.
- Friendly onboarding and route hints: [`FOCUSA_FRIENDLY_ONBOARDING.md`](../current/FOCUSA_FRIENDLY_ONBOARDING.md) and [`FOCUSA_TOOL_CHOREOGRAPHY_MAP.md`](../current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md).

## All tools

- [`focusa_project_identity`](tools/focusa_project_identity.md)
- [`focusa_project_verify`](tools/focusa_project_verify.md)
- [`focusa_trajectory_view`](tools/focusa_trajectory_view.md)
- [`focusa_trajectory_resume`](tools/focusa_trajectory_resume.md)
- [`focusa_trajectory_checkpoint`](tools/focusa_trajectory_checkpoint.md)
- [`focusa_trajectory_propose_workpoint`](tools/focusa_trajectory_propose_workpoint.md)
- [`focusa_trajectory_assess`](tools/focusa_trajectory_assess.md)
- [`focusa_trajectory_define_goal`](tools/focusa_trajectory_define_goal.md)
- [`focusa_scratch`](tools/focusa_scratch.md)
- [`focusa_decide`](tools/focusa_decide.md)
- [`focusa_constraint`](tools/focusa_constraint.md)
- [`focusa_failure`](tools/focusa_failure.md)
- [`focusa_intent`](tools/focusa_intent.md)
- [`focusa_current_focus`](tools/focusa_current_focus.md)
- [`focusa_next_step`](tools/focusa_next_step.md)
- [`focusa_open_question`](tools/focusa_open_question.md)
- [`focusa_recent_result`](tools/focusa_recent_result.md)
- [`focusa_note`](tools/focusa_note.md)
- [`focusa_work_loop_writer_status`](tools/focusa_work_loop_writer_status.md)
- [`focusa_work_loop_status`](tools/focusa_work_loop_status.md)
- [`focusa_work_loop_control`](tools/focusa_work_loop_control.md)
- [`focusa_work_loop_context`](tools/focusa_work_loop_context.md)
- [`focusa_work_loop_checkpoint`](tools/focusa_work_loop_checkpoint.md)
- [`focusa_work_loop_select_next`](tools/focusa_work_loop_select_next.md)
- [`focusa_state_hygiene_doctor`](tools/focusa_state_hygiene_doctor.md)
- [`focusa_state_hygiene_plan`](tools/focusa_state_hygiene_plan.md)
- [`focusa_state_hygiene_apply`](tools/focusa_state_hygiene_apply.md)
- [`focusa_tool_doctor`](tools/focusa_tool_doctor.md)
- [`focusa_active_object_resolve`](tools/focusa_active_object_resolve.md)
- [`focusa_evidence_capture`](tools/focusa_evidence_capture.md)
- [`focusa_workpoint_checkpoint`](tools/focusa_workpoint_checkpoint.md)
- [`focusa_workpoint_link_evidence`](tools/focusa_workpoint_link_evidence.md)
- [`focusa_workpoint_resume`](tools/focusa_workpoint_resume.md)
- [`focusa_tree_head`](tools/focusa_tree_head.md)
- [`focusa_tree_path`](tools/focusa_tree_path.md)
- [`focusa_tree_snapshot_state`](tools/focusa_tree_snapshot_state.md)
- [`focusa_tree_restore_state`](tools/focusa_tree_restore_state.md)
- [`focusa_tree_diff_context`](tools/focusa_tree_diff_context.md)
- [`focusa_metacog_capture`](tools/focusa_metacog_capture.md)
- [`focusa_metacog_retrieve`](tools/focusa_metacog_retrieve.md)
- [`focusa_metacog_reflect`](tools/focusa_metacog_reflect.md)
- [`focusa_metacog_plan_adjust`](tools/focusa_metacog_plan_adjust.md)
- [`focusa_metacog_evaluate_outcome`](tools/focusa_metacog_evaluate_outcome.md)
- [`focusa_tree_recent_snapshots`](tools/focusa_tree_recent_snapshots.md)
- [`focusa_tree_snapshot_compare_latest`](tools/focusa_tree_snapshot_compare_latest.md)
- [`focusa_metacog_recent_reflections`](tools/focusa_metacog_recent_reflections.md)
- [`focusa_metacog_recent_adjustments`](tools/focusa_metacog_recent_adjustments.md)
- [`focusa_metacog_loop_run`](tools/focusa_metacog_loop_run.md)
- [`focusa_metacog_doctor`](tools/focusa_metacog_doctor.md)
- [`focusa_lineage_tree`](tools/focusa_lineage_tree.md)
- [`focusa_li_tree_extract`](tools/focusa_li_tree_extract.md)
- [`focusa_predict_record`](tools/focusa_predict_record.md)
- [`focusa_predict_recent`](tools/focusa_predict_recent.md)
- [`focusa_predict_evaluate`](tools/focusa_predict_evaluate.md)
- [`focusa_predict_stats`](tools/focusa_predict_stats.md)

- [`focusa_resource_mode`](tools/focusa_resource_mode.md) — read/control ResourceMode and LowMem activation.
- [`focusa_traverse`](tools/focusa_traverse.md) — read-only bounded traversal across large Focusa surfaces.
- [`focusa_reflex_primitives`](tools/focusa_reflex_primitives.md) — read-only bounded Spec97 Reflex Primitive summaries. Contract doc path: `docs/focusa-tools/tools/focusa_reflex_primitives.md`.
- [`focusa_silent_sessions`](tools/focusa_silent_sessions.md) — manage tmux-backed background SilentSessions.

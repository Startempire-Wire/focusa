# Focusa Tool Contract Registry

**Spec:** [`docs/90-ontology-backed-tool-contracts-parity-spec.md`](../90-ontology-backed-tool-contracts-parity-spec.md)

This page documents the current machine-readable contract registry in `apps/pi-extension/src/tool-contracts.ts`. It is current-build truth only.

Validation: `node scripts/validate-focusa-tool-contracts.mjs`.

JSON projection: [`focusa-tool-contracts.json`](focusa-tool-contracts.json).

Current count: **62 Focusa tools**.

Ontology API projections: `GET /v1/ontology/tool-contracts` and `GET /v1/ontology/tool-choreography`.

`apps/pi-extension/src/tool-contracts.ts` also derives `spec96.tool_affordance_catalog.v1` from the registry. Each affordance carries when-to-use, when-not-to-use, default inputs, side effects, safety posture, failure classes, recovery guidance, example invocation, expected result shape, and exact per-tool likely next tools. Pi Focus Slice `TOOL_AFFORDANCES` uses this catalog to expose `best_next`, `recovery`, and `do_not_use` guidance without requiring a model to read source code.

## Contracts

| Tool | Family | Ontology action | API routes | CLI commands | Parity | Doc |
| --- | --- | --- | --- | --- | --- | --- |
| `focusa_project_identity` | `project_identity` | `project.identity` | GET /v1/project/identity | focusa project identity | `domain` | [`doc`](../focusa-tools/tools/focusa_project_identity.md) |
| `focusa_project_card` | `project_identity` | `project.card` | GET /v1/project/card | focusa project card | `domain` | [`doc`](../focusa-tools/tools/focusa_project_card.md) |
| `focusa_project_card_outcome` | `project_identity` | `project.card_outcome` | POST /v1/project/card/outcome | focusa project card-outcome | `full` | [`doc`](../focusa-tools/tools/focusa_project_card_outcome.md) |
| `focusa_session_transfer` | `workpoint` | `session.transfer` | GET /v1/project/card, POST /v1/workpoint/checkpoint, POST /v1/workpoint/resume, GET /v1/trajectory/view | — | `pi_only` | [`doc`](../focusa-tools/tools/focusa_session_transfer.md) |
| `focusa_project_verify` | `project_identity` | `project.verify` | POST /v1/project/verify | focusa project verify | `domain` | [`doc`](../focusa-tools/tools/focusa_project_verify.md) |
| `focusa_trajectory_view` | `trajectory` | `trajectory.view` | GET /v1/trajectory/view | focusa trajectory view | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_view.md) |
| `focusa_trajectory_define_goal` | `trajectory` | `trajectory.define_goal` | POST /v1/trajectory/define-goal | focusa trajectory define-goal | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_define_goal.md) |
| `focusa_trajectory_assess` | `trajectory` | `trajectory.assess` | POST /v1/trajectory/assess | focusa trajectory assess | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_assess.md) |
| `focusa_trajectory_propose_workpoint` | `trajectory` | `trajectory.propose_workpoint` | POST /v1/trajectory/propose-workpoint | focusa trajectory propose-workpoint | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_propose_workpoint.md) |
| `focusa_trajectory_checkpoint` | `trajectory` | `trajectory.checkpoint` | POST /v1/trajectory/checkpoint | focusa trajectory checkpoint | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_checkpoint.md) |
| `focusa_trajectory_resume` | `trajectory` | `trajectory.resume` | POST /v1/trajectory/resume | focusa trajectory resume | `domain` | [`doc`](../focusa-tools/tools/focusa_trajectory_resume.md) |
| `focusa_traverse` | `traversal` | `traverse.read` | POST /v1/traverse, POST /v1/traverse/verify-tags | — | `domain` | [`doc`](../focusa-tools/tools/focusa_traverse.md) |
| `focusa_reflex_primitives` | `traversal` | `reflex.primitives.read` | GET /v1/reflex/primitives | — | `domain` | [`doc`](../focusa-tools/tools/focusa_reflex_primitives.md) |
| `focusa_predict_record` | `metacognition` | `prediction.record` | POST /v1/predictions | focusa predict record | `full` | [`doc`](../focusa-tools/tools/focusa_predict_record.md) |
| `focusa_predict_recent` | `metacognition` | `prediction.recent` | GET /v1/predictions/recent | focusa predict recent | `full` | [`doc`](../focusa-tools/tools/focusa_predict_recent.md) |
| `focusa_predict_evaluate` | `metacognition` | `prediction.evaluate` | POST /v1/predictions/{prediction_id}/evaluate | focusa predict evaluate | `full` | [`doc`](../focusa-tools/tools/focusa_predict_evaluate.md) |
| `focusa_predict_stats` | `metacognition` | `prediction.stats` | GET /v1/predictions/stats | focusa predict stats | `full` | [`doc`](../focusa-tools/tools/focusa_predict_stats.md) |
| `focusa_scratch` | `focus_state` | `focus_state.scratch` | — | — | `local_only` | [`doc`](../focusa-tools/tools/focusa_scratch.md) |
| `focusa_decide` | `focus_state` | `focus_state.decide` | POST /v1/focus/update | focusa focus update --decision | `full` | [`doc`](../focusa-tools/tools/focusa_decide.md) |
| `focusa_constraint` | `focus_state` | `focus_state.constraint` | POST /v1/focus/update | focusa focus update --constraint | `full` | [`doc`](../focusa-tools/tools/focusa_constraint.md) |
| `focusa_failure` | `focus_state` | `focus_state.failure` | POST /v1/focus/update | focusa focus update --failure | `full` | [`doc`](../focusa-tools/tools/focusa_failure.md) |
| `focusa_intent` | `focus_state` | `focus_state.intent` | POST /v1/focus/update | focusa focus update --intent | `full` | [`doc`](../focusa-tools/tools/focusa_intent.md) |
| `focusa_current_focus` | `focus_state` | `focus_state.current.focus` | POST /v1/focus/update | focusa focus update --current-focus | `full` | [`doc`](../focusa-tools/tools/focusa_current_focus.md) |
| `focusa_next_step` | `focus_state` | `focus_state.next.step` | POST /v1/focus/update | focusa focus update --next-step | `full` | [`doc`](../focusa-tools/tools/focusa_next_step.md) |
| `focusa_open_question` | `focus_state` | `focus_state.open.question` | POST /v1/focus/update | focusa focus update --open-question | `full` | [`doc`](../focusa-tools/tools/focusa_open_question.md) |
| `focusa_recent_result` | `focus_state` | `focus_state.recent.result` | POST /v1/focus/update | focusa focus update --recent-result | `full` | [`doc`](../focusa-tools/tools/focusa_recent_result.md) |
| `focusa_note` | `focus_state` | `focus_state.note` | POST /v1/focus/update | focusa focus update --note | `full` | [`doc`](../focusa-tools/tools/focusa_note.md) |
| `focusa_work_loop_writer_status` | `work_loop` | `work_loop.writer.status` | GET /v1/work-loop/status?summary_only=true | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_writer_status.md) |
| `focusa_work_loop_status` | `work_loop` | `work_loop.status` | GET /v1/work-loop/status?summary_only=true | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_status.md) |
| `focusa_work_loop_control` | `work_loop` | `work_loop.control` | POST /v1/work-loop/enable, POST /v1/work-loop/pause, POST /v1/work-loop/resume, POST /v1/work-loop/stop | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_control.md) |
| `focusa_work_loop_context` | `work_loop` | `work_loop.context` | POST /v1/work-loop/context | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_context.md) |
| `focusa_work_loop_checkpoint` | `work_loop` | `work_loop.checkpoint` | POST /v1/work-loop/checkpoint | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_checkpoint.md) |
| `focusa_work_loop_select_next` | `work_loop` | `work_loop.select.next` | POST /v1/work-loop/select-next | — | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_select_next.md) |
| `focusa_state_hygiene_doctor` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.doctor` | — | — | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_doctor.md) |
| `focusa_state_hygiene_plan` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.plan` | — | — | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_plan.md) |
| `focusa_state_hygiene_apply` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.apply` | POST /v1/focus/update | — | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_apply.md) |
| `focusa_silent_sessions` | `work_loop` | `work_loop.silent_session.control` | — | tmux list-sessions, tmux new-session, tmux attach-session, tmux capture-pane, tmux list-panes, tmux pipe-pane, tmux send-keys, tmux send-keys C-c, tmux kill-session | `pi_only` | [`doc`](../focusa-tools/tools/focusa_silent_sessions.md) |
| `focusa_tool_doctor` | `diagnostics_hygiene` | `diagnostics_hygiene.tool_doctor` | GET /v1/health, GET /v1/workpoint/current, GET /v1/work-loop/status?summary_only=true | — | `domain` | [`doc`](../focusa-tools/tools/focusa_tool_doctor.md) |
| `focusa_resource_mode` | `diagnostics_hygiene` | `diagnostics_hygiene.resource_mode.control` | GET /v1/resource/mode, POST /v1/resource/mode | focusa resource mode | `domain` | [`doc`](../focusa-tools/tools/focusa_resource_mode.md) |
| `focusa_active_object_resolve` | `workpoint` | `workpoint.active.object.resolve` | POST /v1/workpoint/active-object/resolve | focusa workpoint resolve-object | `full` | [`doc`](../focusa-tools/tools/focusa_active_object_resolve.md) |
| `focusa_evidence_capture` | `workpoint` | `workpoint.evidence.capture` | POST /v1/workpoint/evidence/link | focusa workpoint evidence-link | `full` | [`doc`](../focusa-tools/tools/focusa_evidence_capture.md) |
| `focusa_workpoint_checkpoint` | `workpoint` | `workpoint.checkpoint` | POST /v1/workpoint/checkpoint | focusa workpoint checkpoint | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_checkpoint.md) |
| `focusa_workpoint_link_evidence` | `workpoint` | `workpoint.link.evidence` | POST /v1/workpoint/evidence/link | focusa workpoint evidence-link | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_link_evidence.md) |
| `focusa_workpoint_resume` | `workpoint` | `workpoint.resume` | POST /v1/workpoint/resume | focusa workpoint resume | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_resume.md) |
| `focusa_tree_head` | `tree_lineage` | `tree_lineage.head` | GET /v1/lineage/head | focusa lineage head | `full` | [`doc`](../focusa-tools/tools/focusa_tree_head.md) |
| `focusa_tree_path` | `tree_lineage` | `tree_lineage.path` | GET /v1/lineage/path/{clt_node_id} | focusa lineage path | `full` | [`doc`](../focusa-tools/tools/focusa_tree_path.md) |
| `focusa_tree_snapshot_state` | `tree_lineage` | `tree_lineage.snapshot.state` | POST /v1/focus/snapshots | focusa state snapshot create | `full` | [`doc`](../focusa-tools/tools/focusa_tree_snapshot_state.md) |
| `focusa_tree_restore_state` | `tree_lineage` | `tree_lineage.restore.state` | POST /v1/focus/snapshots/restore | focusa state snapshot restore | `full` | [`doc`](../focusa-tools/tools/focusa_tree_restore_state.md) |
| `focusa_tree_diff_context` | `tree_lineage` | `tree_lineage.diff.context` | POST /v1/focus/snapshots/diff | focusa state snapshot diff | `full` | [`doc`](../focusa-tools/tools/focusa_tree_diff_context.md) |
| `focusa_metacog_capture` | `metacognition` | `metacognition.capture` | POST /v1/metacognition/capture | focusa metacognition capture | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_capture.md) |
| `focusa_metacog_retrieve` | `metacognition` | `metacognition.retrieve` | POST /v1/metacognition/retrieve | focusa metacognition retrieve | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_retrieve.md) |
| `focusa_metacog_reflect` | `metacognition` | `metacognition.reflect` | POST /v1/metacognition/reflect | focusa metacognition reflect | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_reflect.md) |
| `focusa_metacog_plan_adjust` | `metacognition` | `metacognition.plan.adjust` | POST /v1/metacognition/adjust | focusa metacognition adjust | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_plan_adjust.md) |
| `focusa_metacog_evaluate_outcome` | `metacognition` | `metacognition.evaluate.outcome` | POST /v1/metacognition/evaluate | focusa metacognition evaluate | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_evaluate_outcome.md) |
| `focusa_tree_recent_snapshots` | `tree_lineage` | `tree_lineage.recent.snapshots` | GET /v1/focus/snapshots/recent | focusa state snapshot recent | `full` | [`doc`](../focusa-tools/tools/focusa_tree_recent_snapshots.md) |
| `focusa_tree_snapshot_compare_latest` | `tree_lineage` | `tree_lineage.snapshot.compare.latest` | GET /v1/focus/snapshots/recent, POST /v1/focus/snapshots, POST /v1/focus/snapshots/diff | focusa state snapshot compare-latest | `full` | [`doc`](../focusa-tools/tools/focusa_tree_snapshot_compare_latest.md) |
| `focusa_metacog_recent_reflections` | `metacognition` | `metacognition.recent.reflections` | GET /v1/metacognition/reflections/recent | focusa metacognition recent-reflections | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_recent_reflections.md) |
| `focusa_metacog_recent_adjustments` | `metacognition` | `metacognition.recent.adjustments` | GET /v1/metacognition/adjustments/recent | focusa metacognition recent-adjustments | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_recent_adjustments.md) |
| `focusa_metacog_loop_run` | `metacognition` | `metacognition.loop.run` | POST /v1/metacognition/capture, POST /v1/metacognition/retrieve, POST /v1/metacognition/reflect, POST /v1/metacognition/adjust, POST /v1/metacognition/evaluate | focusa metacognition loop run | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_loop_run.md) |
| `focusa_metacog_doctor` | `metacognition` | `metacognition.doctor` | POST /v1/metacognition/retrieve, GET /v1/metacognition/reflections/recent | focusa metacognition doctor | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_doctor.md) |
| `focusa_lineage_tree` | `tree_lineage` | `tree_lineage.tree` | GET /v1/lineage/tree | focusa lineage tree | `full` | [`doc`](../focusa-tools/tools/focusa_lineage_tree.md) |
| `focusa_li_tree_extract` | `tree_lineage` | `tree_lineage.lineage_intelligence.tree.extract` | GET /v1/lineage/tree | focusa lineage extract | `full` | [`doc`](../focusa-tools/tools/focusa_li_tree_extract.md) |

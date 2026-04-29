# Focusa Tool Contract Registry

**Spec:** [`docs/90-ontology-backed-tool-contracts-parity-spec.md`](../90-ontology-backed-tool-contracts-parity-spec.md)

This page documents the current machine-readable contract registry in `apps/pi-extension/src/tool-contracts.ts`. It is current-build truth only.

Validation: `node scripts/validate-focusa-tool-contracts.mjs`.

| Tool | Family | Ontology action | API routes | CLI commands | Parity | Doc |
| --- | --- | --- | --- | --- | --- | --- |
| `focusa_scratch` | `focus_state` | `focus_state.scratch` | <em>exempt</em> | `focusa focus` | `local_only` | [`doc`](../focusa-tools/tools/focusa_scratch.md) |
| `focusa_decide` | `focus_state` | `focus_state.decide` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_decide.md) |
| `focusa_constraint` | `focus_state` | `focus_state.constraint` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_constraint.md) |
| `focusa_failure` | `focus_state` | `focus_state.failure` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_failure.md) |
| `focusa_intent` | `focus_state` | `focus_state.intent` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_intent.md) |
| `focusa_current_focus` | `focus_state` | `focus_state.current.focus` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_current_focus.md) |
| `focusa_next_step` | `focus_state` | `focus_state.next.step` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_next_step.md) |
| `focusa_open_question` | `focus_state` | `focus_state.open.question` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_open_question.md) |
| `focusa_recent_result` | `focus_state` | `focus_state.recent.result` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_recent_result.md) |
| `focusa_note` | `focus_state` | `focus_state.note` | `POST /v1/focus/update` | `focusa focus` | `full` | [`doc`](../focusa-tools/tools/focusa_note.md) |
| `focusa_work_loop_writer_status` | `work_loop` | `work_loop.writer.status` | `GET /v1/work-loop/status` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_writer_status.md) |
| `focusa_work_loop_status` | `work_loop` | `work_loop.status` | `GET /v1/work-loop/status` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_status.md) |
| `focusa_work_loop_control` | `work_loop` | `work_loop.control` | `POST /v1/work-loop/enable`<br>`POST /v1/work-loop/pause`<br>`POST /v1/work-loop/resume`<br>`POST /v1/work-loop/stop` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_control.md) |
| `focusa_work_loop_context` | `work_loop` | `work_loop.context` | `POST /v1/work-loop/context` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_context.md) |
| `focusa_work_loop_checkpoint` | `work_loop` | `work_loop.checkpoint` | `POST /v1/work-loop/checkpoint` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_checkpoint.md) |
| `focusa_work_loop_select_next` | `work_loop` | `work_loop.select.next` | `POST /v1/work-loop/select-next` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_work_loop_select_next.md) |
| `focusa_state_hygiene_doctor` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.doctor` | <em>exempt</em> | <em>exempt</em> | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_doctor.md) |
| `focusa_state_hygiene_plan` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.plan` | <em>exempt</em> | <em>exempt</em> | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_plan.md) |
| `focusa_state_hygiene_apply` | `diagnostics_hygiene` | `diagnostics_hygiene.state_hygiene.apply` | <em>exempt</em> | <em>exempt</em> | `pi_only` | [`doc`](../focusa-tools/tools/focusa_state_hygiene_apply.md) |
| `focusa_tool_doctor` | `diagnostics_hygiene` | `diagnostics_hygiene.tool_doctor` | `GET /v1/health`<br>`GET /v1/workpoint/current`<br>`GET /v1/work-loop/status` | <em>exempt</em> | `domain` | [`doc`](../focusa-tools/tools/focusa_tool_doctor.md) |
| `focusa_active_object_resolve` | `workpoint` | `workpoint.active.object.resolve` | `POST /v1/workpoint/active-object/resolve` | `focusa workpoint` | `full` | [`doc`](../focusa-tools/tools/focusa_active_object_resolve.md) |
| `focusa_evidence_capture` | `workpoint` | `workpoint.evidence.capture` | `POST /v1/workpoint/evidence/link` | `focusa workpoint` | `full` | [`doc`](../focusa-tools/tools/focusa_evidence_capture.md) |
| `focusa_workpoint_checkpoint` | `workpoint` | `workpoint.checkpoint` | `POST /v1/workpoint/checkpoint` | `focusa workpoint` | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_checkpoint.md) |
| `focusa_workpoint_link_evidence` | `workpoint` | `workpoint.link.evidence` | `POST /v1/workpoint/evidence/link` | `focusa workpoint` | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_link_evidence.md) |
| `focusa_workpoint_resume` | `workpoint` | `workpoint.resume` | `POST /v1/workpoint/resume` | `focusa workpoint` | `full` | [`doc`](../focusa-tools/tools/focusa_workpoint_resume.md) |
| `focusa_tree_head` | `tree_lineage` | `tree_lineage.head` | `GET /v1/lineage/head` | `focusa lineage`<br>`focusa clt` | `full` | [`doc`](../focusa-tools/tools/focusa_tree_head.md) |
| `focusa_tree_path` | `tree_lineage` | `tree_lineage.path` | `GET /v1/lineage/path/{clt_node_id}` | `focusa lineage`<br>`focusa clt` | `full` | [`doc`](../focusa-tools/tools/focusa_tree_path.md) |
| `focusa_tree_snapshot_state` | `tree_lineage` | `tree_lineage.snapshot.state` | <em>exempt</em> | `focusa lineage`<br>`focusa clt` | `pi_only` | [`doc`](../focusa-tools/tools/focusa_tree_snapshot_state.md) |
| `focusa_tree_restore_state` | `tree_lineage` | `tree_lineage.restore.state` | <em>exempt</em> | `focusa lineage`<br>`focusa clt` | `pi_only` | [`doc`](../focusa-tools/tools/focusa_tree_restore_state.md) |
| `focusa_tree_diff_context` | `tree_lineage` | `tree_lineage.diff.context` | <em>exempt</em> | `focusa lineage`<br>`focusa clt` | `pi_only` | [`doc`](../focusa-tools/tools/focusa_tree_diff_context.md) |
| `focusa_metacog_capture` | `metacognition` | `metacognition.capture` | `POST /v1/metacognition/capture` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_capture.md) |
| `focusa_metacog_retrieve` | `metacognition` | `metacognition.retrieve` | `POST /v1/metacognition/retrieve` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_retrieve.md) |
| `focusa_metacog_reflect` | `metacognition` | `metacognition.reflect` | `POST /v1/metacognition/reflect` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_reflect.md) |
| `focusa_metacog_plan_adjust` | `metacognition` | `metacognition.plan.adjust` | `POST /v1/metacognition/adjust` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_plan_adjust.md) |
| `focusa_metacog_evaluate_outcome` | `metacognition` | `metacognition.evaluate.outcome` | `POST /v1/metacognition/evaluate` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_evaluate_outcome.md) |
| `focusa_tree_recent_snapshots` | `tree_lineage` | `tree_lineage.recent.snapshots` | <em>exempt</em> | `focusa lineage`<br>`focusa clt` | `pi_only` | [`doc`](../focusa-tools/tools/focusa_tree_recent_snapshots.md) |
| `focusa_tree_snapshot_compare_latest` | `tree_lineage` | `tree_lineage.snapshot.compare.latest` | <em>exempt</em> | `focusa lineage`<br>`focusa clt` | `pi_only` | [`doc`](../focusa-tools/tools/focusa_tree_snapshot_compare_latest.md) |
| `focusa_metacog_recent_reflections` | `metacognition` | `metacognition.recent.reflections` | `GET /v1/metacognition/reflections/recent` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_recent_reflections.md) |
| `focusa_metacog_recent_adjustments` | `metacognition` | `metacognition.recent.adjustments` | `GET /v1/metacognition/adjustments/recent` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_recent_adjustments.md) |
| `focusa_metacog_loop_run` | `metacognition` | `metacognition.loop.run` | `POST /v1/metacognition/capture`<br>`POST /v1/metacognition/retrieve`<br>`POST /v1/metacognition/reflect`<br>`POST /v1/metacognition/adjust`<br>`POST /v1/metacognition/evaluate` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_loop_run.md) |
| `focusa_metacog_doctor` | `metacognition` | `metacognition.doctor` | `POST /v1/metacognition/retrieve`<br>`GET /v1/metacognition/reflections/recent` | `focusa metacognition` | `full` | [`doc`](../focusa-tools/tools/focusa_metacog_doctor.md) |
| `focusa_lineage_tree` | `tree_lineage` | `tree_lineage.tree` | `GET /v1/lineage/tree` | `focusa lineage`<br>`focusa clt` | `full` | [`doc`](../focusa-tools/tools/focusa_lineage_tree.md) |
| `focusa_li_tree_extract` | `tree_lineage` | `tree_lineage.lineage_intelligence.tree.extract` | `GET /v1/lineage/tree` | `focusa lineage`<br>`focusa clt` | `full` | [`doc`](../focusa-tools/tools/focusa_li_tree_extract.md) |

## Exemptions

Contracts with absent API or CLI parity must state explicit exemptions in `tool-contracts.ts`. Current accepted exemptions are defined in Spec90 §7.3.

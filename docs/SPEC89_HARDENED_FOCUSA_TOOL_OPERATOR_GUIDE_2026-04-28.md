# Spec89 Hardened Focusa Tool Operator Guide — 2026-04-28

Purpose: choose the right hardened `focusa_*` tool without relying on transcript memory.

## Default pickup sequence

1. `focusa_workpoint_resume` — resume canonical/degraded continuation contract after compaction, model switch, fork, or uncertainty.
2. `focusa_tool_doctor` — diagnose unavailable/degraded/stale tool state before guessing.
3. `focusa_active_object_resolve` — resolve object candidates; never invent canonical refs.
4. `focusa_evidence_capture` or `focusa_workpoint_link_evidence` — convert bulky proof into stable evidence refs.
5. Task-specific tool family below.

## Tool families

### Focus State write tools

Use for compact cognitive state only:

- `focusa_scratch`: working notes, reasoning, task lists, hypotheses, self-correction.
- `focusa_decide`: one crystallized architectural decision, max 280 chars.
- `focusa_constraint`: discovered hard requirement from specs/API/operator/environment.
- `focusa_failure`: specific failure plus diagnosis and recovery.
- `focusa_intent`, `focusa_current_focus`, `focusa_next_step`, `focusa_open_question`, `focusa_recent_result`, `focusa_note`: bounded frame steering.

Safety: write tools expose duplicate-candidate/idempotency metadata and validation posture through `tool_result_v1`.

### Workpoint tools

Use for continuity and evidence spine:

- `focusa_workpoint_checkpoint`: create typed continuation checkpoint before compaction/risky work.
- `focusa_workpoint_resume`: fetch canonical/degraded resume packet.
- `focusa_workpoint_link_evidence`: link evidence refs to active Workpoint.
- `focusa_active_object_resolve`: resolve active object candidates without fabricating canonical refs.
- `focusa_evidence_capture`: preserve large proof as evidence handle and optionally link it.

Safety: degraded/non-canonical status must stay explicit; never promote fallback silently.

### Work-loop tools

Use for continuous execution control:

- `focusa_work_loop_writer_status`: read active writer ownership without mutation.
- `focusa_work_loop_status`: read loop state, budgets, replay consumer state.
- `focusa_work_loop_control`: turn/pause/resume/stop; use `preflight=true` first when ownership is uncertain.
- `focusa_work_loop_context`, `focusa_work_loop_checkpoint`, `focusa_work_loop_select_next`: update loop context or advance ready work.

Safety: writer conflicts, dirty worktree, missing task, and unavailable daemon states surface as blocked/retry guidance, not generic failure.

### Tree/lineage tools

Use for recoverable state and ancestry:

- `focusa_tree_head`, `focusa_tree_path`, `focusa_lineage_tree`: inspect current branch/head/lineage.
- `focusa_tree_snapshot_state`, `focusa_tree_recent_snapshots`, `focusa_tree_snapshot_compare_latest`, `focusa_tree_diff_context`: create and compare recoverable checkpoints.
- `focusa_tree_restore_state`: restore only when rollback is intended and safe.
- `focusa_li_tree_extract`: extract decision/constraint/risk signals from lineage.

Safety: snapshot before risky comparisons/restores.

### Metacognition tools

Use for reusable learning, not raw journaling:

- `focusa_metacog_capture`: capture reusable signal; include rationale and evidence refs.
- `focusa_metacog_retrieve`: search prior learning before planning or reflection.
- `focusa_metacog_reflect`: generate hypotheses/strategy updates from recent turns.
- `focusa_metacog_plan_adjust`: create tracked adjustment artifact.
- `focusa_metacog_evaluate_outcome`: evaluate adjustment improvement.
- `focusa_metacog_loop_run`: compressed capture/retrieve/reflect/adjust/evaluate loop.
- `focusa_metacog_doctor`: diagnose signal quality and recommendations.

Safety: metacog results include `quality_gate`, `evidence_refs`, and suggested metrics; low-evidence signals should be improved before promotion.

### State hygiene tools

Use for safe cleanup planning:

- `focusa_state_hygiene_doctor`: read duplicate/stale candidates without mutation.
- `focusa_state_hygiene_plan`: propose safe hygiene steps.
- `focusa_state_hygiene_apply`: approval-gated, non-destructive placeholder until reducer-backed hygiene events exist.

Safety: no silent deletion; prefer supersede/update/merge-up over demotion.

## Maintenance guardrails

- Do not demote existing Focusa tools; redesign, clarify, merge upward, or harden weak tools.
- Every new/changed tool should preserve `details.tool_result_v1` with status, retry posture, side effects, canonical/degraded state, evidence refs, and next-tool hints.
- Validate with TypeScript, envelope skeleton, relevant cargo gates, and live stress before closing Beads.
- Evidence files should cite commands and results; Bead closures must cite code, evidence, and spec sections honestly.

# Spec89 Focusa Pi Tool Inventory — 2026-04-28

Active bead: `focusa-bcyd.1.1`.

Source: `apps/pi-extension/src/tools.ts`.

Total registered `focusa_*` Pi tools: **35**.

## Inventory table

| Line | Tool | Family | Side effect class | Description summary |
|---:|---|---|---|---|
| 277 | `focusa_scratch` | scratchpad | local_write | Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. |
| 315 | `focusa_decide` | focus_state | cognitive_write | Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=280 chars) — architectural choices only, not task lists. |
| 373 | `focusa_constraint` | focus_state | cognitive_write | Record a DISCOVERED REQUIREMENT in Focus State. Constraints are hard boundaries from environment/architecture — NOT self-imposed tasks. Max 200 chars. |
| 418 | `focusa_failure` | focus_state | cognitive_write | Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars. |
| 462 | `focusa_intent` | focus_state | cognitive_write | Set the frame intent — what this session is trying to achieve (1-3 sentences, max 500 chars). |
| 483 | `focusa_current_focus` | focus_state | cognitive_write | Update current focus — what you are actively working on right now (1-3 sentences, max 300 chars). |
| 503 | `focusa_next_step` | focus_state | cognitive_write | Record what you plan to do next (max 160 chars). |
| 522 | `focusa_open_question` | focus_state | cognitive_write | Record an open question that needs to be answered (max 200 chars). |
| 542 | `focusa_recent_result` | focus_state | cognitive_write | Record a completed result, output, or reference (max 300 chars). |
| 562 | `focusa_note` | focus_state | cognitive_write | Miscellaneous note (max 200 chars). Bounded at 20, oldest decay first. |
| 688 | `focusa_work_loop_status` | work_loop | read | Get current continuous work-loop state and budgets. |
| 751 | `focusa_work_loop_control` | work_loop | governed_write | Control continuous work loop: on, pause, resume, stop. |
| 821 | `focusa_work_loop_context` | work_loop | governed_write | Update continuation decision context (current ask/scope/steering). |
| 871 | `focusa_work_loop_checkpoint` | work_loop | governed_write | Create a manual continuous-loop checkpoint. |
| 893 | `focusa_work_loop_select_next` | work_loop | governed_write | Ask daemon to defer blocked work and select next ready work item. |
| 932 | `focusa_workpoint_checkpoint` | workpoint | governed_write | Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint. |
| 1004 | `focusa_workpoint_resume` | workpoint | read | Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action. |
| 1231 | `focusa_tree_head` | tree_snapshot_lineage | read | Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work. |
| 1270 | `focusa_tree_path` | tree_snapshot_lineage | read | Safe ancestry lookup. Use when branch position or lineage depth matters and you do not want to infer it from prior turns. |
| 1306 | `focusa_tree_snapshot_state` | tree_snapshot_lineage | state_checkpoint_or_restore | Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason. |
| 1341 | `focusa_tree_restore_state` | tree_snapshot_lineage | state_checkpoint_or_restore | Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool. |
| 1384 | `focusa_tree_diff_context` | tree_snapshot_lineage | read | Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints. |
| 1419 | `focusa_metacog_capture` | metacognition | cognitive_write | Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson. |
| 1474 | `focusa_metacog_retrieve` | metacognition | read | Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection. |
| 1518 | `focusa_metacog_reflect` | metacognition | cognitive_write | Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes. |
| 1554 | `focusa_metacog_plan_adjust` | metacognition | cognitive_write | Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement. |
| 1589 | `focusa_metacog_evaluate_outcome` | metacognition | cognitive_write | Judge whether an adjustment improved results and whether the learning should be promoted. |
| 1627 | `focusa_tree_recent_snapshots` | tree_snapshot_lineage | read | Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id. |
| 1659 | `focusa_tree_snapshot_compare_latest` | tree_snapshot_lineage | state_checkpoint_or_restore | Create a fresh snapshot and compare it to the latest prior snapshot in one move. Best tool when you want checkpoint + diff without manual id hunting. |
| 1738 | `focusa_metacog_recent_reflections` | metacognition | read | Best safe helper for finding recent reflection ids and update sets before adjust or promote work. |
| 1770 | `focusa_metacog_recent_adjustments` | metacognition | read | Best safe helper for finding recent adjustment ids before evaluation or promotion decisions. |
| 1802 | `focusa_metacog_loop_run` | metacognition | cognitive_write | Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining. |
| 1904 | `focusa_metacog_doctor` | metacognition | read | Diagnose signal quality and retrieval usefulness in one move. Best safe diagnostic tool when deciding whether more capture or reflection work is needed. |
| 1946 | `focusa_lineage_tree` | lineage_intelligence | read | Fetch Focusa lineage tree for /tree-aware reasoning and LI addon workflows. |
| 1983 | `focusa_li_tree_extract` | lineage_intelligence | read | Extract decision/constraint/risk signals and reflection trigger from lineage tree for metacognitive compounding. |

## Family counts

- `focus_state`: 9
- `lineage_intelligence`: 2
- `metacognition`: 9
- `scratchpad`: 1
- `tree_snapshot_lineage`: 7
- `work_loop`: 5
- `workpoint`: 2

## Phase 0 connection

- This inventory is the authoritative current tool list for Spec89 contract matrix work.
- Next bead `focusa-bcyd.1.2` should expand each row into typed input/output, side effects, idempotency, failure modes, canonical/degraded behavior, verification hook, Workpoint linkage, and spec refs.
- No existing tools are marked for demotion; weak tools remain hardening/redesign/clarification/merge-up candidates.

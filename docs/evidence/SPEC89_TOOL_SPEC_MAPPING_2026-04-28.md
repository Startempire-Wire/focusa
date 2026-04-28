# Spec89 Tool-to-Spec Mapping — 2026-04-28

Active bead: `focusa-bcyd.1.3`.

This matrix maps all current `focusa_*` Pi tools to source contracts and preserves the operator no-demotion directive.

| Line | Tool | Family | Source contract refs | No-demotion disposition |
|---:|---|---|---|---|
| 277 | `focusa_scratch` | scratchpad | Spec44 thin Pi fallback; Spec55 fallback semantics; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 315 | `focusa_decide` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 373 | `focusa_constraint` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 418 | `focusa_failure` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 462 | `focusa_intent` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 483 | `focusa_current_focus` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 503 | `focusa_next_step` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 522 | `focusa_open_question` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 542 | `focusa_recent_result` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 562 | `focusa_note` | focus_state | Spec44 Focus State writes; Spec55 typed actions; Spec81 first-class tools; Spec87 desirability; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 688 | `focusa_work_loop_status` | work_loop | Spec55 writer/retry semantics; Spec81 CLI/tool parity; Spec87 control payoff; Spec89 work-loop clarity/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 751 | `focusa_work_loop_control` | work_loop | Spec55 writer/retry semantics; Spec81 CLI/tool parity; Spec87 control payoff; Spec89 work-loop clarity/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 821 | `focusa_work_loop_context` | work_loop | Spec55 writer/retry semantics; Spec81 CLI/tool parity; Spec87 control payoff; Spec89 work-loop clarity/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 871 | `focusa_work_loop_checkpoint` | work_loop | Spec55 writer/retry semantics; Spec81 CLI/tool parity; Spec87 control payoff; Spec89 work-loop clarity/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 893 | `focusa_work_loop_select_next` | work_loop | Spec55 writer/retry semantics; Spec81 CLI/tool parity; Spec87 control payoff; Spec89 work-loop clarity/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 932 | `focusa_workpoint_checkpoint` | workpoint | Spec88 Workpoint continuity; Spec44 single cognitive authority; Spec52 thin adapter; Spec55 typed contracts; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1004 | `focusa_workpoint_resume` | workpoint | Spec88 Workpoint continuity; Spec44 single cognitive authority; Spec52 thin adapter; Spec55 typed contracts; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1231 | `focusa_tree_head` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1270 | `focusa_tree_path` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1306 | `focusa_tree_snapshot_state` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1341 | `focusa_tree_restore_state` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1384 | `focusa_tree_diff_context` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1419 | `focusa_metacog_capture` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1474 | `focusa_metacog_retrieve` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1518 | `focusa_metacog_reflect` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1554 | `focusa_metacog_plan_adjust` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1589 | `focusa_metacog_evaluate_outcome` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1627 | `focusa_tree_recent_snapshots` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1659 | `focusa_tree_snapshot_compare_latest` | tree_snapshot_lineage | Spec56 checkpoint/recovery; Spec55 state safety; Spec87 safe starting point; Spec89 no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1738 | `focusa_metacog_recent_reflections` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1770 | `focusa_metacog_recent_adjustments` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1802 | `focusa_metacog_loop_run` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1904 | `focusa_metacog_doctor` | metacognition | Spec81 compound learning loops; Spec87 desirability; Spec55 quality/retry semantics; Spec89 metacog quality gates/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1946 | `focusa_lineage_tree` | lineage_intelligence | Spec55 bounded output; Spec87 pickup evidence; Spec89 lineage intelligence/no-demotion | Preserve; harden/clarify if payoff not obvious. |
| 1983 | `focusa_li_tree_extract` | lineage_intelligence | Spec55 bounded output; Spec87 pickup evidence; Spec89 lineage intelligence/no-demotion | Preserve; harden/clarify if payoff not obvious. |

## Cross-spec anchors

- Spec44: Focusa remains the single cognitive authority; Pi is thin glue.
- Spec52: Pi extension consumes/proposes typed state and must not become a parallel cognitive DB.
- Spec55: tools need typed inputs/outputs, side effects, failure modes, idempotency, verification hooks, retry policy, degraded fallback, and recovery semantics.
- Spec81: tool suite and CLI must be first-class, reliable, discoverable, typed, and useful.
- Spec87: tools must be desirable through clear payoff, low friction, visible summaries, next-step hints, and pickup evidence.
- Spec88: Workpoint checkpoint/resume are canonical continuation primitives.
- Spec89: no existing Focusa tool should be demoted; weak tools remain hardening/redesign/clarification/merge-up candidates.

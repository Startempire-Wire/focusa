# Spec96 Trajectory Agent Golden Evals

This evidence note records the golden eval contract for `focusa-4oik`.

The eval in `tests/spec96_trajectory_agent_golden_eval_test.sh` compares with-trajectory vs without-trajectory prompt/source surfaces for:

- Pi Focus Slice + compaction resume injection.
- CLI/API documentation and awareness-card command surface.
- Generic/non-Pi Focusa Utility Card injection.

Required scenarios:

1. Project mismatch demotes stale context.
2. Compaction resume uses Workpoint Resume Packet v2, then trajectory, then bounded traverse.
3. Degraded daemon mode marks cognition degraded and gives recovery tools.
4. Drift avoidance keeps same-high-level sessions distinct by `project_root + continuity_id`.
5. Assistance reduction favors `active_gap`, `recommended_action`, and `next_tools` over broad questions.
6. Definition of Done remains proof/evidence based.

Baseline prompts intentionally lack trajectory markers. The enriched surfaces must score higher and include enough markers for each scenario.

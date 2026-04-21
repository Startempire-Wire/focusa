# SPEC80 E1.1 — Metric Extraction Pipeline Design

Date: 2026-04-21
Bead: `focusa-yro7.5.1.1`
Label: `documented-authority`

Purpose: define extraction pipeline for SPEC80 outcome contracts and Gate D inputs.

## Target metrics

1. `strategy_adjusted_turn_rate`
2. `failed_turn_ratio`
3. `rework_loop_rate`
4. `novel_context_strategy_reuse_rate`
5. `setback_recovery_rate`
6. `perspective_constraint_density`
7. `steering_uptake_rate`
8. `forced_pause_rate_after_steering`

## Source mapping

- Turn/event stream: loop checkpoints, plan-adjust events, failures, pauses.
- Practice-observation forms (Appendix E): strategy class, adaptation, outcome payloads.
- Lineage context: novel-context tagging by branch/task scope.
- Operator steering events: steering marker + subsequent uptake/pause outcomes.

## Pipeline stages

1. Ingest
- collect normalized turn/session/work-item events in evaluation window.

2. Enrich
- join branch context, novel-context tags, and adaptation markers.

3. Derive counters
- derive numerator/denominator per metric with explicit null-handling.

4. Aggregate
- daily rollup + rolling 14-day rollup.

5. Emit
- scorecard payload for threshold evaluator (E1.2).

## Counter definitions

- `strategy_adjusted_turn_rate` = turns_with_checkpoint_or_plan_adjust / total_turns
- `failed_turn_ratio` = failed_turns / total_turns
- `rework_loop_rate` = turns_marked_rework / total_turns
- `novel_context_strategy_reuse_rate` = novel_context_turns_with_prior_strategy_reuse / novel_context_turns
- `setback_recovery_rate` = loops_continued_after_failure / loops_with_failure
- `perspective_constraint_density` = perspective_aware_constraints / total_constraints
- `steering_uptake_rate` = steering_events_with_uptake / total_steering_events
- `forced_pause_rate_after_steering` = forced_pauses_after_steering / total_steering_events

## Output schema

```json
{
  "window": {"start":"ISO-8601","end":"ISO-8601"},
  "sample_sizes": {"turns":0,"novel_context_turns":0,"loops_with_failures":0},
  "metrics": {
    "strategy_adjusted_turn_rate": 0.0,
    "failed_turn_ratio": 0.0,
    "rework_loop_rate": 0.0,
    "novel_context_strategy_reuse_rate": 0.0,
    "setback_recovery_rate": 0.0,
    "perspective_constraint_density": 0.0,
    "steering_uptake_rate": 0.0,
    "forced_pause_rate_after_steering": 0.0
  }
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §16, §18)
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md

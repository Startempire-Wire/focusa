# SPEC80 E2.1 — Baseline Window Computation Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.2.1`
Label: `documented-authority`

Purpose: define canonical baseline computation for Gate D comparisons.

## Window definitions

- Baseline window: prior 14 days before feature enablement.
- Evaluation window: rolling 14 days after enablement.

## Computation rules

1. Compute daily metric values from extraction pipeline.
2. Baseline value for each metric = median(daily_values) over baseline window.
3. Require minimum sample sizes:
   - >=200 turns overall
   - >=30 novel-context turns
   - >=20 loops with failures (setback metrics)
4. If sample gate fails, mark metric `insufficient_sample`.

## Storage contract

```json
{
  "baseline_id": "string",
  "window": {"start":"ISO-8601","end":"ISO-8601"},
  "metrics": {"metric_name": {"median": 0.0, "sample_size": 0, "status": "ok|insufficient_sample"}},
  "computed_at": "ISO-8601"
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §16)
- docs/evidence/SPEC80_E1_1_METRIC_EXTRACTION_PIPELINE_DESIGN_2026-04-21.md

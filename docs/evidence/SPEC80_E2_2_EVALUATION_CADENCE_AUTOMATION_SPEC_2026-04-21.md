# SPEC80 E2.2 — Evaluation Cadence Automation Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.2.2`
Label: `documented-authority`

Purpose: automate reporting cadence and gate decision schedule from Appendix C.

## Required cadence

1. Daily internal snapshot
- emit metric deltas + sample-size status.

2. Weekly operator report
- summarize trendlines, contract pass state, and risk flags.

3. 14-day gate decision run
- execute full Gate D evaluator with regression override.

## Job contract

```json
{
  "job_id": "string",
  "cadence": "daily|weekly|14day_gate",
  "window": {"start":"ISO-8601","end":"ISO-8601"},
  "status": "ok|partial|failed",
  "outputs": ["report_ref"],
  "run_at": "ISO-8601"
}
```

## Failure handling

- missed run -> emit `cadence_miss` alert and backfill once.
- insufficient sample -> emit `sample_gate_blocked` in report.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§16)
- docs/evidence/SPEC80_E1_2_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md

# SPEC80 E2.2 — Evaluation Cadence Automation Design

Date: 2026-04-21
Bead: `focusa-yro7.5.2.2`
Purpose: define automated scheduling and output contracts for Appendix C daily/weekly/14-day scoring cadence.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (Appendix C §16)
- docs/evidence/SPEC80_BASELINE_WINDOW_COMPUTATION_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md

## Required cadence protocol

Appendix C requires:
1. daily internal snapshot,
2. weekly operator report,
3. Gate D decision every 14 days.

## Scheduler design

### Triggers
- `daily_snapshot_job`: runs once every 24h.
- `weekly_report_job`: runs every 7 days.
- `gate_decision_job`: runs every 14 days.

### Alignment constraints
1. All jobs anchor to the same `enablement_id` timeline.
2. Weekly and 14-day jobs consume only completed daily snapshots.
3. Missed runs must be replayable by deterministic backfill (`from_ts`, `to_ts`).

## Output contracts

### Daily internal snapshot
Fields:
- `run_id`
- `window_end`
- `metric_rows_count`
- `sample_floor_status`
- `contract_precheck_summary`

### Weekly operator report
Fields:
- `report_id`
- `week_range`
- `contract_trends[]`
- `regression_watchlist[]`
- `recommended_actions[]`

### 14-day Gate D decision report
Fields:
- `decision_id`
- `evaluation_window`
- `pass_count`
- `critical_regression`
- `final_decision`
- `evidence_refs[]`

## Failure and recovery semantics

Typed scheduler/report statuses:
- `ok`
- `upstream_data_missing`
- `insufficient_sample`
- `backfill_required`

Recovery rules:
1. Failed weekly/14-day runs must not emit partial pass/fail decisions.
2. Backfilled runs must preserve original window boundaries and deterministic ids.
3. Gate decision run is blocked when required upstream snapshot set is incomplete.

## Determinism constraints

1. Identical source data and schedule windows must produce identical report payloads.
2. Job sequencing order is fixed: daily -> weekly -> 14-day gate decision.
3. Clock/tz normalization uses UTC window boundaries for all cadence jobs.

## Gate linkage

- Implements Appendix C reporting cadence automation requirement for baseline and scoring protocol.
- Provides periodic evidence feed required by E4 Gate D reporting lane.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_BASELINE_WINDOW_COMPUTATION_2026-04-21.md
- docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md

# SPEC80 E1.1 — Outcome Metric Extraction Pipeline Design

Date: 2026-04-21
Bead: `focusa-yro7.5.1.1`
Purpose: design metric extraction pipeline for six Spec80 outcome contracts and their gate-ready inputs.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§8, §10 Gate D)
- docs/79-focusa-governed-continuous-work-loop.md

## Pipeline stages

1. **Ingest**
   - Collect turn/task/replay telemetry events for rolling 14-day windows.
2. **Normalize**
   - Canonicalize event fields needed for numerator/denominator counters.
3. **Aggregate**
   - Compute contract metrics by window and context bucket.
4. **Baseline compare**
   - Compare evaluation window against prior 14-day median baseline.
5. **Emit score inputs**
   - Output per-contract delta values + regression flags for threshold evaluator.

## Source-to-metric mapping

| Outcome contract | Metric(s) | Numerator source signals | Denominator source signals |
|---|---|---|---|
| Self-regulation | `strategy_adjusted_turn_rate` | turns containing checkpoint or plan-adjust evidence | total turns |
| Outcome quality | `failed_turn_ratio`, `rework_loop_rate` | failed turns; turns marked rework | total turns |
| Transfer | `novel_context_strategy_reuse_rate` | successful strategy reuse in contexts tagged novel | total turns in novel-context bucket |
| Motivation/ownership | `setback_recovery_rate` | loops continued after failure | loops with failure |
| Social/perspective quality | `perspective_constraint_density` | perspective-aware constraints | total constraints |
| Instructor/operator regulation | `steering_uptake_rate`, `forced_pause_rate_after_steering` | steering accepted events; forced pauses after steering | steering opportunities/events |

## Windowing + baseline policy

- Evaluation window: rolling 14 days.
- Baseline window: prior 14-day median.
- Required sample floors (for downstream scoring validity):
  - `>=200` total turns
  - `>=30` novel-context turns
  - `>=20` loops with failures

## Extraction output contract

Per metric output row:
- `metric_id`
- `window_start`
- `window_end`
- `numerator`
- `denominator`
- `value`
- `baseline_value`
- `relative_delta`
- `sample_size_ok` (boolean)
- `notes` (optional)

## Determinism constraints

1. Same input telemetry slice must produce identical metric rows.
2. Missing denominator cases emit `value=null` with explicit reason; no implicit zero division.
3. Novel-context tagging criteria must be stable within a window run.

## Gate linkage

- Feeds E1.2 threshold evaluator for Spec80 Gate D decision (`>=4/6` contracts pass, no critical outcome-quality regression).

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md

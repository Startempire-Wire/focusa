# SPEC87 Helper and Composite Tools Note

**Date:** 2026-04-22  
**Spec:** `docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md`  
**Task:** `focusa-eulk.3`

## New tools added

### Tree helpers/composites
- `focusa_tree_recent_snapshots` — `apps/pi-extension/src/tools.ts:1507`
- `focusa_tree_snapshot_compare_latest` — `apps/pi-extension/src/tools.ts:1539`

### Metacognition helpers/composites
- `focusa_metacog_recent_reflections` — `apps/pi-extension/src/tools.ts:1618`
- `focusa_metacog_recent_adjustments` — `apps/pi-extension/src/tools.ts:1650`
- `focusa_metacog_loop_run` — `apps/pi-extension/src/tools.ts:1682`
- `focusa_metacog_doctor` — `apps/pi-extension/src/tools.ts:1784`

## Backend support added

- recent snapshots endpoint: `crates/focusa-api/src/routes/snapshots.rs:467`
- recent reflections endpoint: `crates/focusa-api/src/routes/metacognition.rs:671`
- recent adjustments endpoint: `crates/focusa-api/src/routes/metacognition.rs:673`
- doctor support uses retrieve `summary_only`: `crates/focusa-api/src/routes/metacognition.rs:302`

## Why this matters

These tools reduce the biggest pickup blockers:
- id hunting
- too many manual tool calls
- weak one-step workflow options

## Result

The tool layer now includes low-friction helpers plus real composite tools that compress common tree and metacognition workflows.

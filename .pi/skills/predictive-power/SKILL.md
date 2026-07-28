---
name: predictive-power
description: "Use when recording, evaluating, or interpreting Focusa predictions, risk forecasts, calibration stats, or predictive next-action guidance."
---

# Predictive Power

Use this skill when recording, evaluating, or interpreting Focusa predictions.

## Progressive disclosure

Read `references/01-focusa-predictive-power-runbook.md` for scoped record, evaluation, calibration, and contract-drift recovery.

## Rules

1. Predictions guide; they never override operator steering.
2. Always include evidence refs or route/tool handles when possible.
3. Record before acting when there is meaningful uncertainty.
4. Evaluate after outcome is known.
5. Use `focusa_predict_stats` or `focusa predict stats` to inspect calibration.

## Tool flow

1. `focusa_predict_record`
2. act normally with explicit operator/current-project priority
3. `focusa_predict_evaluate`
4. `focusa_predict_stats`


## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `predictive-power` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: hand-authored; no automatic sibling-body injection
- supersession: none

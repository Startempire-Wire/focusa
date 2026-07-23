---
name: focusa-temporal-authority
description: "Use when deadlines, freshness, temporal evidence, prior-valid fallback, forecast windows, or stale-state decisions affect authority."
---

# Focusa Temporal Authority

Use when deadlines, freshness, temporal evidence, prior-valid fallback, forecast windows, or stale-state decisions affect authority.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-temporal-authority-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- deadline
- freshness
- stale state
- forecast
- history fallback

## Non-trigger examples

- invented dates
- transcript chronology as canonical state

## Required sequence

1. `focusa_trajectory_view`
2. `focusa_hlt_history`
3. `focusa_predict_record`
4. `focusa_predict_evaluate`
5. `focusa_evidence_capture`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_trajectory_assess`
- `focusa_workpoint_resume`
- `focusa_project_verify`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

Decision cites current temporal evidence, freshness, confidence, and bounded forecast/recovery.

Stable evidence or receipt refs must support any completion claim.

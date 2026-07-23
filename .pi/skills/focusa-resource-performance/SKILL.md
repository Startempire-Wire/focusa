---
name: focusa-resource-performance
description: "Use for LowMem/resource pressure, Bloatgaurd budgets, token control, bounded traversal, latency, and hot/cold route performance."
---

# Focusa Resource Performance

Use for LowMem/resource pressure, Bloatgaurd budgets, token control, bounded traversal, latency, and hot/cold route performance.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-resource-performance-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- resource exhausted
- hot-path timeout
- prompt bloat
- large traversal

## Non-trigger examples

- unbounded full payload
- premature generic context loading

## Required sequence

1. `focusa_resource_mode`
2. `focusa_bloatgaurd_report`
3. `focusa_bloatgaurd_tokenbloat_report`
4. `focusa_traverse`
5. `focusa_tool_bundle`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_resource_mode`
- `focusa_traverse`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

Work completes inside declared memory/token/latency budgets with bounded payloads.

Stable evidence or receipt refs must support any completion claim.

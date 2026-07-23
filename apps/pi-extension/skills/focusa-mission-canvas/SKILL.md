---
name: focusa-mission-canvas
description: "Use for Mission Canvas, Work Rail, CRIST interviews, generated UI, workspace artifacts, live refresh, and provider-neutral operation binding."
---

# Focusa Mission Canvas

Use for Mission Canvas, Work Rail, CRIST interviews, generated UI, workspace artifacts, live refresh, and provider-neutral operation binding.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-mission-canvas-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- mission canvas
- CRIST
- work rail
- generated UI
- workspace artifact

## Non-trigger examples

- hand-coded parallel UI contract
- invented operation binding

## Required sequence

1. `focusa_call_stack_design`
2. `focusa_context_cognition`
3. `focusa_evidence_capture`
4. `focusa_active_object_resolve`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_call_stack_verify`
- `focusa_tool_doctor`
- `focusa_workpoint_resume`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

Generated UI binds canonical operations and durable workspace evidence without semantic drift.

Stable evidence or receipt refs must support any completion claim.

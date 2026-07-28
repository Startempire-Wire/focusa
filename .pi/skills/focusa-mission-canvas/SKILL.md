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

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-mission-canvas` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Generated UI binds canonical operations and durable workspace evidence without semantic drift.

Stable evidence or receipt refs must support any completion claim.

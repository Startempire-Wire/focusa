---
name: focusa-project-card
description: "Use for project card authority, routing, evidence, and recovery."
---

# Focusa Project Card

Use for project card authority, routing, evidence, and recovery.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-project-card-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- project card authority, routing, evidence, and recovery

## Non-trigger examples

- unrelated implementation work
- a narrower skill owns the selected capability

## Required sequence

1. `focusa_project_card`
2. `focusa_project_card_outcome`
3. `focusa_predict_record`
4. `focusa_workpoint_checkpoint`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_tool_doctor`
- `focusa_project_verify`
- `focusa_workpoint_resume`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-project-card` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

The scoped operation is verified, evidenced, and handed to the next owning skill.

Stable evidence or receipt refs must support any completion claim.

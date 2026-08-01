---
name: focusa-project-scope
description: "Use when project root, continuity, repository, deployment, alias, or cross-project authority may be ambiguous before trusting state or mutating files."
---

# Focusa Project Scope

Use when project root, continuity, repository, deployment, alias, or cross-project authority may be ambiguous before trusting state or mutating files.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-project-scope-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- project switch
- scope conflict
- cross-project handoff

## Non-trigger examples

- verified same-root read-only action

## Required sequence

1. `focusa_project_identity`
2. `focusa_project_verify`
3. `focusa_active_object_resolve`
4. `focusa_workpoint_resume`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Operator alignment

- Refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps.
- Treat cwd as launch location only; missing trajectory or project markers do not imply a new user or new project.
- Use plain language and progressive disclosure; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested.
- Never invent deadlines or urgency; use temporal authority and express forecast uncertainty as a range.
- Measure meaningful tasks in wall-clock operator time: predict delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons.
- Apply Focusa tools to accomplish the operator's desired outcome within operator constraints, rather than making Focusa mechanics the center of conversation.

## Failure recovery

- `focusa_project_verify`
- `focusa_workpoint_checkpoint`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata

- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-project-scope` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

project_root plus continuity_id authority is verified and target objects resolve in that scope.

Stable evidence or receipt refs must support any completion claim.

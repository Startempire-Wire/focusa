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

## Failure recovery

- `focusa_project_verify`
- `focusa_workpoint_checkpoint`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

project_root plus continuity_id authority is verified and target objects resolve in that scope.

Stable evidence or receipt refs must support any completion claim.

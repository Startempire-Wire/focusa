---
name: focusa
description: "Use for focusa workflows with bounded Focusa authority."
---

# Focusa

Use for focusa workflows with bounded Focusa authority.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- focusa workflows with bounded Focusa authority

## Non-trigger examples

- unrelated implementation work
- a narrower skill owns the selected capability

## Required sequence

1. `focusa_agent_prompt`
2. `focusa_awareness_packet`
3. `focusa_constraint`
4. `focusa_current_focus`
5. `focusa_decide`
6. `focusa_failure`
7. `focusa_intent`
8. `focusa_next_step`
9. `focusa_note`
10. `focusa_open_question`
11. `focusa_recent_result`
12. `focusa_reflex_primitives`
13. `focusa_scratch`
14. `focusa_tool_doctor`
15. `focusa_utility_card`

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
- workflow: `focusa-project-scope` → `focusa` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

The scoped operation is verified, evidenced, and handed to the next owning skill.

Stable evidence or receipt refs must support any completion claim.

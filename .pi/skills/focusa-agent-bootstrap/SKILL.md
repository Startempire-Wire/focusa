---
name: focusa-agent-bootstrap
description: "Use when orienting a new or resumed agent with bounded Focusa identity, Agent Card, progressive tool discovery, preload, Workpoint, and Trajectory context."
---

# Focusa Agent Bootstrap

Use when orienting a new or resumed agent with bounded Focusa identity, Agent Card, progressive tool discovery, preload, Workpoint, and Trajectory context.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-agent-bootstrap-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- new agent session
- resume after context loss
- unknown Focusa capabilities

## Non-trigger examples

- one already-selected tool call
- unrelated repository work

## Required sequence

1. `focusa_agent_card`
2. `focusa_project_identity`
3. `focusa_workpoint_resume`
4. `focusa_trajectory_view`
5. `focusa_tool_search`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_tool_doctor`
- `focusa_project_verify`
- `focusa_workpoint_checkpoint`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-agent-bootstrap` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Agent has verified scope, current Workpoint/Trajectory orientation, and only the schemas needed for the next action.

Stable evidence or receipt refs must support any completion claim.

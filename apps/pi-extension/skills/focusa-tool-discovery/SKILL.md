---
name: focusa-tool-discovery
description: "Use when selecting or composing Focusa capabilities through search, describe, graph, family bundles, strict schemas, and cross-harness bindings without prompt bloat."
---

# Focusa Tool Discovery

Use when selecting or composing Focusa capabilities through search, describe, graph, family bundles, strict schemas, and cross-harness bindings without prompt bloat.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-tool-discovery-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- unknown tool
- multi-tool workflow
- strict parameters required

## Non-trigger examples

- exact tool and schema already loaded
- request to load all schemas

## Required sequence

1. `focusa_tool_search`
2. `focusa_tool_describe`
3. `focusa_tool_graph`
4. `focusa_tool_bundle`
5. `focusa_agent_card`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Operator alignment

- Refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps.
- Treat cwd as launch location only; missing trajectory or project markers do not imply a new user or new project.
- Use plain language and progressive disclosure; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested.
- Never invent deadlines or urgency; use temporal authority and express forecast uncertainty as a range.
- Measure meaningful tasks in wall-clock operator time: predict delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons.
- Apply Focusa tools to accomplish the operator's desired outcome within operator constraints, rather than making Focusa mechanics the center of conversation.

## Failure recovery

- `focusa_tool_search`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata

- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-tool-discovery` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

The narrowest valid capability and dependency sequence are selected under token budget.

Stable evidence or receipt refs must support any completion claim.

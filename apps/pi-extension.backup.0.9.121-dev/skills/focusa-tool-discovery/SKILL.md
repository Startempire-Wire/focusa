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

## Failure recovery

- `focusa_tool_search`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

The narrowest valid capability and dependency sequence are selected under token budget.

Stable evidence or receipt refs must support any completion claim.

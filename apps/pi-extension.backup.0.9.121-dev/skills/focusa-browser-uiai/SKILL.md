---
name: focusa-browser-uiai
description: "Use for UIAI-first browser research/action, WebMCP capability intake, session/origin isolation, diagnostics, evidence, and Workpoint linkage."
---

# Focusa Browser Uiai

Use for UIAI-first browser research/action, WebMCP capability intake, session/origin isolation, diagnostics, evidence, and Workpoint linkage.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-browser-uiai-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- URL or website task
- browser action
- WebMCP page tools
- visual failure

## Non-trigger examples

- generic web fallback before UIAI health
- unbound page mutation

## Required sequence

1. `focusa_browser_workflow_plan`
2. `focusa_browser_capabilities_intake`
3. `focusa_browser_diagnostics_intake`
4. `focusa_evidence_capture`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_browser_diagnostics_intake`
- `focusa_tool_doctor`
- `focusa_resource_mode`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

Browser result is proven in the bound session/origin, diagnostics are ingested, evidence is linked, and unused sessions are closed.

Stable evidence or receipt refs must support any completion claim.

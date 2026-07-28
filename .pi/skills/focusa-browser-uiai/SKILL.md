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

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-browser-uiai` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Browser result is proven in the bound session/origin, diagnostics are ingested, evidence is linked, and unused sessions are closed.

Stable evidence or receipt refs must support any completion claim.

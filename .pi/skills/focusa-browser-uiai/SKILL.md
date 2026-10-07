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

## Place in the complete project journey

Follow `docs/agent/02-focusa-cohesive-project-flow.md`: verified binding → Bootstrap when needed → Genesis when needed → linked Ladder/spec/tasks → Workpoint → Prepare/Act/Reconcile/Advance.
Reuse valid state; preserve the accepted goal while refining only affected work. This skill supplies capabilities for that journey, not a separate workflow or authority.

## Available capabilities — select by current condition

- `focusa_browser_workflow_plan`
- `focusa_browser_capabilities_intake`
- `focusa_browser_diagnostics_intake`
- `focusa_evidence_capture`

This inventory is not a mandatory sequence. Read, preview, mutation, restore and evidence operations have different preconditions; never execute every listed tool merely to finish a skill.

## Operator alignment

- Refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps.
- Treat cwd as launch location only; missing trajectory or project markers do not imply a new user or new project.
- Use plain language and progressive disclosure; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested.
- Never invent deadlines or urgency; use temporal authority and express forecast uncertainty as a range.
- Measure meaningful tasks in wall-clock operator time: predict delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons.
- Apply Focusa tools to accomplish the operator's desired outcome within operator constraints, rather than making Focusa mechanics the center of conversation.

## Failure recovery

- `focusa_browser_diagnostics_intake`
- `focusa_tool_doctor`
- `focusa_resource_mode`

A rejected operation is not a stopped mission. Diagnose its exact cause, select supported in-scope recovery, verify and resume the interrupted action; advance independent admitted work when possible. Pending work requires observation, not duplicate dispatch. Reconcile uncertain effects before replay. Real scope, consent, integrity and budget boundaries remain enforced; never fabricate admission or repeatedly retry unchanged input.

## Routing metadata

- prerequisites: verified ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey; project/cwd selection alone is not attachment
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

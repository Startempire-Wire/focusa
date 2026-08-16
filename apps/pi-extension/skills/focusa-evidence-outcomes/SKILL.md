---
name: focusa-evidence-outcomes
description: "Use for evidence refs, receipts, prediction evaluation, proposal settlement, project-card outcomes, acceptance truth, and false-completion prevention."
---

# Focusa Evidence Outcomes

Use for evidence refs, receipts, prediction evaluation, proposal settlement, project-card outcomes, acceptance truth, and false-completion prevention.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-evidence-outcomes-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- claim completion
- settle proposal
- evaluate prediction
- attach proof

## Non-trigger examples

- raw transcript as proof
- closing on degraded/pending status

## Required sequence

1. `focusa_evidence_capture`
2. `focusa_workpoint_link_evidence`
3. `focusa_predict_evaluate`
4. `focusa_project_card_outcome`
5. `focusa_metacog_capture`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_active_object_resolve`
- `focusa_workpoint_resume`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-evidence-outcomes` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Canonical outcome status is backed by stable evidence/receipt refs and evaluated learning signals.

Stable evidence or receipt refs must support any completion claim.

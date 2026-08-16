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

## Done condition

Canonical outcome status is backed by stable evidence/receipt refs and evaluated learning signals.

Stable evidence or receipt refs must support any completion claim.

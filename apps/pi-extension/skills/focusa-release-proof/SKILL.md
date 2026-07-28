---
name: focusa-release-proof
description: "Use when preparing a release gate, proof bundle, changelog, resolved-issue audit, local quality evidence, and operator-authorized CI/release execution."
---

# Focusa Release Proof

Use when preparing a release gate, proof bundle, changelog, resolved-issue audit, local quality evidence, and operator-authorized CI/release execution.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-release-proof-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- release readiness
- proof rehearsal
- changelog
- close release gate

## Non-trigger examples

- development-time CI without authorization
- claiming unverified platform proof

## Required sequence

1. `focusa_evidence_capture`
2. `focusa_workpoint_link_evidence`
3. `focusa_predict_stats`
4. `focusa_metacog_doctor`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_tool_doctor`
- `focusa_workpoint_resume`
- `focusa_project_verify`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-release-proof` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Every acceptance criterion, issue, changelog entry, platform boundary, and evidence ref is verified before authorized release action.

Stable evidence or receipt refs must support any completion claim.

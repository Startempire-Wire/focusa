---
name: focusa-spec-implementation
description: "Use when turning a Focusa specification into a call-stack design, Trajectory gap, Beads tasks, bounded implementation, and conformance evidence."
---

# Focusa Spec Implementation

Use when turning a Focusa specification into a call-stack design, Trajectory gap, Beads tasks, bounded implementation, and conformance evidence.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-spec-implementation-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- implement spec
- new agent feature
- cross-layer refactor

## Non-trigger examples

- coding without spec/trajectory/tasks
- scope expansion by inference

## Required sequence

1. `focusa_call_stack_design`
2. `focusa_trajectory_assess`
3. `focusa_workpoint_checkpoint`
4. `focusa_call_stack_verify`
5. `focusa_evidence_capture`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_project_verify`
- `focusa_workpoint_resume`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-spec-implementation` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

Implementation matches the typed call stack, acceptance criteria, task closure, and linked proof.

Stable evidence or receipt refs must support any completion claim.

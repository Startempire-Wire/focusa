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

## Done condition

Implementation matches the typed call stack, acceptance criteria, task closure, and linked proof.

Stable evidence or receipt refs must support any completion claim.

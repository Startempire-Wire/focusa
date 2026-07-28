---
name: focusa-session-recovery
description: "Use for compaction, model switch, context overflow, fork, rollover, session transfer, lineage snapshots, and canonical continuation recovery."
---

# Focusa Session Recovery

Use for compaction, model switch, context overflow, fork, rollover, session transfer, lineage snapshots, and canonical continuation recovery.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-session-recovery-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- after compaction
- model switch
- session rollover
- context uncertainty

## Non-trigger examples

- transcript-tail guess
- routine stateless call

## Required sequence

1. `focusa_workpoint_resume`
2. `focusa_trajectory_resume`
3. `focusa_session_transfer`
4. `focusa_tree_snapshot_state`
5. `focusa_tree_restore_state`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_project_identity`
- `focusa_workpoint_checkpoint`
- `focusa_tool_doctor`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Routing metadata
- prerequisites: verified project identity and typed continuity when durable scope matters
- use_instead_when: use the narrower owner in `docs/contracts/65-focusa-skill-ownership-manifest.json`
- next_skills: `focusa-workpoint`, `focusa-evidence-outcomes`, `focusa-metacognition`
- failure_handoff: `focusa-troubleshooting`
- authority_boundary: operator steering leads; daemon and typed Workpoint/Trajectory contracts remain canonical
- workflow: `focusa-project-scope` → `focusa-session-recovery` → `focusa-workpoint` → `focusa-evidence-outcomes`
- minimum_contract: `focusa.tool_affordance_catalog.v1`
- source_status: generated core plus hand-authored registry content; no sibling-body injection
- supersession: none

## Done condition

A canonical Workpoint packet supplies mission, action, evidence, blockers, and exact next action.

Stable evidence or receipt refs must support any completion claim.

---
name: focusa-install-lifecycle
description: "Use for supported install, first Workpoint, repair/rerun, trusted OTA update, rollback, uninstall, and preserved-user-data verification."
---

# Focusa Install Lifecycle

Use for supported install, first Workpoint, repair/rerun, trusted OTA update, rollback, uninstall, and preserved-user-data verification.

## Progressive disclosure

1. Load this core file only when its trigger matches.
2. Read `references/01-focusa-install-lifecycle-runbook.md` only for the selected workflow.
3. Use `focusa_tool_describe` to cold-load exact schemas only for selected tools.
4. Open linked specs/evidence only when a branch requires them.

## Trigger examples

- install Focusa
- repair installation
- OTA update
- rollback
- uninstall

## Non-trigger examples

- unplanned system mutation
- Mac validation from VPS

## Required sequence

1. `focusa_project_identity`
2. `focusa_workpoint_checkpoint`
3. `focusa_evidence_capture`
4. `focusa_tool_doctor`

Current operator steering, verified project scope, and canonical Workpoint authority remain higher priority than this default sequence.

## Failure recovery

- `focusa_tool_doctor`
- `focusa_workpoint_resume`
- `focusa_evidence_capture`

Treat `blocked`, `pending`, `degraded`, `canonical=false`, validation rejection, and ambiguous side effects as recovery states—not completion.

## Done condition

The requested lifecycle state passes end-to-end proof with rollback/preservation evidence.

Stable evidence or receipt refs must support any completion claim.

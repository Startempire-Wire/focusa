# Focusa Session Recovery Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_workpoint_resume -> focusa_trajectory_resume
focusa_trajectory_resume -> focusa_session_transfer
focusa_session_transfer -> focusa_tree_snapshot_state
focusa_tree_snapshot_state -> focusa_tree_restore_state
```

## Minimal path

1. Call `focusa_workpoint_resume` with only required bounded inputs.
2. Call `focusa_trajectory_resume` with only required bounded inputs.
3. Call `focusa_session_transfer` with only required bounded inputs.
4. Call `focusa_tree_snapshot_state` with only required bounded inputs.
5. Call `focusa_tree_restore_state` with only required bounded inputs.

## Current domain procedure

1. Checkpoint both Workpoint and Trajectory before compaction, context overflow, model switch, or session rollover.
2. Preserve cache-safe stable prefix plus newest operator tail; transcript tail is never continuation authority.
3. After bounded transport retry exhaustion, use governed automatic rollover and verify the canonical resume packet.

## Branches

- Unknown tool/schema: `focusa_tool_search` → `focusa_tool_describe`.
- Scope conflict: `focusa_project_verify` → `focusa_workpoint_checkpoint`.
- Daemon/degraded state: `focusa_tool_doctor`; retry only with safe posture.
- Resource timeout: `focusa_resource_mode` → bounded `focusa_traverse`.
- Browser failure: UIAI diagnostics → `focusa_browser_diagnostics_intake` → evidence.
- Mutation ambiguity: inspect side effects/receipts before retry; require operator confirmation when declared.

## Evidence and closure

- Capture stable file/test/API/browser/receipt refs.
- Link proof to the active Workpoint.
- Evaluate relevant predictions and reusable learning only after outcome is known.
- Done: A canonical Workpoint packet supplies mission, action, evidence, blockers, and exact next action.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

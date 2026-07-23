# Focusa Project Scope Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_project_identity -> focusa_project_verify
focusa_project_verify -> focusa_active_object_resolve
focusa_active_object_resolve -> focusa_workpoint_resume
```

## Minimal path

1. Call `focusa_project_identity` with only required bounded inputs.
2. Call `focusa_project_verify` with only required bounded inputs.
3. Call `focusa_active_object_resolve` with only required bounded inputs.
4. Call `focusa_workpoint_resume` with only required bounded inputs.

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
- Done: project_root plus continuity_id authority is verified and target objects resolve in that scope.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

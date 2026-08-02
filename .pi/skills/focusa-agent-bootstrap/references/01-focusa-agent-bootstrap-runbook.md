# Focusa Agent Bootstrap Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Refresh preferred address, timezone, local time, operator state, goals, constraints, desired pace, and confirmed timeline.
- Treat cwd and missing markers as weak evidence; inspect legacy project signals before suggesting creation or binding.
- Start wall-clock measurement and a human-readable bounded prediction for meaningful work; evaluate it against actual duration at completion.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_agent_card -> focusa_project_identity
focusa_project_identity -> focusa_workpoint_resume
focusa_workpoint_resume -> focusa_trajectory_view
focusa_trajectory_view -> focusa_tool_search
focusa_tool_search -> focusa_preload_build
focusa_preload_build -> focusa_context_cognition
focusa_context_cognition -> focusa_project_card
```

## Minimal path

1. Call `focusa_agent_card` with only required bounded inputs.
2. Call `focusa_project_identity` with only required bounded inputs.
3. Call `focusa_workpoint_resume` with only required bounded inputs.
4. Call `focusa_trajectory_view` with only required bounded inputs.
5. Call `focusa_tool_search` with only required bounded inputs.
6. Call `focusa_preload_build` with only required bounded inputs.
7. Call `focusa_context_cognition` with only required bounded inputs.
8. Call `focusa_project_card` with only required bounded inputs.

## Current domain procedure

1. Call `focusa_agent_card` and verify workspace version, registry digest, all-Pi-tool count, installed skills, and runbooks.
2. Use `focusa_tool_search` and `focusa_tool_describe`; never hot-load or invent the complete tool schema set.
3. Follow `docs/agent/01-focusa-agent-docs-index.md` for current architecture, lifecycle, and recovery routes.
4. For v0.9.142, build the scoped preload packet and Context Cognition view before loading broad schemas; Project Card is advisory orientation.

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
- Done: Agent has verified scope, current Workpoint/Trajectory orientation, and only the schemas needed for the next action.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

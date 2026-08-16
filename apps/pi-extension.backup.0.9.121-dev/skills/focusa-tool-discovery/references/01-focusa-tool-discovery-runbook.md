# Focusa Tool Discovery Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_tool_search -> focusa_tool_describe
focusa_tool_describe -> focusa_tool_graph
focusa_tool_graph -> focusa_tool_bundle
focusa_tool_bundle -> focusa_agent_card
```

## Minimal path

1. Call `focusa_tool_search` with only required bounded inputs.
2. Call `focusa_tool_describe` with only required bounded inputs.
3. Call `focusa_tool_graph` with only required bounded inputs.
4. Call `focusa_tool_bundle` with only required bounded inputs.
5. Call `focusa_agent_card` with only required bounded inputs.

## Current domain procedure

1. Treat `docs/contracts/spec141/generated-capability-v2/pi-tools.json` as the complete machine-readable Pi registry.
2. Use the matching `docs/focusa-tools/tools/focusa_<name>.md` reference and skill/runbook route for the selected tool.
3. Release parity requires runtime tools = contracts = Pi descriptors = per-tool docs.

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
- Done: The narrowest valid capability and dependency sequence are selected under token budget.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

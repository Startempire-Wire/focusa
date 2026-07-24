# Focusa Mission Canvas Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_call_stack_design -> focusa_context_cognition
focusa_context_cognition -> focusa_evidence_capture
focusa_evidence_capture -> focusa_active_object_resolve
```

## Minimal path

1. Call `focusa_call_stack_design` with only required bounded inputs.
2. Call `focusa_context_cognition` with only required bounded inputs.
3. Call `focusa_evidence_capture` with only required bounded inputs.
4. Call `focusa_active_object_resolve` with only required bounded inputs.

## Current domain procedure

1. Use the current Spec135 manifest for Mission Canvas, Work Rail, Work Surfaces, CRIST interviews, connectors, domain projections, and adaptive generated UI.
2. Bind UI actions to canonical operation registry entries; never create a parallel hand-coded authority path.
3. Keep UIAI browser capabilities bound to one session and origin, and preserve attachment-scoped context isolation.

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
- Done: Generated UI binds canonical operations and durable workspace evidence without semantic drift.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

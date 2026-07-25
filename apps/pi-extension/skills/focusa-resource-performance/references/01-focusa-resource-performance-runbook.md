# Focusa Resource Performance Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_resource_mode -> focusa_bloatgaurd_report
focusa_bloatgaurd_report -> focusa_bloatgaurd_tokenbloat_report
focusa_bloatgaurd_tokenbloat_report -> focusa_traverse
focusa_traverse -> focusa_tool_bundle
```

## Minimal path

1. Call `focusa_resource_mode` with only required bounded inputs.
2. Call `focusa_bloatgaurd_report` with only required bounded inputs.
3. Call `focusa_bloatgaurd_tokenbloat_report` with only required bounded inputs.
4. Call `focusa_traverse` with only required bounded inputs.
5. Call `focusa_tool_bundle` with only required bounded inputs.

## Current domain procedure

1. Use cache-safe context selection: stable prefix, current operator tail, explicit miss classification, and bounded fallback.
2. Use proactive compaction only when net token savings are positive; preserve canonical continuation packets.
3. Activate LowMem through `focusa_resource_mode` and prefer bounded traversal over full payloads.

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
- Done: Work completes inside declared memory/token/latency budgets with bounded payloads.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

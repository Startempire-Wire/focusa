# Focusa Deslop Runbook

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
focusa_tool_search -> focusa_tool_describe
focusa_tool_describe -> focusa_evidence_capture
```

## Minimal path

1. Call `focusa_tool_search` with only required bounded inputs.
2. Call `focusa_tool_describe` with only required bounded inputs.
3. Call `focusa_evidence_capture` with only required bounded inputs.

## Current domain procedure

1. Load the local deslop skill instructions before any cleanup work (canonical helpers, no renamed duplication).
2. Review diffs and existing similar code before writing new code; reject copy-paste variants.
3. Record cleanup evidence through focusa_evidence_capture with stable handles.

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
- Done: Skill file present with runbook; admitted via config/agent-skills-v2.json registry v2.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

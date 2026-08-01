# Focusa Docs Maintenance Runbook

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
focusa_tool_search -> focusa_context_cognition
focusa_context_cognition -> focusa_project_card
focusa_project_card -> focusa_evidence_capture
```

## Minimal path

1. Call `focusa_tool_search` with only required bounded inputs.
2. Call `focusa_context_cognition` with only required bounded inputs.
3. Call `focusa_project_card` with only required bounded inputs.
4. Call `focusa_evidence_capture` with only required bounded inputs.

## Current domain procedure

1. Verify typed project/workstream scope before durable mutation.
2. Refresh generated tool/operation/descriptor docs, public README/status surfaces, and all 29 skill copies together.
3. Run version, docs-runtime, Agent Card, skill hygiene, and public-surface gates; capture bounded evidence.

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
- Done: All public, machine, generated, and agent-skill surfaces agree with the current registry/version and are evidenced.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

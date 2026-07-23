# Focusa Evidence Outcomes Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_evidence_capture -> focusa_workpoint_link_evidence
focusa_workpoint_link_evidence -> focusa_predict_evaluate
focusa_predict_evaluate -> focusa_project_card_outcome
focusa_project_card_outcome -> focusa_metacog_capture
```

## Minimal path

1. Call `focusa_evidence_capture` with only required bounded inputs.
2. Call `focusa_workpoint_link_evidence` with only required bounded inputs.
3. Call `focusa_predict_evaluate` with only required bounded inputs.
4. Call `focusa_project_card_outcome` with only required bounded inputs.
5. Call `focusa_metacog_capture` with only required bounded inputs.

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
- Done: Canonical outcome status is backed by stable evidence/receipt refs and evaluated learning signals.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

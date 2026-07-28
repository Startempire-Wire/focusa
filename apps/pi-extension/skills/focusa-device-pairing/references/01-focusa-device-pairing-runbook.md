# Focusa Device Pairing Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_device_pair_start -> focusa_device_pair_qr
focusa_device_pair_qr -> focusa_device_pair_complete
focusa_device_pair_complete -> focusa_device_pair_status
focusa_device_pair_status -> focusa_device_pair_list
focusa_device_pair_list -> focusa_device_pair_revoke
```

## Minimal path

1. Call `focusa_device_pair_start` with only required bounded inputs.
2. Call `focusa_device_pair_qr` with only required bounded inputs.
3. Call `focusa_device_pair_complete` with only required bounded inputs.
4. Call `focusa_device_pair_status` with only required bounded inputs.
5. Call `focusa_device_pair_list` with only required bounded inputs.
6. Call `focusa_device_pair_revoke` with only required bounded inputs.

## Current domain procedure

1. Verify typed project/workstream scope before durable mutation.
2. Return bounded evidence and executable recovery.

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
- Done: The scoped operation is verified, evidenced, and handed to the next owning skill.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

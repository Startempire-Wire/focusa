# Focusa Install Lifecycle Runbook

## Preconditions

- Verify project root plus continuity scope when project-bound.
- Resume or checkpoint the canonical Workpoint before long/risky work.
- Confirm current operator steering and mutation approval boundaries.
- Use targeted local gates during development; CI requires explicit release authorization.

## Dependency graph

```text
focusa_project_identity -> focusa_workpoint_checkpoint
focusa_workpoint_checkpoint -> focusa_evidence_capture
focusa_evidence_capture -> focusa_tool_doctor
```

## Minimal path

1. Call `focusa_project_identity` with only required bounded inputs.
2. Call `focusa_workpoint_checkpoint` with only required bounded inputs.
3. Call `focusa_evidence_capture` with only required bounded inputs.
4. Call `focusa_tool_doctor` with only required bounded inputs.

## Current domain procedure

1. Inspect with `bash scripts/install-focusa.sh --dry-run --eval`, then install or repair by rerunning the signed public bootstrapper.
2. Use `focusa update --help` and release metadata for trusted OTA/update verification and rollback evidence.
3. Public `--uninstall` preserves user data by default; destructive removal requires explicit `--uninstall --purge-data`.
4. After each lifecycle transition, verify health, version, Pi tool discovery, Mission Canvas, and Workpoint resume.

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
- Done: The requested lifecycle state passes end-to-end proof with rollback/preservation evidence.

## Cross-harness mapping

Resolve equivalent Pi, MCP, OpenAI, CLI, and REST bindings through Agent Capability Descriptor V2; semantics and authority must remain identical.

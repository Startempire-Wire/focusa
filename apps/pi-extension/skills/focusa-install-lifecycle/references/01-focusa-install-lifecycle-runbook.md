# Focusa Install Lifecycle Runbook

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

1. Inspect with `focusa install --preflight --json` and release update-status/plan commands; install or repair only through the authority-issued signed release flow.
2. Use `focusa update --help` and release metadata for trusted OTA/update verification and rollback evidence.
3. Public `--uninstall` preserves user data by default; destructive removal requires explicit `--uninstall --purge-data`.
4. After each lifecycle transition, verify health, version, Pi tool discovery, Mission Canvas, and Workpoint resume.
5. Read Spec 152, Spec 150A, Spec 152A, and the Spec 152 supersession matrix before install/license/evaluator/UIAI/protected-component work.
6. Every official runtime, including Evaluation, requires an authority-issued signed lease; invalid, expired, or revoked entitlement means recovery posture only.
7. Never recommend the legacy Bash/PowerShell --eval path; use authority device-code, signed lease verification, lifecycle preflight, and explicit recovery/doctor/repair routes.
8. Pairing, local API, Pi, extension, provider, and UIAI tokens authenticate callers but do not create product entitlements; public uninstall preserves data unless explicit purge is requested.
9. Target entitled path: verify exact release identity, resolve a signed lease/device-code grant, validate product/node/sequence/time/features/limits, stage artifacts, activate atomically, then pair devices and create the first Workpoint.
10. Recovery path remains available for health, signed metadata inspection, backup/export, repair, rollback, and data-preserving uninstall without substituting a local license for authority.

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

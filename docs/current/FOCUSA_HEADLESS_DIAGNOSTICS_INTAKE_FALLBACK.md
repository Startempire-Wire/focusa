# Focusa Headless Diagnostics Intake Fallback

**Status:** implemented for `focusa-877z.8.8`.

Preferred Pi route remains `focusa_browser_diagnostics_intake`. Headless API/CLI/MCP/CI callers use `scripts/focusa-headless-diagnostics-intake` to convert UIAI `ResearchDiagnosticsPacket`/diagnostics JSON into lower-level Focusa capture choreography.

## Command

```bash
scripts/focusa-headless-diagnostics-intake packet.json --json
```

## Source behavior and runtime verification

This describes the source adapter; verify installed invocation/schema and exact binding through [the shared journey](../agent/02-focusa-cohesive-project-flow.md). Capture/link can record scoped evidence, but does not establish the truth of an untrusted browser payload or settle the requested outcome.

## Guarantees

- No modal/select/input UI.
- Emits JSON with `capture_status`, `scope.scope_status`, `scope.scope_source`, `headless_next_action`, `fallback_commands`, and `tool_result_v1`.
- Marks output `canonical=false` and `advisory=true` until `focusa_evidence_capture`, `focusa_browser_diagnostics_intake`, or `focusa_workpoint_link_evidence` succeeds.
- Blocks with `scope_verification_required` when project root or continuity id is missing.

## Fallback choreography

1. Verify scope with `focusa_project_identity` / `focusa_workpoint_resume` when missing.
2. Capture stable packet evidence via `focusa_evidence_capture`.
3. Resolve target object via `focusa_active_object_resolve`.
4. Record follow-up prediction if needed.

## Rule

A UIAI packet remains source-attributed browser evidence. Focusa capture/link requires exact Scope/Workstream/continuity/attachment and applicable operation admission; root/continuity inputs alone are insufficient. Successful capture records evidence, not independent verification or whole-outcome acceptance.

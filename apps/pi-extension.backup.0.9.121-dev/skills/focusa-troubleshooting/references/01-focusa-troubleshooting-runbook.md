# Focusa Troubleshooting Runbook

## Triage

1. `focusa_tool_doctor` for daemon, contract, Workpoint, resource, and browser readiness.
2. `focusa_project_identity`/`focusa_project_verify` for scope conflicts.
3. `focusa_state_hygiene_doctor` for duplicate or stale canonical signals.
4. `focusa_resource_mode` for pressure; use bounded traversal rather than full dumps.
5. Use UIAI diagnostics and `focusa_browser_diagnostics_intake` for browser failures.

## Rules

- Treat live/source contract drift, `canonical=false`, and degraded packets as explicit blockers.
- Retry mutations only with known side effects, idempotency, and authority.
- Record a failure with component plus diagnosis and capture stable recovery evidence.
- Search all 112 Pi tools with `focusa_tool_search`; do not invent tool names or parameters.

## Done condition

The root cause is isolated, recovery is proven, canonical state is coherent, and the Workpoint carries the evidence and next safe action.

# `focusa_device_pair_status`

Check the status of a pending or completed pairing by code OR by device_id. Returns the token (when completed) + status + scopes + expires_at. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Check pairing status by code or device_id. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Mac menubar OAuth-like device pairing (focusa-ui0y). Check pairing status by code or device_id.
- Capability family: `session_transfer`; namespace: `focusa.session_transfer`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `code` (optional; string): Pairing code (mutually exclusive with device_id).
- `device_id` (optional; string): Device id (mutually exclusive with code).

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_device_pair_status`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_device_pair_status.md

## Anti-examples

- raw localStorage as canonical
- raw URL paste without a saved pair

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `true`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_device_pair_list` (likely_next)
- `focusa_device_pair_revoke` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_device_pair_list`, `focusa_device_pair_revoke`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:session_transfer`
- Pi: `focusa_device_pair_status`; MCP: `focusa.device.pair.status`; OpenAI: `focusa_device_pair_status`.
- CLI: `focusa device pair-status`.
- REST: `GET /v1/device/pair/status`.
- Specification: contract registry.
- Descriptor digest: `sha256:c6cacbf47d30811ed4ae1d7c84a9934ca0e24e79a21059f631b00abdcfd341bb`.

# `focusa_device_pair_revoke`

Revoke a paired device. Appends a new entry with revoked=true to the append-only JSONL ledger and removes the in-memory token. The next call from the device will be rejected with status=revoked. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Revoke a paired device; appends revoked=true to ledger. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Mac menubar OAuth-like device pairing (focusa-ui0y). Revoke a paired device; appends revoked=true to ledger.
- Capability family: `session_transfer`; namespace: `focusa.session_transfer`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `device_id` (required; string): Device id to revoke.
- `host` (optional; string): Host label (default: 'operator-host').
- `reason` (optional; string): Optional human-readable reason (audit). Stored in the ledger.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_device_pair_revoke`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{
  "device_id": "example"
}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_device_pair_revoke.md

## Anti-examples

- raw localStorage as canonical
- raw URL paste without a saved pair

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_device_pair_revoke`, `write_device_pair_revoke`
- Read-only: `false`; destructive: `true`; idempotent: `false`; open-world: `true`.
- Confirmation required: `true`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_device_pair_list` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_device_pair_list`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:session_transfer`
- Pi: `focusa_device_pair_revoke`; MCP: `focusa.device.pair.revoke`; OpenAI: `focusa_device_pair_revoke`.
- CLI: `focusa device pair-revoke`.
- REST: `POST /v1/device/pair/revoke`.
- Specification: contract registry.
- Descriptor digest: `sha256:f9b9fe3dcd8f5ac60410c139e6a6f88daff2da2518e0e8408d2bbcab40178854`.

# `focusa_device_pair_list`

List paired devices for a host (append-only JSONL ledger, scope-bounded). Returns the recent device list with name, scopes, paired_at, last_seen_at, revoked. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). List paired devices for a host (append-only JSONL ledger). It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Mac menubar OAuth-like device pairing (focusa-ui0y). List paired devices for a host (append-only JSONL ledger).
- Capability family: `session_transfer`; namespace: `focusa.session_transfer`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `host` (optional; string): Host label (default: 'operator-host').
- `limit` (optional; integer; min=1, max=200): Max records to return. Default: 50.

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_device_pair_list`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_device_pair_list.md

## Anti-examples

- raw localStorage as canonical
- raw URL paste without a saved pair

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `read_state`, `read_state`
- Read-only: `true`; destructive: `false`; idempotent: `true`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_device_pair_revoke` (likely_next)
- `focusa_session_transfer` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_device_pair_revoke`, `focusa_session_transfer`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Runbooks: `runbook:session_transfer`
- Pi: `focusa_device_pair_list`; MCP: `focusa.device.pair.list`; OpenAI: `focusa_device_pair_list`.
- CLI: `focusa device pair-list`.
- REST: `GET /v1/device/pair/list`.
- Specification: contract registry.
- Descriptor digest: `sha256:c50c6371970a5d3873f7c86590166ad02460e2d5ddcf8ad9393c71870f274030`.

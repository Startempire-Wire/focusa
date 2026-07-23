# `focusa_device_pair_start`

Mac menubar OAuth-like device pairing (Spec focusa-ui0y). Generate an 8-char pairing code (FOCUS-XXXX-XXXX, 5 min TTL). The operator runs `focusa device pair-complete <code>` on their VPS, then the Mac app polls focusa_device_pair_status to retrieve the long-lived token (30 day TTL). Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Generate an 8-char code + pair_url for VPS-side completion via CLI, QR+phone, or QR+VPS browser. It returns a typed Focusa result with bounded recovery and likely next capabilities.

## When to use

- Mac menubar OAuth-like device pairing (focusa-ui0y). Generate an 8-char code + pair_url for VPS-side completion via CLI, QR+phone, or QR+VPS browser.
- Capability family: `session_transfer`; namespace: `focusa.session_transfer`.
- Load this full contract after metadata search when exact invocation or recovery semantics are needed.

## Parameters and strict input schema

- `device_name` (optional; string): Human-readable device name (e.g. 'operator-macbook-pro'). Defaults to 'operator-device'.
- `platform` (optional; string): Platform string. Default: 'macos'.
- `daemon_base_url` (optional; string): Daemon base URL the device will reconnect to. Default: '<http://127.0.0.1:8787>'.
- `scopes` (optional; array): OAuth-like scopes. Default: ['read', 'write'].

Unknown object properties are rejected. Canonical schema: `agent-capability-descriptors.json#focusa_device_pair_start`.

## Output

Returns `focusa.tool_result.v1` through the typed Pi output envelope. Status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools are machine-readable.

## Example

```json
{}
```

Expected: Visible summary plus tool_result_v1 details; docs: docs/focusa-tools/tools/focusa_device_pair_start.md

## Anti-examples

- raw localStorage as canonical
- raw URL paste without a saved pair

## Authority, permissions, and side effects

- Scope: `{"kind":"read","route_family":"auto"}`
- Authority: `{"kind":"advisory_only"}`
- Side effects: `write_device_pair`, `write_device_pair`
- Read-only: `false`; destructive: `false`; idempotent: `false`; open-world: `true`.
- Confirmation required: `false`; preview supported: `false`.

## Failure and recovery

Declared failure classes: `scope_conflict`, `scope_mismatch`, `resource_exhausted`, `cold_path_timeout`, `hot_path_timeout`, `daemon_unavailable`, `read_model_lag`, `validation_rejected`.

- scope_conflict -> current-ask project verify/rebind before action; scope_mismatch -> checkpoint in the correct project_root+continuity_id context
- resource_exhausted|cold_path_timeout -> focusa_resource_mode plus a narrow focusa_traverse request
- canonical=false|degraded=true -> focusa_tool_doctor then retry only with safe posture

## Dependencies and workflow position

- `focusa_device_pair_status` (likely_next)
- `focusa_device_pair_list` (likely_next)
- `focusa_device_pair_qr` (likely_next)

Prerequisites: verified project_root plus continuity_id when project-bound.
Likely next: `focusa_device_pair_status`, `focusa_device_pair_list`, `focusa_device_pair_qr`.

## Skills, protocols, and source authority

- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Runbooks: `runbook:session_transfer`
- Pi: `focusa_device_pair_start`; MCP: `focusa.device.pair.start`; OpenAI: `focusa_device_pair_start`.
- CLI: `focusa device pair-start`, `focusa device pair-qr`.
- REST: `POST /v1/device/pair/start`.
- Specification: `docs/53-focusa-device-pairing-spec.md`.
- Descriptor digest: `sha256:71349cd5a326c39de2fce33f7e8589b84ae0520761e81a5fd13838ba59ae66a3`.

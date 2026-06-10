# `focusa_device_pair_list`

**Family:** `session_transfer`
**Label:** Device Pair List

## Purpose

**Mac menubar OAuth-like device pairing (focusa-ui0y).** List paired devices for a host (append-only JSONL ledger, scope-bounded). Returns the recent device list with name, scopes, paired_at, last_seen_at, revoked.

This is the **read side** of the device ledger. The append side is `focusa_device_pair_complete` (paired=false) and `focusa_device_pair_revoke` (paired=true).

## When to use

- The operator wants to see which devices are currently paired with this Focusa daemon.
- The operator wants to see which devices have been revoked (audit).
- The Mac app can render the list to show the operator "your device is paired" with a revoke button.

## Parameters

- `host` — host label (e.g. `operator-vps`, `home-mac`). Default: `operator-vps`.
- `limit` — max records to return. Default 50, max 200.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, plus:
- `count` — number of records returned
- `host` — host label
- `devices` — list of `{device_id, name, platform, host, scopes, paired_at, last_seen_at, revoked, revoked_at}`
- `next_tools`: `["focusa_device_pair_revoke", "focusa_session_transfer"]`
- `rehydrate_id` — the most recent `device_id`

The same `device_id` may appear multiple times (paired=false then paired=true on revoke). Sort by `paired_at` descending to get the latest state.

## Example

```json
{ "host": "operator-vps", "limit": 20 }
```

```text
focusa_device_pair_list ok | device pair list → count=3 host=operator-vps
ids: rehydrate_id=019ea...-...
fields: count=3 host=operator-vps advisory=true
next: focusa_device_pair_revoke → focusa_session_transfer

  - 019ea...-... name=operator-cli-final revoked=false
  - 019ea...-... name=operator-cli-final revoked=true
  - 019ea...-... name=operator-macbook-pro revoked=false
```

## Scope rules

- The `host` filter is applied; for a multi-host setup, run multiple calls with different `host` values.
- The ledger is append-only; revoked devices show as `revoked=true` with `revoked_at` populated.
- Agent runtime paths are rejected as `host` (matches the spec's `is_unsafe_agent_runtime_path_inline` rule).

## Notes

- This is a **read-only** operation; it never mutates state.
- The same `device_id` can show up twice (paired=false then paired=true on revoke). The latest record is the active state.
- For audit, the entire ledger can be inspected directly at `data/device-pairing/{host_hash}/devices.jsonl`.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful list query.

## Contract summary

- Family: `session_transfer`
- Side Effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/device/pair/list?host=...&limit=...`
- CLI commands: `focusa device pair-list --host <h> --limit <n>`
- Core surface: `Mac menubar OAuth-like device pairing (device ledger)`
- Bead: `focusa-ui0y`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_device_pair_revoke` — remove a paired device.
- `focusa_session_transfer` — save/continue the current Focusa work (orthogonal).

# `focusa_device_pair_revoke`

**Family:** `session_transfer`
**Label:** Device Pair Revoke

## Purpose

**Mac menubar OAuth-like device pairing (focusa-ui0y).** Revoke a paired device. Appends a new entry with `revoked=true` to the append-only JSONL ledger and removes the in-memory token. The next call from the device will be rejected with `status=revoked`.

This is the **delete side** of the device ledger. The read side is `focusa_device_pair_list`. The append side is `focusa_device_pair_complete` (paired=false) and `focusa_device_pair_revoke` (paired=true).

## When to use

- The operator wants to remove a paired device (lost laptop, rotation, security incident).
- The Mac app is being decommissioned.
- The token is suspected of compromise.

## Parameters

- `device_id` — device id to revoke. Required.
- `host` — host label (e.g. `operator-vps`, `home-mac`). Default: `operator-vps`.
- `reason` — optional human-readable reason (audit). Stored in the ledger.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, plus:
- `device_id` — UUID v7 (echoed)
- `host` — host label
- `revoked_at` — ISO 8601 timestamp
- `ledger_appended` — `true | false`
- `next_tools`: `["focusa_device_pair_list"]`
- `rehydrate_id` — the device_id

## Example

```json
{
  "device_id": "019ea...-...",
  "host": "operator-vps",
  "reason": "lost laptop 2026-06-09"
}
```

```text
focusa_device_pair_revoke ok | device pair revoke → device_id=019ea...-... ledger_appended=true
ids: device_id=019ea...-... rehydrate_id=019ea...-...
fields: ledger_appended=yes host=operator-vps reason=lost laptop 2026-06-09 advisory=true
next: focusa_device_pair_list
```

## Scope rules

- The `host` filter is applied; cross-host revokes return `not_found`.
- Agent runtime paths are rejected as `host` (matches the spec's `is_unsafe_agent_runtime_path_inline` rule).
- Revocation is **append-only**: the original `paired=true` entry remains in the JSONL ledger; the new `revoked=true` entry supersedes it. `focusa_device_pair_list` returns both, sorted by `paired_at` descending.

## Notes

- The in-memory token map is invalidated immediately; the next call from the device with the same token returns `status=revoked`.
- The same `device_id` may be re-paired later; the new entry is appended with `paired_at` updated.
- For audit, the entire ledger can be inspected directly at `data/device-pairing/{host_hash}/devices.jsonl`.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `device_id_missing` — provide `device_id` and retry.
- `pair_device_not_found` — the device is unknown or already revoked.
- `scope_mismatch` — the `host` is an agent runtime path.
- `storage_unwritable` — the daemon can't append to the ledger; check daemon logs.

When `failure_class` is missing, treat the response as a successful revocation; verify with `focusa_device_pair_list`.

## Contract summary

- Family: `session_transfer`
- Side effects: `write_device_pair_revoke` (in-memory token invalidate + JSONL ledger append)
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/device/pair/revoke`
- CLI commands: `focusa device pair-revoke --device-id <id> --host <h> --reason <r>`
- Core surface: `Mac menubar OAuth-like device pairing (revoke)`
- Bead: `focusa-ui0y`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_device_pair_list` — see all paired devices, including revocations.

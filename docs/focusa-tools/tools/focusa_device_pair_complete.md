# `focusa_device_pair_complete`

**Family:** `session_transfer`
**Label:** Device Pair Complete

**Architecture spec:** [`docs/53-focusa-device-pairing-spec.md`](../../53-focusa-device-pairing-spec.md)

## Purpose

**Mac menubar OAuth-like device pairing (focusa-ui0y).** Run on the **VPS side** to complete a pending pairing initiated by `focusa_device_pair_start`. Returns the long-lived token (30-day TTL) that the Mac app will use for subsequent calls. Appends a `DeviceRecord` (revoked=false) to the append-only JSONL ledger.

This tool is invoked in three handoff modes (see [§3 of the pairing spec](../../53-focusa-device-pairing-spec.md#3-handoff-modes)):

- **Mode A (CLI):** the operator runs `focusa device pair-complete <code>` over SSH.
- **Mode B (QR + phone):** the operator scans a QR on the Mac, opens `pair_url` on the phone, and the focusa-pairing PWA helper page calls this tool.
- **Mode C (QR + VPS browser):** same as B but a kiosk/second device scans the QR.

In all three modes, the body of the call is identical — only the transport differs.

## When to use

- The operator is on the VPS (or anywhere with daemon access) and has the `FOCUS-XXXX-XXXX` code from `focusa_device_pair_start`.
- The Mac app should NOT be running this; the VPS runs it once and the Mac app polls `focusa_device_pair_status` to retrieve the token.
- Single-use: re-running with the same code returns `pair_code_already_used`; the daemon does not issue a second token.

## Parameters

- `code` — the `FOCUS-XXXX-XXXX` code from `focusa_device_pair_start`. Required.
- `host` — host label (e.g. `operator-vps`, `home-mac`). Default: `operator-vps`; sanitized to a bounded safe label and checked against unsafe agent runtime paths.
- `operator_id` — operator id (e.g. `verious`). Optional; sanitized to a bounded safe label.
- `completed_by` — who/what completed the pairing. Default: `vps-cli`; sanitized to a bounded safe label.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, plus:
- `code` — the code (echoed)
- `device_id` — UUID v7
- `device_name` — the human name from `focusa_device_pair_start`
- `host` — host label
- `scopes` — scopes granted
- `token` — 32-byte CSPRNG token encoded as base64url-no-pad (30-day TTL)
- `token_expires_at` — ISO 8601 timestamp
- `next_tools`: `["focusa_device_pair_status", "focusa_device_pair_list"]`
- `rehydrate_id` — the device_id

## Example

```json
{
  "code": "FOCUS-019EA...-...",
  "host": "operator-vps",
  "operator_id": "verious",
  "completed_by": "vps-cli"
}
```

```text
focusa_device_pair_complete ok | device pair complete → token issued for device_id=019ea...-...
ids: device_id=019ea...-... rehydrate_id=019ea...-... token=019ea...-...
fields: host=operator-vps operator_id=verious token_ttl_secs=2592000 advisory=true
note: mac app: the on_your_vps_run response is for the operator; the mac app reads the token from focusa_device_pair_status after the operator runs this command on the VPS.
next: focusa_device_pair_status → focusa_device_pair_list
```

## Scope rules

- The `code` must be in `Pending` state and not expired (5-minute TTL from `pair-start`).
- The `host` is recorded in the ledger and is used for the `pair-list` and `pair-revoke` filters; it must not be an agent runtime path.
- The token is **long-lived** (30 days), generated from 32 bytes of CSPRNG entropy, and base64url-no-pad encoded; `pair-revoke` is the only way to invalidate it before then.

## Notes

- The completion is idempotent at the API level: re-running with the same code returns `{"status":"already_completed","failure_class":"pair_code_already_used"}` and the daemon does NOT issue a new token.
- The `DeviceRecord` is appended to `data/device-pairing/{host_hash}/devices.jsonl` — append-only, scope-bounded, replay-friendly.
- The in-memory token map is invalidated by `focusa_device_pair_revoke`.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `code_missing` — provide `code` and retry.
- `pair_code_not_found` — the code is unknown; check spelling.
- `pair_code_expired` — the code is older than 5 minutes; re-run `focusa_device_pair_start` to get a fresh code.
- `pair_code_already_used` — the code was already completed; poll `focusa_device_pair_status` for the existing token.
- `scope_mismatch` — the `host` is an agent runtime path.
- `storage_unwritable` — the daemon can't append to the ledger; check daemon logs.

When `failure_class` is missing, treat the response as a successful completion; verify with `focusa_device_pair_list`.

## Contract summary

- Family: `session_transfer`
- Side effects: `write_device_pair_complete` (in-memory token insert + JSONL ledger append)
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/device/pair/complete`
- CLI commands: `focusa device pair-complete`
- Core surface: `Mac menubar OAuth-like device pairing (long-lived token, 30-day TTL)`
- Bead: `focusa-ui0y`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_device_pair_status` — verify completion + retrieve token.
- `focusa_device_pair_list` — see all paired devices.

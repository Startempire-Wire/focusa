# `focusa_device_pair_status`

**Family:** `session_transfer`
**Label:** Device Pair Status

**Architecture spec:** [`docs/53-focusa-device-pairing-spec.md`](../../53-focusa-device-pairing-spec.md)

## Purpose

**Mac menubar OAuth-like device pairing (focusa-ui0y).** Check the status of a pending or completed pairing by **code** or by **device_id**. Returns the token (when completed) + status + scopes + expires_at. This endpoint is also hit by the focusa-pairing PWA helper page (served at `GET /pair/{device_id}`) so the phone can show "Pairing done — return to your Mac."

The Mac app calls this in a poll loop after `focusa_device_pair_start` to detect completion. The first call that returns `status=completed` + a non-null `token` is the signal to store the token in the macOS Keychain.

## When to use

- The Mac app is polling after `focusa_device_pair_start` and the operator may have already completed the pairing on the VPS.
- The operator wants to look up the long-lived token for a known `device_id`.

## Parameters

- `code` — pairing code (mutually exclusive with `device_id`).
- `device_id` — device id (mutually exclusive with `code`).

## Expected result

Returns `tool_result_v1` with `ok`, plus:
- `status` — one of `pending | completed | expired`
- `code` or `device_id` (echoed)
- `token` — populated when `status=completed`
- `expires_at` — for the code (5 min) or the token (30 days)
- `expired` — `true | false`
- `next_tools`: `["focusa_device_pair_list", "focusa_device_pair_revoke"]`
- `rehydrate_id` — the device_id

## Example

```json
{ "code": "FOCUS-019EA...-..." }
```

```text
focusa_device_pair_status ok | device pair status → status=completed
ids: device_id=019ea...-... rehydrate_id=019ea...-...
fields: status=completed token=019eb...-... expired=no advisory=true
next: focusa_device_pair_list → focusa_device_pair_revoke
```

If the code is still pending:

```text
fields: status=pending token=none expired=no
```

If the code has expired (past 5 minutes):

```text
fields: status=expired token=none expired=yes
```

## Scope rules

- One of `code` or `device_id` is required.
- The `code` lookup is case-insensitive (the daemon uppercases it).
- The `device_id` lookup returns the long-lived token (and the expiry).
- After `focusa_device_pair_revoke`, the `device_id` lookup returns `token=null` (the in-memory token was invalidated).

## Notes

- This is a **read-only** operation; it never mutates state.
- The polling loop is bounded by the 5-minute code TTL.
- The response is consistent with the underlying in-memory map and the JSONL ledger; the `device_id` path is the canonical way to look up a long-lived token.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `query_missing` — provide `code` or `device_id`.
- `pair_code_not_found` — the code is unknown.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

When `failure_class` is missing, treat the response as a successful status query.

## Contract summary

- Family: `session_transfer`
- Side effects: `read_state`
- Result envelope: `tool_result_v1`
- API routes: `GET /v1/device/pair/status?code=...&device_id=...`
- CLI commands: `focusa device pair-status --code <c>|--device-id <id>`
- Core surface: `Mac menubar OAuth-like device pairing (status query)`
- Bead: `focusa-ui0y`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_device_pair_list` — see all paired devices.
- `focusa_device_pair_revoke` — remove a paired device.

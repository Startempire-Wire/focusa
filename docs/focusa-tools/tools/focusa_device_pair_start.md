# `focusa_device_pair_start`

**Family:** `session_transfer`
**Label:** Device Pair Start

**Architecture spec:** [`docs/53-focusa-device-pairing-spec.md`](../../53-focusa-device-pairing-spec.md)

## Purpose

**Mac menubar OAuth-like device pairing (focusa-ui0y).** Generate an 8-char pairing code (format `FOCUS-XXXX-XXXX`, 5-minute TTL) that the operator delivers to the VPS via one of three handoff modes (CLI / QR + phone / QR + VPS browser). The VPS runs `focusa device pair-complete <code>` to mint a long-lived token (30-day TTL); the Mac app polls `focusa_device_pair_status` to retrieve the token and store it in the macOS Keychain (via Tauri API).

## When to use

- The operator wants to connect the Focusa Mac menubar app to a remote Focusa daemon.
- The operator is OK with the dumb-simple UX: mac app shows a code, operator runs one CLI command on the VPS, mac app polls and stores the token.
- For deeper control, the operator can skip the Mac UI and call this tool + the others directly from a Pi session.

## Parameters

- `device_name` — human-readable device name (e.g. `operator-macbook-pro`). Default: `operator-device`.
- `platform` — platform string. Default: `macos`.
- `daemon_base_url` — daemon base URL the device will reconnect to. Default: `http://127.0.0.1:8787`.
- `scopes` — OAuth-like scopes. Default: `["read", "write"]`.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, plus:
- `code` — the `FOCUS-XXXX-XXXX` pairing code
- `device_id` — UUID v7
- `expires_in_secs` — always `300` (5 minutes)
- `operator_handoff.on_your_vps_run` — the exact command the operator runs on the VPS
- `pair_url` — full URL the operator's phone can open (built from `FOCUSA_PAIRING_URL` env or `daemon_base_url`)
- `pair_url_qr_payload` — byte-equal to `pair_url` in this version (forward-compat invariant)
- `next_tools`: `["focusa_device_pair_status", "focusa_device_pair_list", "focusa_device_pair_qr"]`
- `rehydrate_id` — the code

See [§4 of the pairing spec](../../53-focusa-device-pairing-spec.md#4-the-pair_url-field-new-in-pair_start) for the `pair_url` semantics, portability, and multi-tenant guarantees.

## Example

```json
{
  "device_name": "operator-macbook-pro",
  "platform": "macos",
  "daemon_base_url": "http://127.0.0.1:8787"
}
```

```text
focusa_device_pair_start ok | device pair start → code=FOCUS-019EA...-... device_id=019ea... expires_in=300s
ids: code=FOCUS-019EA...-... device_id=019ea...-... rehydrate_id=FOCUS-019EA...-...
fields: expires_in_secs=300 platform=macos on_your_vps_run=focusa device pair-complete FOCUS-019EA...-... --host <host>
note: mac app: show the code to the operator; they run the on_your_vps_run command on their VPS; mac app polls focusa_device_pair_status until completed; then store token in Keychain and reconnect.
next: focusa_device_pair_status → focusa_device_pair_list
```

## Scope rules

- `code` expires in 5 minutes; after that the daemon rejects `pair-complete` with `pair_code_expired`.
- `daemon_base_url` must be a valid http(s) URL; agent-runtime paths (e.g. `/root/pi-mono`) are rejected.
- The `device_id` is generated server-side; the mac app stores it alongside the token for subsequent re-pair/revoke flows.

## Notes

- Per Spec focusa-ui0y the pairing is **OAuth-like, dumb-simple**: the operator does NOT need to type API tokens, base URLs, or auth headers manually. The mac app handles the token storage and refresh; the operator just types the code on the VPS.
- For depth, the operator can call `focusa device pair-list` to see all paired devices, or `focusa device pair-revoke --device-id <id>` to remove one.
- For QR handoff (Mode B), use `focusa device pair-qr` — same endpoint, but the CLI output highlights `pair_url` for QR encoding.

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `device_name_missing` — provide `device_name` and retry.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.
- `scope_mismatch` — the `daemon_base_url` is an agent runtime path.

When `failure_class` is missing, treat the response as a successful pair-start; verify with `focusa_device_pair_status` after the operator runs the VPS command.

## Contract summary

- Family: `session_transfer`
- Side effects: `write_device_pair` (in-memory pending pair append)
- Result envelope: `tool_result_v1`
- API routes: `POST /v1/device/pair/start`
- CLI commands: `focusa device pair-start`
- Core surface: `Mac menubar OAuth-like device pairing (8-char code, 5-min TTL)`
- Bead: `focusa-ui0y`
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_device_pair_status` — poll whether the operator has completed the pairing.
- `focusa_device_pair_list` — see all paired devices.
- `focusa_device_pair_revoke` — remove a paired device.

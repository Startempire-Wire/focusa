# `focusa_device_pair_qr`

**Family:** `session_transfer`
**Label:** Device Pair QR

**Architecture spec:** [`docs/53-focusa-device-pairing-spec.md`](../../53-focusa-device-pairing-spec.md)

## Purpose

**Mac menubar OAuth-like device pairing with QR handoff (focusa-ui0y, Mode B).** Thin wrapper around `focusa_device_pair_start` that emphasizes `pair_url` and `pair_url_qr_payload` so the Mac menubar can render a QR the operator's phone can scan. The QR encodes the `pair_url`, which points to the focusa-pairing PWA helper page at `GET /pair/{device_id}`. The phone opens the PWA, taps **Complete on this VPS**, and the daemon mints the token. The Mac app then polls `focusa_device_pair_status` to retrieve the token (the token is never sent to the phone).

This is the **Telegram/Discord-style** QR pairing flow. See [§3 of the pairing spec](../../53-focusa-device-pairing-spec.md#3-handoff-modes) for the three handoff modes (CLI / QR+phone / QR+browser).

## When to use

- The operator wants the **dumb-simple** UX: Mac shows a QR, operator scans with phone, phone does the work.
- The operator has a public VPS hostname (e.g. `https://focusa-conn.verious.net`) and the daemon was started with `FOCUSA_PAIRING_URL` set to that URL. Otherwise `pair_url` falls back to `daemon_base_url` (default `http://127.0.0.1:8787`), which the phone can't reach from the operator's mobile network.
- The operator is OK with the phone being a passive bridge (the phone never receives the token).

## Parameters

Same as `focusa_device_pair_start`:
- `device_name` — human-readable device name. Default: `operator-device`.
- `platform` — platform string. Default: `macos`.
- `daemon_base_url` — daemon base URL the device reconnects to. Default: `http://127.0.0.1:8787`.
- `scopes` — OAuth-like scopes. Default: `["read", "write"]`.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, plus:
- `code` — the `FOCUS-XXXX-XXXX` pairing code
- `device_id` — UUID v7
- `pair_url` — full URL the operator's phone can open
- `pair_url_qr_payload` — byte-equal to `pair_url` in this version (forward-compat invariant)
- `expires_in_secs` — always `300` (5 minutes)
- `next_tools`: `["focusa_device_pair_status", "focusa_device_pair_list"]`
- `rehydrate_id` — `pair_qr:{device_id}`

See [§4 of the pairing spec](../../53-focusa-device-pairing-spec.md#4-the-pair_url-field-new-in-pair_start) for the `pair_url` semantics, portability, and multi-tenant guarantees.

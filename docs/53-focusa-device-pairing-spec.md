# Focusa Device Pairing Spec (focusa-ui0y)

**Status:** Draft (operator + agent collaboration)
**Date:** 2026-06-10
**Owns:** Mac menubar ⇄ Focusa Connect Page ⇄ VPS daemon OAuth-like device pairing
**Beads:** `focusa-ui0y.1`–`.13`, `focusa-8oc0` and children

See also: [Phone Bridge Flow Plan](54-focusa-pairing-room-plan.md).

This spec owns the full pairing architecture: code flow, QR flow, PWA helper, security model, portability, multi-tenancy, and audit. Tool-level docs (`focusa_device_pair_*`) become reference material that points here.

---

## 1. Goals

- **Mac-like + dumb simple** — first-run Mac UI shows a clean QR offer; the Focusa Connect Page scans/mediates; manual typing is Advanced fallback only. No accounts, no passwords, no OAuth flows.
- **Three-party by default** — pairing is Mac (joining device) + Focusa Connect Page (operator mediator) + VPS daemon (authority/token issuer), not a two-device flow.
- **Depth optional** — Pi/CLI agents can drive the same flow programmatically; the Mac UI is sugar.
- **Portability** — any operator with Focusa installed on their VPS (AlmaLinux, Ubuntu, macOS, containers) can pair any Mac with one URL.
- **Public-VPS safe** — the pairing endpoint can be exposed behind a public hostname (e.g. `https://focusa-conn.verious.net`) without leaking the daemon's bind address.
- **Forward-compatible** — the design leaves room for deep links, AirDrop/universal-clipboard handoff, camera-based local-network discovery, and CLI fallback without breaking the primary three-party model.

## 2. Pairing Model (operator-facing)

### 2.0 Primary model — three-party portable phone-PWA mediation

Focusa pairing is a three-party protocol:

| Party | Role | Must know initially | Learns during flow |
|---|---|---|---|
| Mac menubar app | joining device | nothing about the VPS | VPS origin, connect session, token |
| Focusa Connect Page | operator mediator/control surface | current VPS origin from `window.location.origin` | Mac handoff offer |
| VPS Focusa daemon | authority/token issuer | its own configured/public origin | Mac device record + token |

The portable first-run flow is:

```text
Mac menubar shows a short-lived QR handoff offer.
The generic phone camera is not the scanner for this QR; it will show raw JSON.
Focusa Connect Page, already loaded from the operator's VPS, scans the Mac QR inside `/connect`.
Focusa Connect Page sends the VPS origin + connect session to the Mac handoff endpoint/deep link.
Mac joins that VPS connect session and polls for completion.
Focusa Connect Page shows the Mac identity and operator taps Approve.
VPS mints a token; Mac receives it through polling and stores server+token indefinitely.
```

The Mac QR is not a server URL and is not expected to open in the phone camera app. It is a temporary handoff offer consumed by the Focusa Focusa Connect Page scanner:

```json
{
  "protocol": "focusa-connect-v1",
  "role": "mac_handoff_offer",
  "mac_name": "Verious MacBook",
  "mac_callback": "http://127.0.0.1:<ephemeral>/handoff/<nonce>",
  "mac_pubkey": "base64url...",
  "nonce": "base64url...",
  "expires_in_secs": 300
}
```

The Focusa Connect Page derives the VPS identity from its own origin, not from hardcoded configuration:

```js
const server_url = window.location.origin;
```

Then the Focusa Connect Page delivers a signed server handoff to the Mac:

```json
{
  "protocol": "focusa-connect-v1",
  "role": "server_handoff",
  "server_url": "https://this-focusa-vps.example",
  "connect_id": "01H...",
  "server_pubkey": "base64url...",
  "nonce": "same nonce or server challenge",
  "expires_in_secs": 300
}
```

If direct browser-to-Mac callback is blocked, the Focusa Connect Page must offer Apple-like fallbacks in this order:

1. Use the Focusa Focusa Connect Page `/connect` scanner; generic camera scanning is not a success path for Mac-offer JSON.
2. Open `focusa://connect?...` deep link.
3. Share/AirDrop the same deep link to the Mac.
4. Copy link.
5. Advanced manual server URL/code.

Manual URL fields, device names, CLI commands, and diagnostics are always Advanced/fallback UI, never the first-run primary screen.

### 2.0.1 Connect-session API contract

The three-party flow uses short-lived connect sessions before falling back to the older
`device/pair/*` code flow:

| Route | Caller | Purpose |
|---|---|---|
| `POST /v1/connect/start` | Focusa Connect Page after scanning Mac QR | create a rendezvous from the Mac handoff offer and return a `server_handoff` |
| `GET /v1/connect/status?connect_id=...` | Mac | poll until the phone approves and the VPS mints a token |
| `POST /v1/connect/approve` | Focusa Connect Page | operator approval; VPS mints token and appends the device record |

`connect/start` input accepts `mac_name`, `mac_nonce`, optional `mac_pubkey`, optional
`mac_callback`, optional `server_url`, and optional scopes. The VPS chooses the public
server URL from `FOCUSA_PAIRING_URL` first, then request `server_url`, then local default.

`connect/status` returns pending/expired/completed plus the token only after approval.
`connect/approve` is the only route that mints a token.


Three actors:

| Actor | Role |
|---|---|
| **Mac** | The device being paired. Holds a `device_id` after pairing. Stores a `DeviceToken` in macOS Keychain. |
| **VPS / Daemon** | The trust root. Owns the `devices.jsonl` ledger. Mints tokens, validates them, supports revoke. |
| **Phone / Browser** (optional) | The bridge that scans the QR and hits the public pairing URL. Does NOT receive a token. |

### 2.1 Trust anchor

The **code** is the trust anchor. It is:

- 8 chars, format `FOCUS-XXXX-XXXX`, generated server-side
- Single-use (consumed on pair-complete)
- 5-minute TTL
- Bound to a `device_id` (UUIDv7) and a `device_name`

The **token** is what the Mac uses thereafter:

- 32-byte random, base64url
- 30-day TTL (renewable)
- Stored in macOS Keychain (Tauri `keyring` API)
- Sent as `Authorization: Bearer <token>` on all subsequent calls
- Revocable from VPS side via `focusa device pair-revoke`

### 2.2 Pairing invariants (portable across hosts)

These must hold for every Focusa install:

1. The code is the only thing the operator ever types. **No passwords, no usernames, no email loops.**
2. The Mac never needs to know the VPS bind address. It only needs the `daemon_base_url` returned in `pair_start`.
3. The VPS never needs to know the Mac's network. It only needs the code.
4. The `devices.jsonl` ledger is the canonical audit trail. It is append-only; revocation is a new entry with `revoked=true`.
5. Token expiry is enforced server-side. The Mac may cache; the daemon revalidates on every call.

## 3. Handoff Modes

The code can be delivered to the VPS three ways. All three produce the same `DeviceRecord` in the ledger.

### 3.1 Mode A — CLI fallback (current, working)

```
Mac → POST /v1/device/pair/start → {code, on_your_vps_run}
Operator runs on VPS:  focusa device pair-complete FOCUS-XXXX-XXXX
VPS → POST /v1/device/pair/complete → {device_id, token}
Mac polls GET /v1/device/pair/status?code=... → {token, expires_at}
```

**Pros:** works over SSH, no network exposure, dumb simple.
**Cons:** operator must ssh to VPS.

### 3.2 Mode B — QR + phone browser (Telegram/Discord-style)

```
Mac → POST /v1/device/pair/start → {code, pair_url, pair_url_qr_payload}
Mac renders QR encoding pair_url.
Phone scans QR → opens pair_url in browser.

This legacy mode only works after the Mac already knows the correct VPS URL or public pairing URL. It is not the portable first-run default.
Browser shows tiny focusa-pairing helper page (PWA).
Operator on phone taps "Complete on this VPS" → helper page POSTs to
        /v1/device/pair/complete using the code embedded in the URL.
Mac polls GET /v1/device/pair/status?code=... → {token, expires_at}
```

**Pros:** zero ssh, phone does the work, the phone DOES NOT receive the token.
**Cons:** requires the VPS to be reachable from the phone's network (i.e., a public hostname or a tunnel).

### 3.3 Mode C — QR + VPS browser (no phone, same network)

Same as Mode B but the operator's Mac or a kiosk scans the QR and the VPS browser completes. Useful when the Mac and VPS share a LAN but the operator is at the keyboard of a different device.

### 3.4 Why both?

| Operator situation | Best mode |
|---|---|
| VPS behind NAT, operator has SSH | A (CLI) |
| First-run Mac does not know VPS, Focusa Connect Page can access VPS | Primary three-party phone-PWA mediation (§2.0) |
| VPS has a public URL and Mac already knows it | B (server-generated QR + phone fallback) |
| Mac and VPS on same LAN, Mac not in front of operator | C (QR + kiosk) |
| Pi/agent driving the pairing | A via CLI / programmatic |

**The code/token ledger is the same in all fallback modes.** The primary three-party flow adds a connect-session rendezvous before device pairing so the Mac can learn the VPS without manual typing.

## 4. The `pair_url` field (new in `pair_start`)

`POST /v1/device/pair/start` now returns:

```json
{
  ...existing fields...,
  "pair_url": "https://focusa-conn.verious.net/pair/019ea...",
  "pair_url_qr_payload": "https://focusa-conn.verious.net/pair/019ea..."
}
```

The URL is built from `FOCUSA_PAIRING_URL` env var if set, else from `daemon_base_url`. This lets an operator with a public VPS host run:

```bash
FOCUSA_PAIRING_URL=https://focusa-conn.verious.net focusa-daemon
```

…and every `pair_start` will return a URL the phone can hit even if the Mac would otherwise see `http://127.0.0.1:8787` (which it can't, from the phone's network).

**`pair_url` and `pair_url_qr_payload` are identical today.** They're separate fields so the encoding can diverge later (e.g., compact base32 for tight QR, or signed JWS for tap-to-pair).

## 5. The PWA Helper Page

Served at `GET /pair/{device_id}`. Loads:

- A 200-LOC HTML page with focusa-pairing branding
- `manifest.json` (PWA install — operator can add to home screen)
- `service-worker.js` (offline shell)
- Inline JS that reads `device_id` from the URL, calls `GET /v1/device/pair/status?code=...` (or directly `POST /v1/device/pair/complete` if the operator confirms the device name on screen), and shows one of three states:

  1. **Pending** — code is alive; "Tap 'Complete on this VPS' to finish"
  2. **Completed** — token was minted; "Pairing done. Return to your Mac."
  3. **Expired** — code is gone; "Generate a new code on your Mac."

The PWA **does NOT receive the token**. It only confirms completion. The Mac polls `pair_status` and retrieves the token through its existing daemon connection.

### 5.1 Why no camera (yet)?

The PWA can later be extended with `getUserMedia` to scan a **different** QR — for the **reverse** flow where the VPS shows a QR and the phone scans it. That's Mode D and is forward-compatible. Not in this spec.

### 5.2 Threat model for the PWA

| Threat | Mitigation |
|---|---|
| Attacker hits `/pair/{device_id}` with a guessed id | UUIDv7 is unguessable; rate-limit anyway |
| Attacker completes pair-complete from a different origin | Helper page lives on the same origin as the daemon; same-origin policy applies |
| Attacker XSSes the helper page | Page is 200 LOC, no external assets, no third-party scripts |
| Attacker MITM the phone → VPS connection | HTTPS at the public hostname; certificate pinning is operator's responsibility |
| Operator scans a malicious QR | The code is server-generated and bound to a device_id; scanning a foreign code just opens a benign page |

## 6. Portability + Multi-Tenancy

### 6.1 What "any operator can use it" means

A second operator with their own Focusa install (say `focusa-jane.example.com`) must be able to:

1. Set `FOCUSA_PAIRING_URL=https://focusa-jane.example.com` (or whatever their public hostname is).
2. Run `focusa-daemon` — daemon serves `/pair/{device_id}` on that hostname.
3. Pair their Mac — phone scans QR, hits `https://focusa-jane.example.com/pair/...`, completes.

**No code changes, no shared state, no global registry.** Each daemon is its own trust root.

### 6.2 What changes in the daemon

- `FOCUSA_PAIRING_URL` env var (new, optional)
- `GET /pair/{device_id}` route (new, returns the PWA shell)
- `GET /pair/{device_id}/manifest.json` (new, PWA manifest)
- `GET /pair/{device_id}/sw.js` (new, service worker)

These are **additive**. Existing installations with `FOCUSA_PAIRING_URL` unset fall back to `daemon_base_url` (i.e., `http://127.0.0.1:8787` by default), and the QR is still scannable from the Mac's local network.

### 6.3 Multi-device is a property of the ledger

`GET /v1/device/pair/list?host=...` returns all `DeviceRecord`s for the operator. Each Mac has its own `device_id` and `device_name`. Revoking one does NOT affect others. **The list is the multi-tenant boundary** — a single daemon trusts the devices it has been told to trust.

## 7. Forward Compatibility

This spec leaves room for:

| Future | How |
|---|---|
| **Reverse pairing** (VPS shows QR, phone scans) | New `pair_url` mode where the VPS displays a QR encoding `https://focusa-phone.example.com/pair/{code}`; phone opens, gets connected. Same `pair-complete` API. |
| **Local-network discovery** | PWA gets `getUserMedia` + a Tauri-side beacon. Out of scope for this spec. |
| **mTLS** | Replace `Authorization: Bearer` with mTLS. Same `DeviceRecord` shape. |
| **Tap-to-pair** (NFC / BLE) | PWA helper could surface a Web NFC handler. Same `pair_url` field, different transport. |
| **Code → token directly** (skip VPS CLI) | PWA helper could surface "complete on this device" if the phone is on the same LAN as the Mac's daemon. Out of scope. |

## 8. Failure Modes

| Failure | User-visible | Recovery |
|---|---|---|
| Code expires before completion | Mac shows "Code expired — generate a new one" | Mac calls `pair_start` again, new code |
| Phone can't reach VPS | PWA shows "Network error" with retry | Operator falls back to Mode A (SSH + CLI) |
| Two Mac apps generate codes for the same device_name | VPS keeps both `DeviceRecord`s, both can be revoked independently | List + revoke the unwanted one |
| Token leaks | Operator runs `focusa device pair-revoke --device-id <id>` | Mac must re-pair; old token returns 401 |
| `FOCUSA_PAIRING_URL` is misconfigured | PWA opens on a host the phone can't reach | Operator falls back to CLI; URL is documented in `pair_start` response for debugging |
| Daemon restart loses in-memory pending codes | `pair_start` returns a fresh code on retry | New code, new `device_id` |

## 9. Audit (extends `focusa-ui0y.6`)

The existing audit covers:

- Code format, TTL, single-use
- Token expiry, rotation, revocation
- Append-only ledger integrity
- Code format is `FOCUS-[A-Z0-9]{4}-[A-Z0-9]{4}` (no `0/O/1/I` ambiguity)

The audit must additionally cover:

- `pair_url` is built from `FOCUSA_PAIRING_URL` when set, else `daemon_base_url`
- `pair_url` URL-encodes the `device_id` (UUIDs are URL-safe but assert it)
- PWA helper page does not leak the token
- PWA helper page does not include third-party scripts
- `pair_url_qr_payload` is byte-equal to `pair_url` in this version (forward-compat invariant)
- Multi-tenant: a code generated on operator-A's daemon cannot be completed on operator-B's daemon (the `device_id` is a UUID, not a global key)

## 10. End State (this spec ships when)

1. `pair_start` returns `pair_url` and `pair_url_qr_payload` (done)
2. Daemon serves a working PWA helper at `GET /pair/{device_id}` (bead `focusa-ui0y.8`)
3. Menubar renders a QR from `pair_url` and surfaces Mode A/B/C choices (bead `focusa-ui0y.9`, `.10`)
4. CLI has `focusa device pair qr` shortcut (bead `focusa-ui0y.11`)
5. Pi has `focusa_device_pair_qr` tool (bead `focusa-ui0y.12`)
6. Audits extended (bead `focusa-ui0y.13`)
7. Docs: this spec + tool-level updates in `docs/focusa-tools/tools/`
8. End-to-end test: a fresh operator on a fresh VPS can pair their Mac using only `FOCUSA_PAIRING_URL=https://...` and a phone camera

## 11. Bead decomposition (proposed)

| Bead | Surface | Work |
|---|---|---|
| `focusa-ui0y.1` | core | Types (DONE) |
| `focusa-ui0y.2` | api | Routes (DONE) |
| `focusa-ui0y.3` | cli | Subcommands (DONE) |
| `focusa-ui0y.4` | pi | Tools (DONE) |
| `focusa-ui0y.5` | menubar | Initial UI (DONE) |
| `focusa-ui0y.6` | audits | Static tests (OPEN) |
| `focusa-ui0y.7` | api | `pair_url` field + `FOCUSA_PAIRING_URL` env (in flight) |
| `focusa-ui0y.8` | api | PWA helper page at `GET /pair/{device_id}` |
| `focusa-ui0y.9` | menubar | QR renderer component (using `qrcode` lib) |
| `focusa-ui0y.10` | menubar | Pairing panel redesign (Mode A/B/C tabs) |
| `focusa-ui0y.11` | cli | `focusa device pair qr` shortcut |
| `focusa-ui0y.12` | pi | `focusa_device_pair_qr` tool |
| `focusa-ui0y.13` | audits | QR + PWA + multi-tenant tests (extends `.6`) |

## 12. Open questions for operator

- Should `pair_url` ever be a `focusa://` deep link (opens the menubar app) instead of HTTPS? Trade-off: more seamless, but only works when the Mac is the scanning device, not the phone.
- Should the PWA helper page have a "complete on this device" button when the phone is on the same LAN as the daemon? Adds getUserMedia + LAN discovery.
- Should the audit be a static test (`tests/device_pairing_static_test.py`) or a full e2e (`scripts/pair-e2e.sh`)? My recommendation: both.

# Focusa Phone Bridge Flow Plan

**Status:** implemented plan with Context Authority hardening
**Scope:** Phone Bridge Flow across three parties: VPS/server, phone running the Focusa Connect Page, and Mac Menubar App.
**Primary command:** `focusa pair`

**Context Authority:** `focusa pair --json` includes `environment_contract`, `runtime_inventory`, and `action_preflight` so pairing initiation remains pairing initiation and cannot silently become a release-asset install workflow.

## 1. Goal

Make first-time Mac pairing feel like an Apple handoff with no manual server URL typing.

Pairing must be fully portable and host-neutral. The architecture cannot assume this VPS, cPanel, LiteSpeed, Cloudflare, Tailscale, localhost, or any single domain. It must first resolve a verified phone-reachable transport from the operator's actual environment: same-machine localhost, LAN/private IP, Tailscale/Funnel, Cloudflare Tunnel/ngrok, reverse proxy, direct VPS public URL, or future hosted Focusa. Any URL shown to an operator must be probed and reachable for both `/connect` and `/v1/connect/room/*`; otherwise the flow must offer a clear portable fallback.

Pairing must not perform or imply Focusa installation. If runtime binaries are stale on a live build host, the safe repair path is local repo build/restart, not GitHub release asset replacement.

```text
VPS shell/TUI        Focusa Connect Page                Mac menubar
-----------          ---------                -----------
focusa pair   ->     scan server QR     ->    shows Mac QR
                    opens /connect            (handoff offer)
                    scans Mac QR       ->     waits
                    taps Connect       ->     receives token
```

## 2. User story

1. Operator installs/starts Focusa on any VPS.
2. Operator runs:

   ```bash
   focusa pair
   ```

   For diagnostics or agent-driven operation:

   ```bash
   focusa --json pair
   ```

   JSON output includes transport diagnostics plus context-authority facts:

   ```json
   {
     "environment_contract": {"schema": "focusa.environment_contract.v1"},
     "runtime_inventory": {"schema": "focusa.runtime_inventory.v1"},
     "action_preflight": {"schema": "focusa.operational_context_gate.v1"}
   }
   ```

3. CLI/TUI shows a QR for a phone URL:

   ```text
   https://<this-vps>/connect/<room_id>
   ```

4. Phone opens the Focusa Connect PWA from the VPS origin.
5. Mac menubar starts a local callback listener and shows a Mac handoff QR containing the callback URL.
6. Focusa Connect Page scans the Mac handoff QR and combines:
   - room id from the server QR URL
   - server origin from `window.location.origin`
   - Mac offer from Mac QR (including callback URL)
7. Focusa Connect Page shows `Connect <Mac name>?`.
8. Operator taps Connect.
9. VPS mints token. Focusa Connect Page POSTs the Mac Completion Payload to the Mac callback URL.
10. Mac receives the completion payload automatically and stores server URL + token indefinitely until explicit disconnect.

## 3. The two QR codes

### 3.1 Server/TUI QR

Shown by `focusa pair`.

Payload is a normal URL so any phone camera can open it:

```text
https://<server>/connect/<room_id>
```

Purpose:
- gets phone into the right Bridge Room
- gives Focusa Connect Page the VPS origin
- does not identify the Mac yet

### 3.2 Mac menubar QR

Shown by the Mac app.

Payload is a handoff offer consumed by the Focusa Connect Page scanner. It may be raw JSON or a compact encoded URL understood by the room page.

```json
{
  "protocol": "focusa-connect-v1",
  "role": "mac_handoff_offer",
  "mac_name": "Verious MacBook",
  "nonce": "...",
  "mac_pubkey": "optional...",
  "mac_callback": "optional...",
  "expires_in_secs": 300
}
```

Purpose:
- identifies the Mac
- gives the room a nonce/challenge
- provides the Mac callback URL for automatic completion delivery
- never contains a long-lived token

## 4. Focusa Connect Page as the combine function

The PWA is the middle layer:

```js
server_url = window.location.origin
room_id = route param from /connect/<room_id>
mac_offer = scanned Mac QR payload
```

Then it calls:

```text
POST /v1/connect/room/<room_id>/mac-offer
POST /v1/connect/room/<room_id>/approve
```

## 5. Bridge Room backend model

Room fields:

```json
{
  "room_id": "01J...",
  "server_url": "https://vps.example",
  "status": "waiting_for_mac|mac_seen|approved|completed|expired",
  "mac_offer": null,
  "device_id": null,
  "token": null,
  "created_at": "...",
  "expires_at": "..."
}
```

## 6. API shape

Preferred final routes:

| Route | Caller | Purpose |
|---|---|---|
| `POST /v1/connect/room/start` | CLI/TUI | create Bridge Room and return `connect_url` |
| `GET /connect/<room_id>` | phone | PWA room page |
| `GET /v1/connect/room/<room_id>/status` | Mac/phone | poll room state |
| `POST /v1/connect/room/<room_id>/mac-offer` | Focusa Connect Page | submit scanned Mac QR offer |
| `POST /v1/connect/room/<room_id>/approve` | Focusa Connect Page | approve Mac and mint token |

Current transitional routes already exist:
- `GET /connect`
- `POST /v1/connect/start`
- `GET /v1/connect/status`
- `POST /v1/connect/approve`

## 7. Phone Bridge Transport Resolver

The Phone Bridge Flow needs a phone-reachable transport. That transport may be a public URL, non-local daemon URL, private/Tailscale URL, temporary tunnel, or optional reverse proxy.

The resolver is the pairing authority boundary: it must verify transport capability before any room URL is advertised. A local `127.0.0.1` URL is valid only for same-machine testing; it is never a portable cross-device URL. Host-specific integrations are optional adapters, not architecture.

`focusa pair` runs the resolver automatically. Operators should not need to run setup steps first for normal use.

Diagnostic/manual override helper:

```bash
scripts/phone-bridge-transport.sh detect
scripts/phone-bridge-transport.sh options
scripts/phone-bridge-transport.sh check --url https://focusa.example.com
sudo scripts/phone-bridge-transport.sh write --url https://focusa.example.com
```

Transport validation requires both routes to work:

```text
/connect/*    serves the Focusa Connect Page
/v1/connect/* reaches the Bridge Room API
```

Optional reverse-proxy snippets remain available, but live webserver mutation is not a default requirement:

```bash
scripts/phone-bridge-transport.sh proxy-snippets
```

`focusa pair` uses the same resolver posture automatically: start/repair daemon first, restart stale daemons when the running version does not match the CLI, then probe configured URLs, non-local API URLs, verified hostname/IP candidates, private/Tailscale candidates, and local fallback. A candidate is accepted only when both the Focusa Connect page and Bridge Room API are reachable. JSON output includes `checked_candidates`, per-route probe diagnostics, first rejection reason, daemon repair status, and operator hints; human output includes a concise rejection summary.

Daemon-side Bridge Room endpoints also report lifecycle diagnostics (`room_started`, `mac_offer_seen`, `approval_completed`), `next_step_hint`, and structured rejection payloads so Focusa Connect failures are explainable without raw stale JSON.

## 8. CLI/TUI plan

### 7.1 `focusa pair`

Default human output:

```text
Focusa Bridge Room

Open on phone:
https://<server>/connect/<room_id>

[terminal QR]

Then scan the Mac menubar QR from the Focusa Connect Page.
```

Options:

```bash
focusa pair --url https://vps.example
focusa pair --no-qr
focusa pair --json
```

URL resolution priority:
1. `--url`
2. `FOCUSA_PAIRING_URL`
3. `FOCUSA_PUBLIC_URL`
4. installer files: `/etc/focusa/pairing-url`, `/etc/focusa/public-url`, `.focusa-pairing-url`, `.focusa-public-url`
5. non-local `FOCUSA_API_URL` / `FOCUSA_BASE_URL`
6. verified hostname candidates: `https://<fqdn>`, `http://<fqdn>`, `http://<fqdn>:8787`
7. verified public IPv4 candidates: `https://<ip>`, `http://<ip>`, `http://<ip>:8787`
8. daemon local URL (`http://127.0.0.1:8787`) with setup guidance

Hostname/IP candidates count only when `GET /connect` returns the Focusa Connect page, not merely any HTTP 200. This keeps installs portable and avoids cPanel/default-site false positives.

### 7.2 TUI

Future TUI can render the same `connect_url` QR in a centered panel. No separate protocol.

## 9. UI rules

Mac first-run:
- QR first
- one sentence max
- no server URL fields unless Advanced opened
- Copy errors always available
- if generic camera scans raw Mac offer, copy must make clear to use Focusa Connect Page scanner

Focusa Connect Page:
- first screen: `Connect Mac to Focusa`
- primary button: `Scan Mac code`
- Advanced paste fallback hidden
- approval screen shows Mac name + server

## 10. Security rules

- Rooms expire quickly (default 5 min).
- Phone never receives long-lived token except as transient response body if needed for status; Mac is the intended token consumer.
- Token is minted only after explicit phone approval.
- Mac offer contains nonce and optional public key.
- Server URL comes from server origin or configured public URL, not from Mac QR.

## 11. Implementation phases

### Phase A — command surface

- Add `focusa pair`.
- Print `/connect` or `/connect/<room_id>` URL.
- Render terminal QR.

### Phase B — room API

- Add room start/status/mac-offer/approve endpoints.
- Keep transitional `/v1/connect/*` routes as compatibility.

### Phase C — Focusa Connect Page room page

- Serve `/connect/<room_id>`.
- Scan Mac QR.
- Submit offer + approve.

### Phase D — Mac room polling

- Mac joins room/status and stores token.

### Phase E — proof

- API smoke test.
- UIAI Focusa Connect Page QA.
- Menubar first-run QA.
- Release asset tag/version proof.

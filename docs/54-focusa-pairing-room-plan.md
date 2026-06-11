# Focusa Phone Bridge Flow Plan

**Status:** implementation plan
**Scope:** Phone Bridge Flow across three parties: VPS/server, phone running the Focusa Connect Page, and Mac Menubar App.
**Primary command:** `focusa pair`

## 1. Goal

Make first-time Mac pairing feel like an Apple handoff with no manual server URL typing.

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

3. CLI/TUI shows a QR for a phone URL:

   ```text
   https://<this-vps>/connect/<room_id>
   ```

4. Phone opens the Focusa Connect PWA from the VPS origin.
5. Mac menubar first screen shows a Mac handoff QR.
6. Focusa Connect Page scans the Mac handoff QR and combines:
   - room id from the server QR URL
   - server origin from `window.location.origin`
   - Mac offer from Mac QR
7. Focusa Connect Page shows `Connect <Mac name>?`.
8. Operator taps Connect.
9. VPS mints token.
10. Mac stores server URL + token indefinitely until explicit disconnect.

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

## 7. Public URL / proxy setup

The Phone Bridge Flow requires a **Public Focusa URL** before a real phone can open the Focusa Connect Page.

Installers/operators must provide one of:

```bash
export FOCUSA_PAIRING_URL=https://focusa.example.com
# or
sudo mkdir -p /etc/focusa
printf 'https://focusa.example.com\n' | sudo tee /etc/focusa/public-url
```

The public URL must reverse-proxy both routes to the local daemon:

```text
/connect/*    -> http://127.0.0.1:8787/connect/*
/v1/connect/* -> http://127.0.0.1:8787/v1/connect/*
```

Portable setup helper:

```bash
scripts/setup-phone-bridge-url.sh --url https://focusa.example.com --print-proxy
scripts/setup-phone-bridge-url.sh --url https://focusa.example.com --check
sudo scripts/setup-phone-bridge-url.sh --url https://focusa.example.com --write
```

`focusa pair` then reads the configured URL and emits a phone-scannable Bridge Room URL.

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

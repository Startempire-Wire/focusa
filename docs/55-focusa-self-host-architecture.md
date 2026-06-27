# Focusa Self-Host Architecture (v0.9.35-dev)

**Status:** Canonical. v0.9.35-dev replaces v0.9.34-dev for self-host flows.
**Owner:** Focusa daemon + menubar + CLI.
**Predecessors:** v0.9.33-dev (JSON-QR, in-app scanner), v0.9.34-dev (URL-QR, Mac-creates-room).

---

## 1. The one-paragraph summary

A self-hoster runs `curl install.focusa.dev/focusa | bash` on a VPS, then runs `focusa pairing wizard` on that VPS. The wizard detects Tailscale MagicDNS, creates a pairing room on the VPS, and prints a scannable terminal QR. The operator's phone camera opens that QR in the browser; the Focusa Connect Page (a PWA served by the VPS) loads, requests browser-camera permission, and scans a static `mac_offer` QR displayed by the Mac menubar app. The PWA posts the offer to the VPS, asks the operator to tap Approve, and the VPS mints a token. The Mac, which discovered the VPS via Tailscale MagicDNS and was already polling the room, receives the token and stores it in the Keychain. The whole flow uses zero URL paste and zero operator typing after install.

## 2. The three actors and what each owns

| Actor | Owns | Lifetime |
|---|---|---|
| **Focusa daemon** (VPS) | The room (state), the PWA source (HTML/JS), the pairing token | 5-minute room TTL, 30-day token TTL |
| **Mac menubar app** | The `mac_offer` (name, nonce, pubkey), the token (in Keychain after pairing) | Token until revoked or 30 days |
| **Phone browser** | Nothing — purely a renderer of the VPS-served PWA | Stateless per visit |

The phone is **not** a participant with persistent state. No app install, no localStorage, no service worker, no cached manifest. It opens the URL once, the VPS serves the page, the operator taps Approve, the phone forgets everything. If the operator reopens the URL, the VPS serves a fresh page reflecting current room state.

## 3. The canonical flow (six steps)

```
                ┌──────────────────────┐
                │  Mac menubar app     │
                │                      │
                │  Idle: shows static  │
                │  mac_offer QR        │
                └──────────────────────┘
                          ▲
                          │ (2) PWA browser camera scans this
                          │
   ┌──────────────────────┐    (1) phone native    ┌─────────────────────────┐
   │  VPS terminal        │       camera scans     │  Phone                  │
   │                      │ ◄──────────────────►   │                         │
   │  $ focusa pairing    │     terminal QR        │  Native camera ──────┐  │
   │    wizard            │                        │                       │  │
   │                      │                        │  ┌─────────────────┐  │  │
   │  ┌────────────────┐  │                        │  │ PWA loads       │  │  │
   │  │ ▓▓ ▓▓▓ ▓▓▓▓▓ │  │                        │  │ https://vps/    │  │  │
   │  │ ▓ ▓▓▓ ▓ ▓ ▓▓▓ │  │                        │  │   connect/room/ │  │  │
   │  │ ▓▓ ▓ ▓▓ ▓▓▓ ▓ │  │                        │  │   <id>/scan     │  │  │
   │  └────────────────┘  │                        │  └─────────────────┘  │  │
   │                      │                        │           │            │  │
   │  Contents:           │                        │  (3) PWA asks camera   │  │
   │   room_id            │                        │           │            │  │
   │                      │                        │  (4) reads mac_offer   │  │
   └──────────────────────┘                        │           │            │  │
            │                                       │  (5) tap Approve ──────┤  │
            │                                       │                       │  │
            │           ┌─────────────────────────┐│  (6) PWA POSTs to VPS  │  │
            └──────────►│  VPS / Focusa daemon    ││                       │  │
                        │                         ◄┼───────────────────────┘  │
                        │  room state:            │                        │
                        │    room_id              │                        │
                        │    mac_offer: <scanned> │                        │
                        │    status: mac_seen     │                        │
                        │                         │                        │
                        │  ◄─ PWA POSTs mac_offer│                        │
                        │  ◄─ PWA POSTs approve   │                        │
                        │      status = completed │                        │
                        │      token minted       │                        │
                        │                         │                        │
                        │  Mac (Tailscale/Bonjour │                        │
                        │   discovered VPS)       │                        │
                        │  polls room status       │                        │
                        │  ◄─ token                │                        │
                        │                         │                        │
                        │  Mac stores in Keychain  │                        │
                        └─────────────────────────┘
```

**Step 1 — VPS creates the room.** Operator runs `focusa pairing wizard` on the VPS terminal. The wizard detects Tailscale MagicDNS (or falls back to `FOCUSA_PUBLIC_URL` env, then to `http://127.0.0.1:8787` for dev), calls `POST /v1/connect/room/create`, and receives a `room_id` + a `pair_url` containing the room_id.

**Step 2 — VPS prints terminal QR.** Wizard renders `pair_url` as a Unicode-block QR (50×50 cells, ~25 lines tall) directly in the terminal. Operator picks up phone.

**Step 3 — Phone native camera scans terminal QR.** Opens browser at `https://<vps>/connect/room/<room_id>/scan`. The Focusa Connect Page PWA loads. PWA reads `room_id` from URL, requests browser-camera permission via `getUserMedia`.

**Step 4 — PWA browser camera scans Mac's static mac_offer QR.** Operator points phone at the Mac menubar. PWA camera reads the `mac_offer` payload (mac_name, mac_nonce, mac_pubkey). PWA POSTs `{mac_name, mac_nonce, mac_pubkey}` to `/v1/connect/room/<room_id>/join`.

**Step 5 — PWA shows Approve.** VPS room state flips to `mac_seen`. PWA renders "Tap Approve to pair this Mac." Operator taps Approve. PWA POSTs `{host, operator_id, completed_by}` to `/v1/connect/room/<room_id>/approve`. VPS room flips to `completed`. Token minted.

**Step 6 — Mac receives token.** Mac, which discovered the VPS via Tailscale MagicDNS (or Bonjour fallback) in the background and was already polling `/v1/connect/room/<room_id>/status` (room_id learned from the `mac_offer` it generated), sees `status=completed, token=…`. Mac writes token to Keychain. FirstRunWizard flips to `connected`. Done.

## 4. State machine: room

```
                    ┌─────────────────┐
        create      │  not_found      │      expire (5 min TTL)
       ───────────► │                 │ ─────────────────────► expired
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ waiting_for_mac │
                    └────────┬────────┘
                             │ mac joins
                             ▼
                    ┌─────────────────┐
                    │   mac_seen      │
                    └────────┬────────┘
                             │ approve
                             ▼
                    ┌─────────────────┐
                    │   completed     │
                    └────────┬────────┘
                             │ mac polls token, stores in Keychain
                             ▼
                    ┌─────────────────┐
                    │   consumed      │
                    └─────────────────┘
```

The room is created on the VPS. State transitions are atomic. The PairingStore persists every transition (durable across daemon restarts mid-pairing).

## 5. URL discovery — zero paste

The Mac app must learn the VPS URL to poll the room. Three discovery paths, in priority order:

1. **Tailscale MagicDNS** — Mac tries `http://focusa-vps:8787`, then `http://focusa:8787`, then the operator's Tailscale hostname if known. Tailscale is the recommended self-host topology.
2. **Bonjour/mDNS** — VPS daemon registers `_focusa._tcp.local` on port 8787 with TXT record containing the room_id (if active). Mac discovers on LAN.
3. **Operator-CLI handoff** — `focusa pairing show --mac` on VPS prints the room URL to stdout; operator can paste once into Mac Settings as a last-resort fallback. **This is the only place URL paste appears in the operator UI.**

There is no permanent "Server URL" field in the Mac app. There is no `PUBLIC_PAIRING_URL_KEY` localStorage key. The first-run wizard auto-discovers and forgets any URL it was given after first successful connection.

## 6. The wizard (two surfaces, one state machine)

### 6.1 VPS side: `focusa pairing wizard`

Interactive bash-equivalent flow implemented as a Rust subcommand. Steps:

1. Probe `FOCUSA_DAEMON_URL/v1/health`.
2. Detect Tailscale hostname (`tailscale status --json`); fall back to env, then daemon URL.
3. Ask operator "Pair your Mac now? [Y/n]".
4. Call `POST /v1/connect/room/create` with the public URL.
5. Render the returned `pair_url` as a terminal QR (Unicode block characters, 50×50).
6. Print the URL underneath for the manual-paste fallback.
7. Poll `/v1/connect/room/{room_id}/status` once per second for up to 60 seconds.
8. On `completed`, print next steps ("open /Applications/Focusa.app").

### 6.2 Mac side: `FirstRunWizard.svelte`

Six-step Svelte component driven by a `WizardState` enum persisted to SQLite (resumes across app quits):

```
welcome → vps_install → vps_discover → show_qr → waiting_phone → connected
```

Each step has its own panel + a `[Continue]` button. State is durable: if the operator quits the app during step 4, reopening brings them back to step 4 with the same room_id and VPS URL.

## 7. Revoke + re-pair

### 7.1 Revoke

`POST /v1/device/pair/revoke` with `{device_id, host}` (or `focusa device pair-revoke <device_id>` on the CLI). The PairingStore appends a revocation entry to the JSONL ledger and removes the in-memory token. Mac, on next API call with the revoked token, receives `401 pairing_revoked` and the FirstRunWizard flips back to `welcome`.

### 7.2 Re-pair

Re-running `focusa pairing wizard` on the VPS creates a new room with a fresh `room_id`. The Mac, on next launch (or after `401 pairing_revoked`), auto-enters the wizard and rejoins. The previous device_id is replaced by the new one in the PairingStore.

### 7.3 Multi-Mac

Each Mac gets its own `device_id` and its own room. PairingPanel lists all paired devices per host. Re-running the wizard adds a new device without disturbing existing ones. Revoking one device does not affect others.

## 8. Token expiry

Tokens expire in 30 days. On expiry, Mac receives `401 token_expired` and FirstRunWizard flips to `welcome`. Operator re-runs the wizard to mint a new token. PairingStore records the expiry; no silent re-pair.

## 9. Architecture invariants (what MUST be true)

1. **VPS owns the room.** No room is ever created client-side. The Mac only joins rooms the VPS has created.
2. **PWA is VPS-served.** No PWA assets live on the phone. The phone is a renderer, not an owner.
3. **Mac never types URLs.** All URL conveyance is via Tailscale MagicDNS, Bonjour, or one-shot CLI handoff. The operator UI contains no "paste VPS URL here" field.
4. **Phone is the sole scanner.** Phone native camera scans terminal QR; PWA browser camera scans Mac QR. Mac has no camera role in pairing.
5. **PairingStore persists everything.** Every room transition is durable across daemon restarts. The room survives a daemon restart mid-pairing.
6. **One token per device per host.** PairingPanel enforces uniqueness. Re-pair replaces the device_id.
7. **Tokens are revocable.** `pair_revoke` is idempotent (revoking an already-revoked device is a no-op).

## 10. Failure modes + recovery

| Failure | Detection | Recovery |
|---|---|---|
| Daemon not running | `v1/health` returns connection refused | `systemctl --user restart focusa-daemon` |
| Tailscale not installed | `tailscale status --json` fails | Install Tailscale, or use Bonjour (LAN), or one-shot CLI handoff |
| Bonjour not working | No `_focusa._tcp.local` response | Check firewall (UDP 5353), or use Tailscale |
| Phone can't open URL | Browser shows connection error | Verify VPS URL is reachable from phone network (Tailscale phone client installed?) |
| Mac not joining room | Wizard stuck at `waiting_for_mac` | `focusa pairing doctor` to diagnose; check Mac firewall / Tailscale status |
| Phone approved but Mac still waiting | Status stuck at `mac_seen` | Check Mac is polling the right room_id; check Mac Keychain not corrupted |
| Token expired | `401 token_expired` | Re-run wizard |
| Token revoked | `401 pairing_revoked` | Re-run wizard |

## 11. What was kept from older versions

From v0.9.33-dev:
- **`focusa device pair-complete <code>`** operator CLI flow — kept as headless pairing path (no GUI, no phone).
- **`focusa_start_bridge_callback`** TCP fast-path — kept as Phase-2 latency optimization (Mac polls in v0.9.35-dev Phase-1; bridge callback becomes a "latency: instant" toggle).
- **PairingPanel.svelte** — post-pair management UI (list, revoke, history). Lives separately from the wizard.
- **PairingDoctor + PairingTransport subcommands** — ops tools, always available.

From v0.9.34-dev:
- **URL-shaped QR** principle — kept (now for terminal QR, not Mac QR).
- **PairingStore (SQLite)** — kept and extended with multi-device indexing.
- **`/connect/firstrun` HTML page** — kept as the entry page used by `firstrun`-style flows.
- **Tailscale auto-detection on VPS** — kept; extended to Mac side.
- **Installer auto-runs transport-setup** — kept.

## 12. What was removed

| Removed (v0.9.35-dev) | Replaced by |
|---|---|
| Mac-creates-room `/v1/connect/room/firstrun` | VPS-creates `/v1/connect/room/create` + Mac-joins `/v1/connect/room/{id}/join` |
| URL-shaped QR on Mac (`pair_url_qr_payload`) | Static mac_offer QR on Mac (no VPS URL embedded) |
| `Settings.svelte` `publicPairingUrl` paste field | Tailscale MagicDNS + Bonjour auto-discovery |
| `PUBLIC_PAIRING_URL_KEY` localStorage | None — Mac forgets any URL after first successful connection |
| `focusa-pairing-wizard.sh` bash script | `focusa pairing wizard` Rust subcommand |
| `FirstRunConnect.svelte` (single QR panel) | `FirstRunWizard.svelte` (6-step wizard component) |

## 13. Implementation status

| Surface | Status (v0.9.35-dev) |
|---|---|
| `POST /v1/connect/room/create` (VPS-creates) | pending |
| `POST /v1/connect/room/{id}/join` (Mac-joins) | pending |
| `GET /v1/connect/room/{id}/status` | done (v0.9.34-dev) |
| `POST /v1/connect/room/{id}/approve` | done (v0.9.34-dev) |
| `POST /v1/connect/room/{id}/mac-offer` | done (v0.9.34-dev, rename pending) |
| `/connect/room/<id>/scan` PWA with getUserMedia | pending |
| `/connect/firstrun` page | done (v0.9.34-dev, deprecated in favor of `/scan`) |
| `focusa pairing wizard` Rust subcommand | pending |
| `focusa pairing create-room` subcommand | pending |
| `focusa pairing history` subcommand | pending |
| `focusa pairing status` subcommand | pending |
| `FirstRunWizard.svelte` (6-step) | pending |
| Mac Tailscale MagicDNS auto-discovery | pending |
| Mac Bonjour/mDNS auto-discovery | pending |
| VPS Bonjour/mDNS service registration | pending |
| Mac static mac_offer QR (idle state) | pending |
| Revoke + re-pair test script | pending |
| Doc 55 (this file) | done |
| Doc 53 §2.0 rewrite | done |
| Doc 54 update (VPS-initiated model) | done |
| Doc 56 (wizard spec) | done |
| Doc 57 (revoke + re-pair spec) | done |

## 14. Operator runbook

**First-time self-host:**
```bash
# On the VPS:
curl install.focusa.dev/focusa | bash
focusa pairing wizard
# → prints terminal QR
# → operator scans with phone
# → phone taps Approve
# → wizard prints "Pairing complete"
```

**Add a second Mac:**
```bash
# On the VPS:
focusa pairing wizard
# → prints a new terminal QR
# → operator points second Mac at the QR via phone camera
# → second Mac joins via Tailscale discovery
# → PairingPanel now lists two devices
```

**Revoke a Mac:**
```bash
# On the VPS:
focusa device pair-list
focusa device pair-revoke <device_id>
# → JSONL ledger appends revocation entry
# → Mac receives 401 on next API call
# → Mac wizard flips to "welcome"
```

**Re-pair after revoke or expiry:**
```bash
# On the VPS:
focusa pairing wizard
# → operator scans the new QR with phone
# → Mac, on relaunch, discovers the new room via Tailscale polling
```

**Diagnose pairing failures:**
```bash
focusa pairing doctor
# → reports: daemon alive, Tailscale reachable, room state, Mac polling status, recent errors
```

## 15. Versioning

This doc describes **v0.9.35-dev**. Predecessors are documented for context but are no longer canonical for new development. See:

- `docs/53-focusa-device-pairing-spec.md` §2.0 — pairing API contract
- `docs/54-focusa-pairing-room-plan.md` — room state machine + storage
- `docs/56-focusa-pairing-wizard-spec.md` — wizard command contract
- `docs/57-focusa-pairing-revoke-and-repair.md` — revoke + re-pair cycle
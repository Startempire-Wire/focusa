# Focusa Self-Host Architecture (v0.9.39-dev)

**Status:** Canonical. v0.9.39-dev replaces v0.9.34-dev for self-host flows.
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

**Step 1 — VPS creates the room.** Operator runs `focusa pairing wizard` on the VPS terminal. The wizard detects Tailscale MagicDNS (or falls back to `FOCUSA_PUBLIC_URL` env, then to `http://127.0.0.1:8787` for dev), calls `POST /v1/connect/room/create`, and receives a `room_id`, `room_claim_secret`, and a `pair_url` containing `#secret=<room_claim_secret>`.

**Step 2 — VPS prints terminal QR.** Wizard renders the secret-bearing `pair_url` as a Unicode-block QR (50×50 cells, ~25 lines tall) directly in the terminal. Operator picks up phone.

**Step 3 — Phone native camera scans terminal QR.** Opens browser at `https://<vps>/connect/room/<room_id>/scan#secret=...`. The Focusa Connect Page PWA loads. PWA reads the secret from `location.hash`, immediately removes the hash from the visible URL via `history.replaceState`, then requests browser-camera permission via `getUserMedia`.

**Step 4 — PWA browser camera scans Mac's static mac_offer QR.** Operator points phone at the Mac menubar. PWA camera reads the `mac_offer` payload (mac_name, mac_nonce, mac_pubkey, optional mac_callback). PWA POSTs `{mac_name, mac_nonce, mac_pubkey, mac_callback, room_claim_secret}` to `/v1/connect/room/<room_id>/mac-offer`.

**Step 5 — PWA shows Approve.** VPS room state flips to `mac_seen`. PWA renders "Tap Approve to pair this Mac." Operator taps Approve. PWA POSTs `{host, operator_id, completed_by}` to `/v1/connect/room/<room_id>/approve`. VPS room flips to `completed`. Token minted.

**Step 6 — Mac receives token.** Mac, which discovered the VPS via Tailscale MagicDNS (or Bonjour fallback) in the background, waits for the phone to bind its static QR to a room, then polls `/v1/connect/room/<room_id>/status`. If mac_callback is reachable, the daemon can also push the completion payload immediately. Mac writes token to Keychain. FirstRunWizard flips to `connected`. Done.

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

There is no permanent "Server URL" field in the Mac app by default. The first-run wizard auto-discovers and forgets any URL it was given after first successful connection via Tailscale, Bonjour, or QR scan.

**V2 P1.1 exception — Advanced paste fallback:** Operators who cannot use Tailscale or Bonjour may manually enter a focusa daemon URL in the Mac app. This URL is persisted to `localStorage` under `PUBLIC_PAIRING_URL_KEY`. This path is **explicitly non-canonical** and is reset after a successful pairing. The canonical V2 architecture expects Tailscale MagicDNS, Bonjour, or a one-shot CLI handoff (`focusa pairing show --mac`) — never a permanently stored URL.

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
- **`focusa_start_bridge_callback`** TCP fast-path — kept as Phase-2 latency optimization (Mac polls in v0.9.39-dev Phase-1; bridge callback becomes a "latency: instant" toggle).
- **PairingPanel.svelte** — post-pair management UI (list, revoke, history). Lives separately from the wizard.
- **PairingDoctor + PairingTransport subcommands** — ops tools, always available.

From v0.9.34-dev:
- **URL-shaped QR** principle — kept (now for terminal QR, not Mac QR).
- **PairingStore (SQLite)** — kept and extended with multi-device indexing.
- **`/connect/firstrun` HTML page** — kept as the entry page used by `firstrun`-style flows.
- **Tailscale auto-detection on VPS** — kept; extended to Mac side.
- **Installer auto-runs transport-setup** — kept.

## 12. What was removed

| Removed (v0.9.39-dev) | Replaced by |
|---|---|
| Mac-creates-room `/v1/connect/room/firstrun` | VPS-creates `/v1/connect/room/create` + phone binds `/v1/connect/room/{id}/mac-offer` with `room_claim_secret` |
| URL-shaped QR on Mac (`pair_url_qr_payload`) | Static mac_offer QR on Mac (no VPS URL embedded) |
| `Settings.svelte` `publicPairingUrl` paste field | Tailscale MagicDNS + Bonjour auto-discovery |
| `PUBLIC_PAIRING_URL_KEY` localStorage | **V2 P1.1**: persisted for operators without Tailscale/Bonjour; explicitly non-canonical; reset after successful pairing |
| `focusa-pairing-wizard.sh` bash script | `focusa pairing wizard` Rust subcommand |
| `FirstRunConnect.svelte` (single QR panel) | `FirstRunWizard.svelte` (6-step wizard component) |

## 13. Implementation status

| Surface | Status (v0.9.39-dev) |
|---|---|
| `POST /v1/connect/room/create` (VPS-creates) | pending |
| `POST /v1/connect/room/{id}/join` (legacy binder) | pending / legacy-compatible only |
| `GET /v1/connect/room/{id}/status` | done (v0.9.34-dev) |
| `POST /v1/connect/room/{id}/approve` | done (v0.9.34-dev) |
| `POST /v1/connect/room/{id}/mac-offer` | done; canonical phone-side binder with `room_claim_secret` |
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

This doc describes **v0.9.39-dev**. Predecessors are documented for context but are no longer canonical for new development. See:

- `docs/53-focusa-device-pairing-spec.md` §2.0 — pairing API contract
- `docs/54-focusa-pairing-room-plan.md` — room state machine + storage
- `docs/56-focusa-pairing-wizard-spec.md` — wizard command contract
- `docs/57-focusa-pairing-revoke-and-repair.md` — revoke + re-pair cycle

---

## 16. Known Gaps (v0.9.39-dev → v0.9.35 GA)

These are the explicit gaps between v0.9.39-dev and a fully smooth, signed, notarized, distribution-ready Intel-Mac test. Each gap has a bead, an owner surface, and a test that proves it's closed. **None of these gaps is a blocker for shipping the daemon + CLI + headless-test stack** — the gaps block a smooth Intel-Mac operator experience only.

| # | Gap | Owner surface | Bead | Test that proves closure |
|---|---|---|---|---|
| **G01** | `tauri.conf.json` version stamp stuck at `0.9.33-dev` | `apps/menubar/src-tauri/tauri.conf.json` | `focusa-7lqf` (P0) | tauri.conf.json reads `0.9.39-dev` |
| **G02** | No v0.9.39-dev Intel-Mac `.app` bundle (latest release is v0.9.33-dev) | release.yml + Tauri build | `focusa-2mzd` (P1) | release asset exists for `v0.9.39-dev-x86_64-apple-darwin` |
| **G03** | `FirstRunConnect.svelte` uses v0.9.34-dev model (URL-shaped QR, Mac-creates-room) instead of v0.9.39-dev (static mac_offer, VPS-creates, Mac-joins) | `apps/menubar/src/lib/components/FirstRunConnect.svelte` | `focusa-73qu` (P1) | menubar_headless_e2e renders VPS-initiated flow |
| **G04** | No `FirstRunWizard.svelte` (6-step wizard component: welcome → vps_install → vps_discover → show_qr → waiting_phone → connected) | `apps/menubar/src/lib/components/` | `focusa-73qu` (P1) | wizard renders all 6 steps with state persistence |
| **G05** | Mac does not display a STATIC `mac_offer` QR (idle state); instead it embeds VPS URL in the QR | `FirstRunConnect.svelte` | `focusa-wb65` (P1) | QR payload contains only `mac_offer`, no `server_url` |
| **G06** | Mac has no Tailscale MagicDNS auto-discovery (relies on operator-pasted URL) | `FirstRunConnect.svelte` + daemon helper | `focusa-59kh` (P1) | Mac auto-finds VPS when on Tailscale network |
| **G07** | Mac has no Bonjour/mDNS auto-discovery (LAN-only fallback) | `FirstRunConnect.svelte` + Tauri command | `focusa-rnva` (P1) | Mac finds VPS via `_focusa._tcp.local` on LAN |
| **G08** | VPS daemon does not advertise `_focusa._tcp.local` Bonjour service | `focusa-daemon` + `mdns-sd` crate | `focusa-9sjk` (P1) | `dns-sd -B _focusa._tcp local.` shows the daemon |
| **G09** | Mac has no handler for `401 token_expired` (30-day TTL) → no re-pair prompt | `FirstRunConnect.svelte` | `focusa-1vmn` (P1) | expiry response triggers FirstRunWizard |
| **G10** | Mac has no handler for `401 pairing_revoked` (admin-side revoke) → no re-pair prompt | `FirstRunConnect.svelte` | `focusa-1vmn` (P1) | revoke response triggers FirstRunWizard |
| **G11** | `focusa pairing status` subcommand missing (dashboard view) | `crates/focusa-cli/src/commands/` | `focusa-2bgr` (P2) | subcommand reports daemon + URL + active rooms + N paired devices |
| **G12** | `focusa pairing history` subcommand missing (audit log) | `crates/focusa-cli/src/commands/` | `focusa-6xjl` (P2) | subcommand lists recent pairings from PairingStore |
| **G13** | `focusa pairing email-link` subcommand missing (phone-camera-broken fallback) | `crates/focusa-cli/src/commands/` | `focusa-4fqp` (P2) | subcommand sends a one-time URL to operator email |
| **G14** | No Intel-Mac-specific operator runbook (build, install, Gatekeeper workaround, Quarantine attribute) | `docs/` | `focusa-3hkj` (P2) | doc covers x86_64 build, right-click → Open, xattr -dr com.apple.quarantine |
| **G15** | No per-platform failure-mode docs (Intel vs ARM, macOS 10.15+, Gatekeeper, codesign) | `docs/` | `focusa-3hkj` (P2) | doc lists failure mode + recovery per platform |
| **G16** | No Apple Developer ID codesign in release pipeline (Gatekeeper blocks unsigned apps) | `.github/workflows/release.yml` + `focusa codesign sign` | `focusa-csg1` (P3, deferrable) | signed `.dmg` uploaded to release |
| **G17** | No notarization step in release pipeline (unsigned DMG shows warning) | `.github/workflows/release.yml` | `focusa-ntr1` (P3, deferrable) | notarized `.dmg` uploaded to release |
| **G18** | Apple Developer ID credentials not stored in repo secrets | GitHub Actions secrets | `focusa-csg1` (P3, deferrable) | secrets configured + workflow uses them |

### 16.1 Why G01–G15 are NOT blocked by G16–G18

**Codesign + notarize (G16, G17, G18) are independent of all other gaps.** An unsigned `.app` builds and runs fine on Intel Mac after the user right-clicks → Open → confirm. The cost is two extra dialogs per first launch, which is acceptable for a `dev` release tagged as `0.9.39-dev`.

The Mac UI half (G03–G10) is testable on Intel Mac **without codesign** by:
1. Building the `.app` via `npm run tauri build` from this branch
2. `xattr -dr com.apple.quarantine Focusa.app` (removes quarantine)
3. Right-click → Open → confirm
4. The `.app` will report `v0.9.39-dev` and use the v0.9.39-dev model

Codesign becomes mandatory only at the GA tag (no `-dev` suffix). Until then, the operator accepts the right-click → Open dance.

### 16.2 Acceptance criteria for closing the gap map

All P0 + P1 gaps (G01–G10) must close before a smooth Intel-Mac test. Specifically:
- `svelte-check` passes
- `cargo clippy --workspace -- -D warnings` passes
- `bash tests/spec_focusa_ui0y_device_pairing_menubar_static_test.sh` passes (all 28+ checks)
- `focusa pairing cycle-test --with-pwa-verify --rounds 10` passes
- `cargo test --package focusa-cli --test menubar_headless_e2e -- --ignored --nocapture` passes
- All three GitHub Actions jobs (Rust, Menubar, Spec Gates) pass

When all those gates pass, the operator can build `Focusa.app` for Intel Mac via `npm run tauri build`, `xattr -dr com.apple.quarantine`, and have a fully working self-host pairing flow.
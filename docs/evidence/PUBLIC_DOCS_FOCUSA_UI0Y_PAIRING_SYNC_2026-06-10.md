# Public Docs focusa-ui0y Pairing Sync — 2026-06-10

## Scope

Public-facing Focusa docs were synced after the `focusa-ui0y` device
pairing feature landed (sub-beads .6–.13). The feature adds three handoff
modes for pairing a Mac to a Focusa VPS:

- **Mode A** — CLI (operator runs `focusa device pair-complete <code>` over SSH)
- **Mode B** — QR + phone (Telegram/Discord-style; Mac shows QR, phone opens PWA helper)
- **Mode C** — QR + VPS browser (kiosk on same LAN as daemon)

## Architecture

- **Spec:** [`docs/53-focusa-device-pairing-spec.md`](../53-focusa-device-pairing-spec.md) — owns
  the full pairing architecture (5 sections + threat model + portability invariants).
- **Portability:** each Focusa install is its own trust root. Multi-tenant safe
  via the `FOCUSA_PAIRING_URL` env var (when unset, falls back to `daemon_base_url`).

## New code surfaces

| Surface | Endpoint / Command | File |
|---|---|---|
| API | `POST /v1/device/pair/start` (extended) | `crates/focusa-api/src/routes/device_pairing.rs` |
| API | `GET /pair/{device_id}` (PWA helper HTML) | `crates/focusa-api/src/routes/device_pairing.rs` |
| API | `GET /pair/{device_id}/manifest.json` (PWA manifest) | same file |
| API | `GET /pair/{device_id}/sw.js` (service worker) | same file |
| CLI | `focusa device pair-qr` (shortcut for QR handoff) | `crates/focusa-cli/src/commands/device_pairing.rs` |
| Pi | `focusa_device_pair_qr` tool | `apps/pi-extension/src/tools.ts` |
| Menubar | `QRCode.svelte` (inline SVG renderer) | `apps/menubar/src/lib/components/QRCode.svelte` |
| Menubar | `PairingPanel.svelte` Mode A/B/C tabs | `apps/menubar/src/lib/components/PairingPanel.svelte` |

## New env var

- `FOCUSA_PAIRING_URL` — public URL the operator's phone will hit
  (e.g. `https://focusa-conn.verious.net`). When unset, the daemon
  uses `daemon_base_url` (default `http://127.0.0.1:8787`).

## Updated public docs

- `README.md` — added focusa-ui0y to recent additions, maturity table, tool table
- `docs/53-focusa-device-pairing-spec.md` — new consolidated spec
- `docs/11-menubar-ui-spec.md` — DevicePairing surface + Mode A/B/C requirements
- `docs/43-multi-device-sync.md` — Device Pairing Handoff section
- `docs/current/API_REFERENCE_CURRENT.md` — device/pair route inventory + PWA endpoints
- `docs/current/CLI_REFERENCE_CURRENT.md` — device subcommand section
- `docs/current/PORTABILITY_AUDIT.md` — pairing portability entry
- `docs/current/focusa-tool-contracts.json` — new tool entry + spec_path
- `docs/current/focusa-tool-choreography.json` — 3 new edges, tool_count 78→79
- 5 tool-level docs updated to reference the new spec
- 1 new tool doc (`focusa_device_pair_qr.md`)

## Verification

- **23/23** static checks in `tests/spec_focusa_ui0y_device_pairing_static_test.sh` pass
- **ALL** checks in `tests/spec_focusa_ui0y_device_pairing_menubar_static_test.sh` pass
- `scripts/validate-focusa-tool-contracts.mjs` — passed (79/79 tools, 231/231 edges)
- `scripts/audit-focusa-tool-implementation-spec-gaps.mjs` — passed
- Live daemon: `pair_start` returns `pair_url` + `pair_url_qr_payload` + 3 `next_tools`
- Live PWA: HTML (4581 bytes), manifest, sw.js — all 200
- Live CLI: `focusa device pair-qr` prints `pair_url` prominently

## Beads

- `focusa-ui0y.1`–`.5` — done previously (base 5 surfaces)
- `focusa-ui0y.6` — base audits ✅ (extended in this slice)
- `focusa-ui0y.7` — `pair_url` field + `FOCUSA_PAIRING_URL` env ✅
- `focusa-ui0y.8` — PWA helper page at `GET /pair/{device_id}` ✅
- `focusa-ui0y.9` — menubar QR renderer component ✅
- `focusa-ui0y.10` — menubar PairingPanel redesign (Mode A/B/C tabs) ✅
- `focusa-ui0y.11` — CLI `focusa device pair-qr` shortcut ✅
- `focusa-ui0y.12` — Pi `focusa_device_pair_qr` tool ✅
- `focusa-ui0y.13` — extended audits (covers pair_url + PWA + multi-tenant) ✅

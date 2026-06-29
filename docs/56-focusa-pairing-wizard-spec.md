# Focusa Pairing Wizard Spec (v0.9.39-dev)

**Status:** Canonical for `focusa pairing wizard` Rust subcommand.
**Architecture overview:** `docs/55-focusa-self-host-architecture.md` §6.

---

## 1. Purpose

The wizard is the canonical first-run pairing flow for self-hosted Focusa. It runs on the VPS terminal after install. It detects the operator's transport (Tailscale MagicDNS, env override, daemon URL), creates a pairing room, renders a scannable terminal QR, polls for phone approval, and reports success. The wizard is a Rust subcommand, not a bash script — no python/qrencode/tailscale-CLI dependencies at runtime beyond what the daemon already requires.

## 2. Invocation

```
focusa pairing wizard
focusa pairing wizard --no-tailnet       # skip Tailscale detection, use FOCUSA_PUBLIC_URL or daemon URL
focusa pairing wizard --timeout 60       # poll for up to 60 seconds (default: 300)
focusa pairing wizard --demo             # self-test: auto-approve via local daemon
FOCUSA_DAEMON_URL=http://... focusa pairing wizard
```

`focusa pairing create-room` is an alias that exits after creating the room + printing the QR + a status-poll hint (does not block on phone approval).

## 3. Behavior

```
$ focusa pairing wizard

  ╔══════════════════════════════════════════════════════════╗
  ║          Focusa Pairing Wizard                           ║
  ║          focusa-pairing-wizard v0.9.39-dev               ║
  ╚══════════════════════════════════════════════════════════╝

▶  Welcome to Focusa pairing.

✓  Focusa daemon detected (v0.9.39-dev) at http://127.0.0.1:8787

▶  Resolving phone-reachable URL…
✓  Tailscale MagicDNS resolves: focusa-vps.tail-net.ts.net → 100.94.238.56
   Pairing URL: https://focusa-vps.tail-net.ts.net

▶  Pair your Mac now? [Y/n]: Y

▶  Creating pairing room…
✓  Room 019f07cd…  expires in 5 min

  Scan this QR with your iPhone or Android camera:

  ██████████████    ████          ██████        ████      ████████
  ██          ██          ██  ██  ██    ██  ██  ██        ██
  ██  ██████  ██  ██  ██    ████  ████████        ██████  ██
  ██  ██████  ██  ██  ██████  ██  ██    ██  ██████  ██  ██  ██
  ██  ██████  ██  ██  ██    ████  ████████  ██    ████  ██████
  ██          ██        ██  ██  ██    ██  ██        ██  ██
  ██████████████  ██  ██  ██  ██  ██  ██  ██  ██████  ██  ██
                       ████████  ████      ████████████  ██  ██
  ████  ██      ██  ████████  ████████████  ██          ██
  …(50 rows × 50 cols)

  URL: https://focusa-vps.tail-net.ts.net/connect/room/019f07cd…/scan

▶  Waiting for Mac to join the room…
  [01s] status=waiting_for_mac
  [02s] status=waiting_for_mac
  …
  [08s] status=mac_seen        (phone scanned terminal QR + PWA opened)
  [12s] status=mac_seen        (waiting for phone to tap Approve)
  [14s] status=completed       (phone tapped Approve; token issued)

✓  Pairing complete.

  Next:
    1. On your Mac: open /Applications/Focusa.app
       (the wizard will detect this VPS and connect automatically)
    2. Verify:      focusa doctor
```

## 4. Tailscale detection

If `tailscale status --json` exits 0 and contains `Self.DNSName`, the wizard extracts the MagicDNS hostname and uses `https://<hostname>` as the public URL. Otherwise, the wizard falls back to:

1. `FOCUSA_PUBLIC_URL` env var
2. `FOCUSA_PAIRING_URL` env var
3. `FOCUSA_DAEMON_URL` env var (the daemon's bind URL — usually `http://127.0.0.1:8787`)
4. The compiled-in default

The wizard prints the detected URL prominently so the operator can verify it before the QR is rendered.

## 5. QR rendering

The wizard uses the `qrcode` crate (already a dependency of `focusa-cli`) to render the `pair_url` as a Unicode-block QR. Block characters (`██` and `  `) are used for high density (50×50 cells in ~25 terminal lines). Background and foreground are inverted from terminal theme defaults to ensure scannability in both light and dark terminals.

The wizard also prints the URL underneath the QR for the manual-paste fallback.

## 6. Polling

After the QR is printed, the wizard polls `/v1/connect/room/{room_id}/status` once per second for up to `--timeout` seconds (default 300). Each poll result is printed as a single-line overwrite to keep the terminal tidy. The wizard exits with code 0 on `completed`, code 1 on timeout, code 2 on daemon unreachable.

## 7. Error handling

- **Daemon unreachable**: exit 2 with diagnostic; suggest `focusa pairing doctor`.
- **Tailscale not installed**: warn but continue; fall back to env or daemon URL.
- **Room creation fails**: exit 3 with the daemon error message.
- **Timeout**: exit 1 with recovery hint ("re-run `focusa pairing wizard` to create a new room").
- **Operator declines** (answers "n" to "Pair your Mac now?"): exit 0 cleanly.

## 8. Exit codes

| Code | Meaning |
|---|---|
| 0 | Pairing completed or operator declined cleanly |
| 1 | Timeout (no phone approval within timeout window) |
| 2 | Daemon unreachable |
| 3 | Room creation failed (daemon returned error) |
| 4 | Tailscale detection failed AND no env override set |

## 9. Companion commands

- `focusa pairing status` — show currently-active rooms + recently-completed pairings
- `focusa pairing history` — show pairing history (audit log from PairingStore)
- `focusa pairing transport-setup` — discover + persist the phone-reachable URL
- `focusa pairing doctor` — diagnose pairing failures (daemon, Tailscale, Bonjour, Mac polling)
- `focusa pairing create-room` — non-interactive variant (returns room_id + pair_url as JSON, no QR)

## 10. Implementation notes

The wizard is implemented in `crates/focusa-cli/src/commands/pairing_wizard.rs` and registered in `commands/mod.rs`. It replaces the bash script `crates/focusa-cli/scripts/focusa-pairing-wizard.sh` (which is retained as a fallback for systems without the Rust CLI installed but is no longer the canonical implementation).

The wizard does NOT depend on:
- bash (pure Rust)
- python3 (QR rendering in Rust via `qrcode` crate)
- `qrencode` or `qr2term` system binaries
- `tailscale` CLI (falls back gracefully if missing)

The wizard DOES depend on:
- The Focusa daemon running at `$FOCUSA_DAEMON_URL`
- A terminal that supports Unicode block characters (any modern terminal)

## 11. Versioning

This spec ships with v0.9.39-dev. Predecessors:

- v0.9.34-dev: `crates/focusa-cli/scripts/focusa-pairing-wizard.sh` (bash + python qrcode)
- v0.9.33-dev: `scripts/phone-bridge-transport.sh` (bash, no QR)

The Rust wizard supersedes both. The bash scripts remain in-tree for backward compatibility but are not canonical.
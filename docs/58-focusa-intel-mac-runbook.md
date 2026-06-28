# Focusa Intel-Mac Operator Runbook (v0.9.35-dev → v0.9.35 GA)

**Status:** Canonical for self-host pairing on Intel Mac (also applies to Apple Silicon).
**Owner surface:** `apps/menubar/` (Tauri), `crates/focusa-cli/` (wizard), `crates/focusa-api/` (daemon).
**Bead:** `focusa-3hkj` (G14) + `focusa-3hkj-2` (G15).

This runbook covers everything an operator needs to test Focusa on a real Mac end-to-end. Read top-to-bottom on the first run; use as a checklist on subsequent runs.

---

## 1. Prerequisites

| Requirement | Min | Recommended | Verify |
|---|---|---|---|
| macOS version | 10.15 Catalina | 13 Ventura or later | `sw_vers` |
| Architecture | Intel x86_64 OR Apple Silicon arm64 | either | `uname -m` |
| Rust toolchain | stable | nightly (matches CI) | `cargo --version` |
| Node.js | 22 LTS | 22 LTS | `node --version` |
| Xcode CLI tools | latest | latest | `xcode-select -p` |
| iPhone (for phone-as-bridge) | iOS 14+ | iOS 17+ | — |
| Android (for phone-as-bridge) | Android 8+ | Android 14+ | — |
| Camera (for phone scan) | any | any | — |
| Tailscale (recommended) | — | install on both VPS + Mac | `tailscale status` |

## 2. VPS-side one-time setup

```bash
# 2.1 Install Focusa (creates daemon, sets up systemd unit, opens port 8787)
curl install.focusa.dev/focusa | bash

# 2.2 Verify daemon is healthy
focusa pairing status
# Expected: daemon: 0.9.35-dev (http://127.0.0.1:8787)

# 2.3 If you have Tailscale on this VPS, the wizard will auto-discover:
tailscale status | grep -i hostname
# Output: tail9229d6.ts.net (or similar MagicDNS name)
```

## 3. VPS-side: start a pairing room

```bash
# 3.1 Run the wizard
focusa pairing wizard
# It will:
#   - Detect Tailscale MagicDNS hostname (e.g. focusa-vps.tail-net.ts.net)
#   - Call POST /v1/connect/room/create
#   - Print a 50x50-cell Unicode QR + the pair_url
#   - Poll /v1/connect/room/{id}/status every 1.5s for 5 minutes

# 3.2 If you don't have Tailscale:
FOCUSA_PUBLIC_URL=https://focusa-vps.example.com focusa pairing wizard
# Or: focusa pairing create-room  # non-interactive, returns JSON
```

The wizard output looks like:
```
  ╔══════════════════════════════════════════════════════════╗
  ║          Focusa Pairing Wizard                           ║
  ║          focusa-pairing-wizard v0.9.35-dev               ║
  ╚══════════════════════════════════════════════════════════╝
  ...
  Pairing URL: https://focusa-vps.tail-net.ts.net/connect/room/019f.../scan
  ...
```

## 4. Mac-side: install + first launch

The menubar `.app` is published with every tag. Download the Intel build (the operator has an Intel Mac):

```bash
# 4.1 Download (replace URL with the latest v* tag from the GitHub Releases page)
# Latest:  https://github.com/Startempire-Wire/focusa/releases/download/v0.9.35-dev/Focusa_x64.app.tar.gz
# DMG alt: https://github.com/Startempire-Wire/focusa/releases/download/v0.9.35-dev/Focusa_0.9.35-dev_x64.dmg
curl -L -o /tmp/Focusa_x64.app.tar.gz \
  https://github.com/Startempire-Wire/focusa/releases/download/v0.9.35-dev/Focusa_x64.app.tar.gz

# 4.2 Extract (it contains a top-level Focusa.app/)
mkdir -p /tmp/focusa-app
tar xzf /tmp/Focusa_x64.app.tar.gz -C /tmp/focusa-app

# 4.3 Remove the Gatekeeper quarantine attribute (release is ad-hoc-signed, not notarized)
xattr -dr com.apple.quarantine /tmp/focusa-app/Focusa.app

# 4.4 Move to /Applications (the canonical location so Launch Services registers it)
sudo mv /tmp/focusa-app/Focusa.app /Applications/

# 4.5 First launch
open /Applications/Focusa.app
# macOS Gatekeeper shows: "Focusa.app is from an unidentified developer. Are you sure you want to open it?" → click "Open"
# (One-time per machine; after first launch, double-click works normally.)
```

> **Alternative install path** — instead of moving to `/Applications`, you can also drag the .app from Finder directly. Either path triggers Gatekeeper the same way.

The menubar icon appears in the macOS menu bar (top right). Click it.

**Troubleshooting first launch:**

| Symptom | Cause | Fix |
|---|---|---|
| "Focusa.app cannot be opened because it is from an unidentified developer" | Gatekeeper blocking unsigned/ad-hoc binary | Right-click the .app → Open → Open (the second Open button bypasses Gatekeeper for that one launch) |
| Double-click does nothing | Quarantine attribute | `xattr -dr com.apple.quarantine /Applications/Focusa.app` |
| Icon appears in Dock but not menubar | App is in foreground window state | `osascript -e 'tell application "Focusa" to activate'` then check menubar top-right |
| Menubar icon missing entirely | LSUIElement not set; check `info.plist` `LSUIElement: true` | The current build sets `visible: false` and `decorations: false` to force menubar-only. If still missing, run `pkill focusa-menubar && open /Applications/Focusa.app` |
| Crashes immediately on launch | Tauri runtime missing | `xcrun --find otool && otool -L /Applications/Focusa.app/Contents/MacOS/focusa-menubar | head`; reinstall macOS CLT: `xcode-select --install` |

## 5. Mac-side: the 6-step wizard

| Step | Panel | Action | What happens |
|---|---|---|---|
| 1 | **Welcome** | Click **Get started** | Advance to step 2 |
| 2 | **Install on your VPS** | Click **Continue** if you've already installed Focusa (see §2) | Advance to step 3 |
| 3 | **Discover your VPS** | Click **Discover** | Mac probes Tailscale MagicDNS, Bonjour, saved pairing URL. On success: `Found: https://focusa-vps.tail-net.ts.net (Tailscale MagicDNS)`. If all 3 fail, click **Advanced** → paste URL → click **Use this URL** |
| 4 | **Scan with your phone** | Open iPhone/Android camera → point at Mac's QR | QR opens `https://focusa-vps.tail-net.ts.net/connect/room/.../scan` in phone browser. Mac auto-advances to step 5 |
| 5 | **Waiting for phone approval** | (wait) | Phone PWA shows "Pair this Mac" + Approve button. Tap. Mac auto-advances to step 6 |
| 6 | **Paired** | Token stored in macOS Keychain. Done. | The Focusa menubar tray icon stays in the macOS menu bar |

## 6. Phone-side: what the operator sees

After scanning the Mac's QR with the phone camera:

1. Phone browser opens (Safari on iOS, Chrome on Android)
2. Browser shows the Focusa Connect Page
3. Page asks camera permission ("Focusa wants to access camera")
   - Tap **Allow** — needed to scan the Mac's static mac_offer QR (NOT needed if you skipped step 4 above)
4. Page shows "Pair this Mac" with a button
5. Tap **Approve**
6. Page shows "Paired" in green
7. Browser can be closed

## 7. Expected UX (happy path)

Total steps: 4 user actions
1. Operator runs `focusa pairing wizard` on VPS
2. Operator opens Focusa on Mac → clicks through wizard
3. Operator scans Mac's QR with phone
4. Phone taps Approve

Total time: ~30 seconds

## 8. Failure mode cheat sheet (Intel Mac specific)

| Symptom | Diagnosis | Recovery |
|---|---|---|
| `npm run tauri build` fails with `error: linker 'cc' not found` | Xcode CLI tools not installed | `xcode-select --install` |
| Gatekeeper blocks app on first launch | Unsigned `.app` | Right-click → Open → confirm |
| `xattr -dr com.apple.quarantine` says `No such file` | `.app` not in /Applications | `mv` it first |
| Menubar icon doesn't appear | `set_activation_policy(Accessory)` failed | Check `apps/menubar/src-tauri/src/main.rs` for the policy call |
| `npm run tauri build` succeeds but `Focusa_x64.app.tar.gz` is missing | Build skipped macOS target | Verify `tauri.conf.json` has `"targets": "all"` or include `x86_64-apple-darwin` |
| `error: linking with `cc` failed: architecture x86_64` on Apple Silicon Mac | Default target is aarch64; need `--target x86_64-apple-darwin` | `npm run tauri build -- --target x86_64-apple-darwin` |
| Mac app opens but wizard stuck on "Discover your VPS" | All 3 discovery paths failed | Open Advanced → paste URL → verify `curl https://your-url/v1/health` works |
| Phone opens PWA but camera permission denied | iOS Safari requires explicit Allow | Settings → Safari → Camera → Allow |
| Phone scans but shows raw JSON | Phone browser is set to never open URLs in apps | Default; iOS/Android typically show "Open in Safari/Chrome" |
| Mac app stuck on "Waiting for phone approval" | Phone didn't tap Approve OR VPS unreachable from phone | Open Mac's pairing URL in a desktop browser; verify it loads; if it doesn't, Tailscale not installed on phone |
| Token expired after 30 days | Server returns `401 token_expired` | FirstRunWizard detects this and re-enters vps_discover step. Operator re-runs `focusa pairing wizard` on VPS |
| Operator revoked this Mac | Server returns `401 pairing_revoked` | Same as above |

## 9. Apple Silicon vs Intel Mac differences

| Surface | Apple Silicon (arm64) | Intel (x86_64) |
|---|---|---|
| Build target | `aarch64-apple-darwin` | `x86_64-apple-darwin` |
| Universal binary | `npm run tauri build -- --target universal-apple-darwin` | same flag |
| Codesign | Same `focusa codesign sign --identity "Developer ID Application: <team>"` | same |
| Notarization | Same `xcrun notarytool submit ... --wait` | same |
| Gatekeeper | Same `xattr -dr com.apple.quarantine` | same |
| Tauri deps | Same `cargo tauri build` | same |

The Intel-Mac and Apple-Silicon `.app` bundles are functionally identical; only the CPU architecture differs. The `--target universal-apple-darwin` flag produces a single `.app` that runs on both.

## 10. Verifying success

```bash
# 10.1 On the VPS:
focusa pairing status
# Expected: paired: 1 active devices (or more, if you've paired before)

focusa pairing history --limit 1
# Expected: 1 entry, revoked=no, token preview=abc123…

# 10.2 On the Mac:
# - menubar tray icon present
# - click it → PairingPanel (not FirstRunWizard) shows your device as "Paired"
# - all API calls succeed (no 401 in dev console)

# 10.3 From the phone:
# - re-scan the QR → "Pair this Mac" shows "Mac: <name>" with the Mac's name from step 4
# - tapping Approve on a 2nd Mac joins a 2nd device
```

## 11. Next steps after first pairing

- **Open Settings:** Click the menubar icon → Settings tab → adjust discovery host, paste alternative VPS URL
- **Re-pair after expiry:** Re-run `focusa pairing wizard` on VPS, the Mac wizard detects `401 token_expired` and re-enters the discovery step automatically
- **Pair a second Mac:** Run `focusa pairing wizard` on VPS, open Focusa on the 2nd Mac, scan the new QR with the phone, tap Approve
- **Revoke a Mac:** `focusa device pair-revoke <device_id>` on VPS — the Mac receives `401 pairing_revoked` on next API call and re-enters the wizard
- **Diagnose failures:** `focusa pairing doctor` on VPS — single-command root-cause report
- **Run the cycle test:** `focusa pairing cycle-test --rounds 10 --with-pwa-verify` — validates the full daemon + PWA flow in 75ms

## 12. CI integration

The same flow is exercised in CI by:
- `tests/spec_focusa_ui0y_device_pairing_menubar_static_test.sh` (28+ static checks)
- `crates/focusa-cli/tests/menubar_headless_e2e.rs` (`cargo test -- --ignored`)
- `crates/focusa-cli/tests/revoke_repair_cycle.rs` (`cargo test -- --ignored`)
- `focusa pairing cycle-test --with-pwa-verify --rounds 10`

A green CI on the main branch is a strong signal that the Intel-Mac operator runbook will succeed.

## 13. When codesign + notarize is required

The unsigned `.app` works on Intel Mac after `xattr -dr com.apple.quarantine` + right-click → Open. For **GA releases** (no `-dev` suffix), Apple requires:

1. Apple Developer ID (`Developer ID Application: <team>`)
2. `codesign --deep --force --options runtime --sign "Developer ID Application: <team>" Focusa.app`
3. `xcrun notarytool submit Focusa.app.zip --key ... --key-id ... --issuer ... --team-id ... --wait`
4. `xcrun stapler staple Focusa.app`

The release pipeline wires all 4 steps in `.github/workflows/release.yml` (G16-G18) but requires these GitHub Actions secrets:
- `APPLE_SIGNING_IDENTITY` (e.g. `Developer ID Application: Acme Corp (TEAMID)`)
- `APPLE_TEAM_ID`
- `APPLE_IDENTITY_NAME` (human-readable)
- `APPLE_APP_SPECIFIC_PASSWORD` (App Store Connect login)
- `APPLE_API_KEY_ID` + `APPLE_API_ISSUER_ID` + `APPLE_API_KEY_P8` (App Store Connect API key)

Until those secrets are configured, the pipeline still uploads an unsigned `.app` + `.dmg` for `-dev` releases; the unsigned install dance (§4.3) is the only friction.

## 14. See also

- `docs/55-focusa-self-host-architecture.md` — full architecture overview
- `docs/53-focusa-device-pairing-spec.md` §2.0 — canonical pairing flow
- `docs/56-focusa-pairing-wizard-spec.md` — wizard CLI contract
- `docs/57-focusa-pairing-revoke-and-repair.md` — revoke + re-pair semantics
- `apps/menubar/src/lib/components/FirstRunWizard.svelte` — Mac wizard source code (the v0.9.35-dev state machine)
- `apps/menubar/src-tauri/src/main.rs` — Tauri commands + Bonjour discovery
- `crates/focusa-cli/src/commands/pairing_wizard.rs` — wizard Rust source
- `crates/focusa-core/src/bonjour.rs` — Bonjour module (currently a stub; full mdns-sd integration queued for v0.9.36)
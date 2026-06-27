# Focusa Portability Audit — External Tester Readiness

Date: 2026-05-25
Scope: Rust workspace, CLI, daemon/API, TUI, Tauri menubar, Pi extension, scripts, CI/release packaging, README onboarding.

## Target tester environments

| Surface | Primary modern environments | Current posture |
|---|---|---|
| Rust CLI / daemon / TUI | macOS arm64/x64, Linux x64 glibc | Buildable from source; release workflow now includes macOS + Linux x64 Rust binaries. |
| Menubar app | macOS arm64/x64 | Tauri package proof remains macOS-focused. Linux/Windows desktop packaging is not yet claimed. |
| Pi extension | Pi environments with Node 20+ and a POSIX shell | Typechecks; daemon restart default no longer assumes systemd first. |
| API scripts / spec gates | Linux CI, local POSIX shell | CI-proven on Ubuntu; helper scripts assume bash, curl, jq, Python 3, and cargo. |
| Docs / onboarding | macOS/Linux source testers | README no longer references host-specific `/home/wirebot` or cPanel Node paths for quickstart validation. |

## Improvements made during audit

1. **Pi extension restart portability**
   - Before: default `daemonRestartCommand` assumed `systemctl`, failing on macOS, containers, WSL, and non-systemd Linux.
   - Now: default first tries `focusa-daemon` from `PATH` via background start, then falls back to `systemctl`.
   - Files: `apps/pi-extension/src/config.ts`, `apps/pi-extension/src/state.ts`.

2. **Explicit toolchain floors**
   - Rust workspace and member crates now declare/inherit `rust-version = "1.91"` for edition 2024 compatibility.
   - JS packages now declare Node `>=20`.
   - CI menubar job now runs on Node 22 for a current LTS-like tester path.
   - Files: `Cargo.toml`, `apps/menubar/package.json`, `apps/pi-extension/package.json`, `.github/workflows/ci.yml`.

3. **README host-path cleanup**
   - Removed hardcoded deployed daemon path under `/home/wirebot`.
   - Replaced cPanel-specific Pi loader import path with npm-global resolution.
   - File: `README.md`.

4. **Release artifact coverage**
   - Release workflow now publishes Linux x64 glibc Rust binaries in addition to macOS Rust binaries.
   - Release menubar packaging pins Bun `1.3.6` instead of floating `latest`.
   - File: `.github/workflows/release.yml`.

## Remaining portability caveats

1. **Menubar desktop packaging beyond macOS**
   - Current CI proves macOS `.app` packaging only.
   - Linux/Windows Tauri packaging requires OS-specific dependencies and should remain unclaimed until CI jobs exist.

2. **POSIX shell assumptions**
   - Scripts and Pi daemon kickstart use bash/POSIX utilities.
   - Windows testers should use WSL or build/run Rust binaries directly without relying on shell scripts.

3. **External helper tools**
   - Full release/spec proof paths use `curl`, `jq`, `python3`, `ripgrep`, `gh`, and `guardian` depending on command.
   - Core Rust build does not require all helpers; docs should distinguish source build from full maintainer proof.

4. **Service installation**
   - Focusa is local-first and can run foreground from source/binary.
   - OS-native service installers are not yet provided for launchd/systemd/Windows Service.

## Verification run

- `npm run check` and `npm run build` in `apps/menubar`.
- `npx tsc --noEmit --skipLibCheck` in `apps/pi-extension`.
- `cargo check --workspace` with a clean external target directory.
- Latest pushed CI before this audit: `ff37db7` succeeded; follow-up release-proof artifact commit `87b1c2c` was running when audit started.

## Recommended next portability work

1. Add a CI matrix job for Rust build/test on macOS and Linux stable toolchains, while keeping nightly clippy if required.
2. Add a lightweight `doctor portability` or documented checklist that checks Node/Rust/cargo/curl/jq/gh availability.
3. Add OS-specific service snippets: launchd plist, systemd user unit, and Windows/WSL notes.
4. Add Linux Tauri packaging only after required system libraries are documented and CI-proven.

## Device pairing portability (focusa-ui0y)

- `FOCUSA_PAIRING_URL` env var (added 2026-06-10) lets any operator expose the
  PWA helper page (`GET /pair/{device_id}`) on their own public hostname
  (e.g. `https://focusa-conn.verious.net`). When unset, the daemon falls
  back to `daemon_base_url` (default `http://127.0.0.1:8787`), so the QR
  flow degrades gracefully to local-network or CLI mode.
- The `pair_url` field in `pair_start` is built from this env var, with
  full URL-encoding of the `device_id` (UUIDv7, URL-safe by default).
- The PWA helper page is 200 LOC, no third-party scripts, no external
  assets — safe to serve from any Focusa install.
- Multi-tenant isolation: each daemon is its own trust root; codes and
  tokens generated on operator-A's daemon cannot be completed on
  operator-B's daemon. The `devices.jsonl` ledger is the boundary.

## Addendum A — Zero-Setup Operator Contract (2026-06-27)

Single contract: a fresh operator reaches a working Mac pairing flow with **one curl + one `focusa pair`**, without reverse-proxy, LaunchAgent, or notarytool hand-editing.

### A.1 Five-minute setup

```bash
curl -fsSL https://install.focusa.dev/focusa | bash
focusa pairing transport setup      # auto-bundles cloudflared quick tunnel if no phone-reachable URL exists
focusa pair                          # prints QR + connect_url
open /Applications/Focusa.app        # Mac first-run + scan from phone
```

If anything fails: `focusa pairing doctor` returns a single root-cause report.

### A.2 Architecture commitments and enforcement

| Invariant | Enforced by |
|---|---|
| Protocol never hardcodes environment-specific domains | `tests/spec_pairing_portable_architecture_static_test.sh` |
| Phone-reachable transport is verified before any QR is shown | `crates/focusa-cli/src/commands/pair.rs` transport resolver |
| Stale daemon blocks pairing | `focusa pair` fail-closed version guard |
| Only `connect/approve` mints a token | `crates/focusa-api/src/routes/device_pairing.rs` |
| Daemon restarts do not drop in-flight pairing sessions | PairingState persisted in `focusa-core` SQLite |
| macOS app does not show "developer cannot be verified" | `focusa codesign sign + notarize` runs on Mac installer |
| Daemon auto-starts on host | `focusa install-service` auto-enables |
| Phone camera raw JSON outcome is explained | `apps/menubar/.../FirstRunConnect.svelte` copy |
| Mac receives bridge callback even across NAT | `focusa://` deep-link fallback in Tauri `Info.plist` |

### A.3 Transport options (auto-detected order)

1. `FOCUSA_PAIRING_URL` env var
2. `/etc/focusa/public-url` file
3. `FOCUSA_API_URL` / `FOCUSA_BASE_URL` (non-local only)
4. Hostname `https://<host>`, `http://<host>`, `http://<host>:8787`
5. Public IPv4 variants
6. Private/Tailscale IPv4
7. `http://127.0.0.1:8787` (same-machine only)

If nothing is reachable and the host is a VPS, the resolver tries a **neutral multi-transport bundle** in order: (1) `cloudflared` quick tunnel (`*.trycloudflare.com`) if installed, (2) Tailscale Funnel if `tailscaled` is up, (3) `bore.pub` (Rust static), (4) `localhost.run` SSH, (5) `ssh -R` reverse tunnel if a jump host is configured, (6) operator-supplied `/etc/focusa/public-url`. The chosen URL is verified by the resolver before being written to `/etc/focusa/public-url`. Single-vendor lock-in is rejected by `tests/spec_pairing_portable_architecture_static_test.sh`.

### A.4 Service install defaults

`focusa install-service` (Rust core, `crates/focusa-cli/src/commands/service.rs`):
- Detects manager: `systemd --user` on Linux, `launchd` on macOS.
- Writes unit/LaunchAgent.
- Runs `systemctl --user enable --now focusa-daemon.service` (Linux; requires `loginctl enable-linger $USER`).
- Runs `launchctl load -w <plist>` (macOS).
- Returns manager/unit_path/enabled/loaded JSON report.

Shell installer (`scripts/install-focusa.sh`) delegates to this command by default; `--no-service` skips it.

### A.5 macOS app signing

`focusa codesign sign --developer-id "Developer ID Application: <Team>" --team-id <id> --apple-id <email> --app-specific-password <pwd>` runs:

1. `codesign --deep --force --options runtime --sign "$DEVELOPER_ID" Focusa.app`
2. `ditto -c -k Focusa.app Focusa.zip`
3. `xcrun notarytool submit Focusa.zip --apple-id ... --team-id ... --password ... --wait`
4. `xcrun stapler staple Focusa.app`
5. `spctl --assess --type execute --verbose=2 Focusa.app`

AlmaLinux CI cannot run this; gate at release time on a real Mac.

### A.6 Deep-link fallback (`focusa://`)

Tauri `Info.plist` registers:

```xml
<key>CFBundleURLTypes</key>
<array><dict>
  <key>CFBundleURLName</key><string>com.startempire.focusa.menubar</string>
  <key>CFBundleURLSchemes</key><array><string>focusa</string></array>
</dict></array>
```

When the phone browser cannot reach the Mac bridge callback, Focusa Connect Page falls back to `focusa://connect?payload=<base64-mac_completion_payload>` that opens the menubar app and completes pairing.

### A.7 Persistent pairing state

`PairingState` and `ConnectSession` move from in-memory `OnceLock` to `focusa-core` SQLite tables:
- `pairing_codes(code TEXT PK, ...)`
- `connect_sessions(connect_id TEXT PK, ...)`
- `device_tokens(token TEXT PK, ...)`

Daemon restarts now preserve codes and connect sessions for their full TTL.

### A.8 First-time operator TL;DR

1. `curl -fsSL https://install.focusa.dev/focusa | bash`
2. `focusa pairing transport setup`
3. `focusa pair`
4. Open `Focusa.app`, scan from Focusa Connect Page
5. Approve on phone → token in Keychain → paired

Anything breaking this flow is a P0 bead.

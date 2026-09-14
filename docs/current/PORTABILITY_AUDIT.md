# Focusa Portability Audit — External Tester Readiness

Date: 2026-05-25
Scope: Rust workspace, CLI, daemon/API, TUI, Tauri menubar, Pi extension, scripts, CI/release packaging, README onboarding.

## Target tester environments

| Surface | Primary modern environments | Current posture |
|---|---|---|
| Rust CLI / daemon / TUI / session runner | macOS arm64/x64, Linux x64 glibc/musl plus Linux ARM64 glibc, Windows x64/ARM64 release targets | Source and release workflows cover all four Rust surfaces; Spec 132 terminal matrix proof remains open until all native hosted/runtime proofs and E7 target builds are complete. |
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
   - Rust service/install paths report systemd user units, launchd user agents, skipped service, or warning states truthfully. Windows service registration must remain warning/skipped unless native host support is actually available and proven.

5. **Spec 132 installer terminal proof**
   - Linux/plain/CI/TERM=dumb/NO_COLOR/reduced/static fixture coverage exists in repo evidence.
   - Native Windows ConPTY and macOS interactive proof must remain unclaimed until hosted runtime evidence is available.
   - Runtime fixture scripts require a built `target/debug/focusa`; do not substitute static proof for executable PTY/ConPTY proof.

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

### A.3 Transport options (auto-detected order, commercialization-safe defaults)

Mass-adoption / long-term policy: **default transports must be open-source-reusable-licensed, self-hostable, and require no third-party account**. Vendor coordination-server tunnels are **opt-in interop only**, never defaults. Single-vendor lock-in is rejected by `tests/spec_pairing_portable_architecture_static_test.sh`.

**Default order (all permissive-licensed, self-hostable, no third-party account):**

1. `FOCUSA_PAIRING_URL` env var (operator-supplied)
2. `/etc/focusa/public-url` file (operator-supplied)
3. `FOCUSA_API_URL` / `FOCUSA_BASE_URL` (non-local only)
4. Hostname `https://<host>`, `http://<host>`, `http://<host>:8787`
5. Public IPv4 variants
6. Private IPv4 (LAN / `ssh`-reachable only)
7. **`ssh -R` reverse tunnel** to an operator-controlled jump host (OpenSSH, BSD-style)
8. **`frp`** (Apache 2.0, self-hostable Fast Reverse Proxy)
9. **`bore`** (MIT, self-hostable TCP tunnel)
10. `http://127.0.0.1:8787` (same-machine only)

**Vendor interop (opt-in only — never default, never bundled):**

| Tunnel | License | Why opt-in | Enable flag |
|---|---|---|---|
| `cloudflared` quick tunnel | Apache 2.0 | requires Cloudflare coordination server; account-less quick tunnels have no SLA and may be investigated for TOS | `FOCUSA_TUNNEL_CLOUDFLARED=1` |
| Tailscale Funnel | BSD-3 (client) | requires Tailscale account + coordination server; pricing/terms change unilaterally | `FOCUSA_TUNNEL_TAILSCALE=1` |
| `ngrok` quick tunnel | BSD-3 (client) | requires ngrok account; service-side TOS | `FOCUSA_TUNNEL_NGROK=1` |

`localhost.run` is **not supported** (unclear license, single-operator SSH relay).

**Bundling policy.** The Focusa installer **never bundles vendor binaries**. If an operator opts into a vendor transport, the installer downloads the binary at runtime from the vendor's official release, and the install step records that vendor's `LICENSE` and `NOTICE` files under `/usr/local/share/focusa/licenses/<vendor>/` so attribution is preserved. Self-hostable transports (`frp`, `bore`) are also not bundled by default — the installer records the operator-chosen mode and downloads only what the operator enabled.

**Service-dependency risk.** Vendor coordination servers are single points of failure and can change pricing, terms, or shut down. For mass adoption we want every operator able to run with **zero third-party coordination server** — that's the explicit motivation for the self-hostable default order.

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

## Addendum B — Reproducible Builds & Release Signatures (2026-06-27)

Mass-adoption builds require reproducible artifacts and verifiable signatures.

- **Pinned toolchain.** Every release workflow job pins `dtolnay/rust-toolchain@nightly` to `nightly-2026-01-08`. Dev branches may float to current nightly but tagged releases are rebuilt from the exact pinned toolchain.
- **Reproducible builds.** The release workflow targets `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-apple-darwin`, `x86_64-apple-darwin`. Source maps and debug symbols are stripped from release artifacts. Bit-for-bit reproducibility requires deterministic Rust builds (CARGO_INCREMENTAL=0, CARGO_PROFILE_RELEASE_LTO=fat, fixed source date). The release workflow sets `SOURCE_DATE_EPOCH` from the git tag timestamp.
- **Signed checksum manifest.** Each release publishes `SHA256SUMS.txt`, `SHA256SUMS.txt.sig` (cosign keyless), and `SHA256SUMS.txt.pem`. The installer verifies with optional `FOCUSA_REQUIRE_COSIGN=1` fail-closed mode.
- **Reproducibility test.** A nightly job rebuilds the latest tag and diffs SHA256SUMS.txt against the published manifest. Any diff fails CI.

## Addendum C — Unsigned-Mac Placeholder Until First Revenues (2026-06-27)

Operator decision: the $99 Apple Developer Program enrollment is **deferred until Focusa makes its first revenues**. Until then, `Focusa.app` ships **unsigned** and Apple Silicon users see Gatekeeper’s “developer cannot be verified” warning. This addendum documents the supported path so users aren’t surprised and so the operator knows exactly what is missing.

### C.1 What still works (no Apple Developer credentials required)

- Full Mac pairing flow: `Open Focusa.app → first-run Mac QR → Focusa Connect Page → token stored in Keychain → authenticated API calls → restart preserves token → revoke/re-pair`.
- All `focusa install-service`, `focusa pairing transport setup`, `focusa codesign inspect`, `focusa pair` commands.
- `focusa codesign sign` is built and ready; only the credentials and a Mac are missing.

### C.2 What requires operator action today

- **First-launch override.** Apple Silicon users must right-click → Open → confirm Open in the dialog. Subsequent launches may still prompt; users can allow once via “Open Anyway” in System Settings → Privacy & Security.
- **Notarized download.** Operators should NOT advertise `Focusa.app` as “verified” or “safe to install without warnings” while unsigned.

### C.3 What unlocks when first revenue hits

- Purchase Apple Developer Program enrollment.
- Acquire Team ID, Developer ID Application certificate, app-specific password.
- Run: `focusa codesign sign --app-path dist/macos/Focusa.app --developer-id "Developer ID Application: <Team>" --team-id <id> --apple-id <email> --app-specific-password <pwd> --json`.
- Re-tag Mac release and replace artifacts in `release/v0.9.34-dev` and later with signed+notarized+stapled bundles.
- Close `focusa-covz` with proof: `release:v0.9.x-dev ; spctl --assess passes ; gate:mac`.

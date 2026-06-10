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

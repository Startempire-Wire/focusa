# Public Docs MVP Sync — 2026-06-10

## Scope

MVP readiness cleanup for Focusa Operator Preview snapshot `v0.9.14-dev`.

## Completed MVP audit items

- B1: `focusa status` now displays app semver (`app version`) and labels the reducer counter separately.
- B2: README documents scoped CLI usage with `focusa project identity`, `--project-root`, and `--continuity-id` workpoint examples.
- B3: Snapshot metadata bumped from `0.9.13-dev` to `0.9.14-dev`; local tag `v0.9.14-dev` points at the publish HEAD.
- B4: `apps/focusa-awareness` builds standalone without importing the unavailable OpenClaw SDK package.
- B5: README and `docs/current/focusa-tool-contracts.json` agree on 79 Pi tools/contracts.

## Runtime proof

```text
focusa status
  app version: 0.9.14-dev
  reducer: <monotonic reducer counter>
  daemons: 0

curl http://127.0.0.1:8787/v1/health
  {"ok":true,"version":"0.9.14-dev"}
```

## Build proof

```bash
cargo build --release --package focusa-api --package focusa-cli
cd apps/focusa-awareness && bun run check && bun run build
```

## Scope-safety proof

`/v1/project/identity?project_root=/home/wirebot/wirebot-core` returns `status=degraded` without a matching `.focusa-project.json` root marker, preventing non-marker git/beads folders from becoming verified project authority.

## Git proof

- HEAD: `8ce7c10 docs: sync tool contract count metadata`
- Tag: `v0.9.14-dev`

## Menubar pairing hardening update

After the initial MVP sync, the Mac menubar pairing path was hardened before release:

- `apps/menubar/src-tauri/src/main.rs` exposes Tauri commands for macOS Keychain token save/load/clear.
- `apps/menubar/src/lib/stores/pairing.svelte.ts` no longer stores bearer tokens in localStorage; it stores only metadata and a token preview.
- Paired daemon requests attach `Authorization: Bearer <token>` from the in-memory token loaded from Keychain.
- `DevicePairCompletion` is now wired into `/v1/device/pair/complete` response internals instead of remaining an unused core type.

Validation:

```bash
cargo clippy --workspace -- -D warnings
cargo test --workspace
cd apps/menubar && bun run check && bun run build
```

Native Linux Tauri build remains blocked on this AlmaLinux host by `glib-2.0 >= 2.70` (host has 2.56.4), so the final `.app` artifact must be produced by the GitHub macOS release job or on the operator Mac.

## GitHub release publication proof

Release workflow `27323208477` completed successfully for tag `v0.9.14-dev`.

Release URL: https://github.com/Startempire-Wire/focusa/releases/tag/v0.9.14-dev

Published Mac menubar assets:

- `Focusa_0.9.14-dev_aarch64.dmg`
- `Focusa_0.9.14-dev_x64.dmg`
- `Focusa_aarch64.app.tar.gz`
- `Focusa_x64.app.tar.gz`

Published companion CLI/daemon assets include Apple Silicon, Intel macOS, and Linux binaries for `focusa`, `focusa-daemon`, and `focusa-tui`.

## Menubar diagnostics update

Pairing failures now expose a copyable diagnostics log in the app.

Implemented surfaces:

- `apps/menubar/src/lib/stores/diagnostics.svelte.ts` — timestamped local diagnostics ledger with error classes for network, timeout, HTTP, JSON parse, Keychain, global JS, unhandled promise rejection, and pairing phases.
- `apps/menubar/src/routes/+layout.svelte` — installs global `window.error` and `unhandledrejection` capture.
- `apps/menubar/src/lib/api.ts` — records generic API/network/HTTP/JSON failures.
- `apps/menubar/src/lib/stores/pairing.svelte.ts` — records pairing start/poll/list/revoke/bootstrap failures with URL, method, status, failure class, body, stack, and timestamp where available.
- `apps/menubar/src/lib/components/PairingPanel.svelte` — shows error timestamp/class/phase and a **Copy error log** button.

Validation:

```bash
cd apps/menubar && bun run check && bun run build
```

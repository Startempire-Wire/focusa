# Mac Menubar Pairing Partial Local Proof — Actual Mac E2E Pending — 2026-06-15

Work item: `focusa-ui0y.15`  
Project: `/home/wirebot/focusa`  
Continuity: `focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61`

## Result

Status: **partial/local proof only — actual Mac E2E is not complete**. This file is evidence of API/web/build behavior and blockers, not release signoff.

Validated now:

- Pairing daemon API start → pending status → complete → completed status → list → revoke/token-cleared flow.
- Menubar Svelte production web artifact build.
- Pairing UI source exists and is covered by `svelte-check` / Vite build.
- Native Tauri artifact build attempted and failed for the known AlmaLinux GLib blocker.

Not validated on this host:

- Real macOS `.app` launch.
- macOS Keychain persistence through app restart.
- Native Tauri invoke/window/menu lifecycle.

## Real API pairing proof

Command family used: `curl http://127.0.0.1:8787/v1/device/pair/*`.

Observed proof summary:

1. `POST /v1/device/pair/start`
   - `status=completed`
   - `code=FOCUS-019ECA5C-4F7B`
   - `device_id=019eca5c-fce5-75c1-a9cf-4a3d91c0096f`
   - `pair_url=https://focusa.example.test/pair/019eca5c-fce5-75c1-a9cf-4a3d91c0096f`
   - `scopes=[read, write]`
2. `GET /v1/device/pair/status?code=...` before completion
   - `status=pending`
   - `token_present=false`
3. `POST /v1/device/pair/complete`
   - `status=completed`
   - `token_present=true` (token value intentionally not recorded)
   - `scopes=[read, write]`
4. `GET /v1/device/pair/status?code=...` after completion
   - `status=completed`
   - `token_present=true`
5. `GET /v1/device/pair/list?host=operator-vps&limit=5`
   - `status=completed`
   - returned recent paired device records
6. `POST /v1/device/pair/revoke`
   - `status=completed`
   - target `device_id=019eca5c-fce5-75c1-a9cf-4a3d91c0096f`
7. `GET /v1/device/pair/status?device_id=...` after revoke
   - `status=completed`
   - `token_present=false`

## CLI proof / gap

Human CLI proof exists for `focusa device pair-start` and prints a dumb-simple operator command:

```text
device pair start | code=... device_id=... expires_in=300s
on_your_vps_run: focusa device pair-complete <code> --host <host>
```

Gap found: `focusa --json device pair-start` still emitted human text, not JSON. This is tracked by the existing CLI JSON parity ready beads (`focusa-531p`, `focusa-nb30`) and was not fixed in this Mac checklist bead.

## Menubar web artifact proof

Command:

```bash
npm --prefix apps/menubar run build
```

Result: **passed**.

Warnings only:

- `PairingPanel.svelte`: unused CSS selector `.code`
- `ToolsRegistryPeek.svelte`: unused CSS selector `.proof-row section p`

Artifact path:

- `apps/menubar/build/`

Relevant UI/runtime source refs:

- `apps/menubar/src/lib/components/PairingPanel.svelte`
- `apps/menubar/src/lib/focusaClient.ts`
- `apps/menubar/src-tauri/src/main.rs`
- `apps/menubar/src-tauri/Cargo.toml`

## Native Tauri local artifact proof

Command attempted:

```bash
npm --prefix apps/menubar run tauri -- build --debug
```

Result: **blocked on AlmaLinux host native dependency**.

Observed blocker:

```text
error: failed to run custom build command for `glib-sys v0.18.1`
The system library `glib-2.0` required by crate `glib-sys` was not found.
```

This matches the existing native coverage blocker recorded on `focusa-qasy.25`: AlmaLinux provides older GLib than the Tauri GTK stack expects. Upgrading system GTK/GLib on the production VPS is outside safe automatic changes.

## UIAI browser proof blocker

Attempted UIAI browser open:

- `http://127.0.0.1:1420`
- `http://localhost:1420`

Result: **blocked by UIAI private/internal URL guard**.

Observed error:

```text
UIAI 400: URL not allowed: private/internal addresses blocked (set allow_private_urls to enable)
```

## Required Operator Mac E2E checklist — still unfinished

Run on an actual Mac before release signoff:

1. Install/open the latest Focusa menubar `.app`.
2. Set daemon URL to the VPS/public Focusa daemon URL.
3. In Pair tab, start pairing and confirm code + QR/link are visible.
4. Complete from VPS/phone:
   - `focusa device pair-complete <code> --host operator-vps --operator-id verious`
5. Confirm Mac poll changes from pending → completed.
6. Confirm token stored in macOS Keychain.
7. Quit and relaunch app.
8. Confirm paired state survives restart and Authorization header is used.
9. Confirm `focusa device pair-list --host operator-vps` shows the Mac.
10. Revoke the device.
11. Confirm Mac returns to unpaired/needs-pairing state.
12. Capture screenshots/logs for Pair tab, Settings, completion state, restart state, list, revoke.

## Closure posture

This bead **must remain open or blocked** until actual Mac-native evidence exists. Local API/web proof is useful but insufficient for completion. Required missing evidence: macOS `.app` launch, Keychain token persistence, app restart persistence, native Tauri invoke/window/menu lifecycle, screenshots/logs, list/revoke confirmation from the Mac app.

# Pi Extension Mission Canvas Stale-Ctx Crash Resolution

**Issues:** Startempire-Wire/focusa#124, Startempire-Wire/focusa#301 (P0 escalation)
**Date:** 2026-08-15
**Affected surface:** `apps/pi-extension` Mission Canvas work-rail widget (deployed line v0.9.152)

## Root cause

The Mission Canvas widget cached the extension/session context (`latestUiContext`)
from the last `session_start`/`turn_end` event and reused it from background work:

1. `ensureRefreshLifecycle()` stores the ctx in a module global.
2. `publishScopedStateChange()` fans receipts out to subscribers from a
   `queueMicrotask` — outside Pi's extension-handler protection.
3. The subscriber calls `refreshMissionCanvasWidget(latestUiContext)`, which
   reads `ctx.hasUI`.

After any session replacement or reload (`ctx.newSession()` — including the
extension's own session-transfer command — `fork`, `switchSession`, or
`reload`), Pi marks the old ctx stale; every property access throws
`assertActive`. The throw escaped through the microtask as an
`uncaughtException` and terminated the Pi process.

The earlier fix attempt did not hold because no layer of the chain
(`scoped-surface-refresh.ts:93` dispatch → `mission-canvas-widget.ts:167`
listener → `:66` `ctx.hasUI` read) had any guard.

## Fix (defense in depth)

Installed at `/root/.pi/agent/extensions/focusa/` (and its hardlinked twin
`focusa-runtime/` — both directories are the same inodes).

1. `src/mission-canvas-widget.ts`
   - `refreshMissionCanvasWidget()` body wrapped in try/catch; a stale handle
     drops the refresh and clears `latestUiContext` (silent degradation).
   - Both polling calls (`30s` interval, `1s` post-session-start) now append
     `.catch(() => {})` so a dead handle cannot become an unhandled rejection.
2. `src/scoped-surface-refresh.ts`
   - Dispatch microtask loop guards each listener call in try/catch; a
     crashing subscriber can no longer take the process down.
3. `src/index.ts`
   - `ctrl+shift+f` (Show Focusa status) rebound to `ctrl+shift+e`; the old
     combo collided with Pi's built-in `tui.altScreen.search`.

## Verification evidence

- TypeScript typecheck: clean (`tsc -p tsconfig.json`).
- `npm run test:surface-refresh`: 10/10 passing.
- New regression test `tests/mission-canvas-stale-ctx-regression.test.mjs`:
  4/4 passing. Simulates Pi's `assertActive` guard via a throwing ctx proxy
  through the widget refresh and the real dispatch loop.
- Live verification: Pi restarted 2026-08-15 11:46:27 PDT with the patched
  code (files written 11:43:51). Fresh session ledger shows zero errors and
  normal Focusa receipt/anchor writes.

## Backups and follow-ups

- Pre-fix src backups: `/root/.pi/focusa-ext-fix-backup-20260815/`.
- Remaining risk: Pi OTA activation can overwrite the installed extension
  (tracked separately in issue #309 — crash-safe OTA commit/rollback).
- The canonical repo copy at `apps/pi-extension` is the older 0.9.135-dev
  line and predates the `scoped-surface-refresh` caching architecture; it does
  not contain this code path and was not modified.

# Spec 135 F13/F14 Rich Host Evidence and Receipt

## F13 — Canvas ON native rich-host lifecycle

Evidence:

- `apps/pi-extension/src/rich-host/lifecycle.ts` enforces one host per exact attachment and focuses an existing window on repeated ON.
- `apps/pi-extension/src/rich-host/platform.ts` independently probes host capability and resolves native, web, TUI, or headless renderers.
- `apps/pi-extension/tests/rich-host-lifecycle.test.mjs` proves singleton launch/focus, lifecycle writes, signed asset verification, and projection revision discipline.
- `apps/pi-extension/tests/rich-host-entrypoint.integration.mjs` performs a real extension-to-host process handshake and loopback frontend load.

Receipt: `receipt:spec135:f13:rich-host-on:v1`

## F14 — Canvas OFF and stock Pi restoration

Evidence:

- `RichHostLifecycleManager.off()` hides or closes the attachment-owned host and records the durable lifecycle transition.
- `mission-canvas-tool.ts` always invokes the existing Mission Canvas controller after host OFF, preserving the stock Pi shell restoration path.
- `session_shutdown` closes owned host processes and removes handshake files.
- Tests prove hide, close, and cleanup behavior without requiring a graphical display.

Receipt: `receipt:spec135:f14:rich-host-off:v1`

## Verification

```bash
cd apps/pi-extension
npm run typecheck
npm run test:rich-host
npm run test:mission-canvas
```

Observed result: all commands passed on the implementation worktree. Cross-platform repetitions are defined in `.github/workflows/spec135-rich-host-smoke.yml` for macOS, Windows, and Linux with Node 20 and 22.

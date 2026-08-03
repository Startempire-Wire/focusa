# ADR-0135: Portable Pi Mission Canvas rich-host lifecycle

- Status: Accepted
- Scope: Spec 135 P04
- Call-stack design: `019fb6a3-d81b-7ad2-8b9f-444089c1259d`

## Decision

Use the repository's existing Tauri 2 stack as the packaged native window/webview host. The Pi extension remains the lifecycle and attachment authority and communicates with the Focusa daemon through the generated Mission Canvas HTTP/event contracts. It never moves canonical workspace state into the window process.

When the packaged Tauri binary is unavailable, resolution falls back in this order: system browser window, stock Pi/TUI projection, then headless operation. Interaction mode and renderer availability are independent inputs; `canvas-guided` does not imply that a native binary exists.

## Call stack

1. Pi command/tool: `focusa_mission_canvas` and `/mission-canvas`.
2. `RichHostLifecycleManager`: attachment-scoped singleton, launch/focus/hide/close, heartbeat, reconnect.
3. `MissionCanvasApiClient`: scoped projection, event, host-lifecycle, and draft calls.
4. `RichHostRendererResolver`: platform/capability probe independent of interaction mode.
5. `RichHostAssetVerifier`: version, SHA-256, and Ed25519 manifest verification.
6. `RichHostProcessAdapter`: macOS, Windows, Linux, fake-test, TUI, and headless adapters.
7. Focusa Core Mission Canvas repository/event authority.
8. Generated projection rendered by the host frontend; Evidence and Receipt refs returned to Pi.

## IPC and authentication

The extension writes a mode-`0600` one-time handshake file containing daemon base URL, bearer token, exact scope, protocol version, expiry, and nonce. The packaged host receives only the handshake path and expected digest through its environment. It must atomically consume and delete the file before opening a window. Tokens are never placed in command arguments, URLs, logs, projection caches, or frontend local storage.

The host accepts daemon responses only when the returned scope exactly matches project root, continuity, session, and attachment. Revision regressions and attachment mismatches fail closed.

## Packaging

```text
apps/pi-extension/rich-host/
  assets/                 # immutable generated web assets
  manifests/<version>.json
  bin/darwin-{arm64,x64}/focusa-rich-host
  bin/win32-x64/focusa-rich-host.exe
  bin/linux-{x64,arm64}/focusa-rich-host
```

The npm package may omit native binaries. Capability probing reports that truthfully and selects a fallback. CI verifies each packaged manifest and runs platform adapter smoke tests without requiring a graphical display.

## Lifecycle invariants

- One rich window per exact Pi attachment.
- `ON` launches or focuses; it never creates a duplicate.
- `OFF` hides or closes according to policy and restores stock Pi presentation.
- Close button updates durable lifecycle state before process exit.
- Pi process exit closes owned hosts and removes handshake files.
- Daemon restart reconnects from the last durable event cursor and rejects stale projections.
- Unsent Pi and Canvas drafts are preserved independently and synchronized by revision.

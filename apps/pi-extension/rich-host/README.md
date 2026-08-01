# Focusa Pi Rich Host

This directory is the portable package boundary for the Mission Canvas rich host.

- `host-entrypoint.mjs` is the signed-package webview/browser fallback.
- `assets/` contains immutable renderer assets.
- `manifests/<version>.json` contains SHA-256 and Ed25519 metadata generated during release.
- `bin/<platform>-<arch>/` contains optional Tauri 2 binaries.

Native binaries are optional. Capability probing must report their absence and select a system-webview, stock Pi/TUI, or headless fallback. Release packaging must never generate or commit signing private keys.

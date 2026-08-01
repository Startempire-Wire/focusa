# Spec 135 Pi-native Mission Canvas hardening proof

The authoritative Mission Canvas renderer is `apps/pi-extension/src/mission-canvas-view.ts`, mounted by `MissionCanvasShell` through Pi's `ctx.ui.custom()` in the current terminal. No browser, webview, sidecar, or separate rich-host process is part of the product path.

## Security boundary

- Projection text is normalized before terminal emission.
- ANSI escape sequences and control characters are stripped from untrusted model fields.
- Rendering uses bounded contribution selection and `virtualWindow` surface selection.
- Input actions are explicit allowlisted mode/profile/surface transitions.
- OFF disposes the custom component and restores stock Pi in the same session.
- No HTML injection, browser storage, remote renderer, or network token is used by the interactive surface.

## Resilience and accessibility

- `mission-canvas-performance.test.mjs` runs 5,000 transitions and bounds heap growth.
- `mission-canvas-accessibility.test.mjs` verifies keyboard, responsive, reduced-motion, high-contrast, and minimum-width behavior.
- Reference tests include hostile ANSI/OSC input and assert that payloads do not reach rendered output.
- `spec135-pi-extension-npm-audit.json` reports zero production vulnerabilities.
- `spec135-pi-extension-sbom.cdx.json` records the portable extension dependency inventory.

## Explicit absence gate

The following active implementation surfaces are absent:

- `apps/pi-extension/rich-host/`
- `apps/pi-extension/src/rich-host/`
- `apps/pi-extension/tests/rich-host-*.mjs`
- `apps/pi-extension/tests/run-rich-host-lifecycle.mjs`

UIAI fixture routes remain projection/evidence contract tests only; they do not claim renderer authority.

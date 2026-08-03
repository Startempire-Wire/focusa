# Spec 135 Generated C.R.I.S.T. and UIAI Rich Surface Proof

## Generated runtime

The rich host permanently loads `a2ui-runtime.js`, built from `@focusa/a2ui-renderer`, A2UI 0.9.1, Lit 3.3.1, and the Focusa Custom Elements catalog. `focusa-generated-surface` accepts snapshot/delta messages and permits actions only when their canonical Operation Registry identifiers are projected as enabled.

Context, Role, Interview, Spec, and Tasks use the same generated-surface renderer and catalog. Missing messages render bounded progress/recovery controls rather than an empty shell.

## UIAI boundary

The UIAI Work Surface renders bounded screenshot, snapshot, diagnostic, and artifact projections. It does not embed or replace UIAI Engine Cockpit, evaluate page code, or accept arbitrary screenshot origins. Browser operations remain governed operation bindings owned by UIAI.

## Evidence

- `packages/a2ui-renderer/tests/renderer.test.mjs`: deterministic snapshot/delta traversal, unknown component/action fail-closed behavior, protocol and payload bounds.
- `tests/spec135_generated_browser_surface_test.py`: permanent bundle, catalog, action, recovery, browser-view, and isolation assertions.
- `apps/pi-extension/tests/rich-host-frontend.test.mjs`: no-dead-DOM, accessibility, renderer, and interaction checks.
- `packages/a2ui-renderer/src/rich-host.ts`: runtime traversal custom element.

Receipts:

- `receipt:spec135:generated-rich-runtime:v1`
- `receipt:spec135:uiai-surface-isolation:v1`

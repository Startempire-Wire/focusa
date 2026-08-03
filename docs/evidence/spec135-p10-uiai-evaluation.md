# Spec 135 P10 UIAI Evaluation Evidence

## Governed harness

`apps/pi-extension/tests/mission-canvas-uiai-server.mjs` provides canonical seeded projections, idempotent scenario reset, projection/evidence/receipt correlation, and populated, empty-optionals, single-queue, and zero-queue scenarios.

`uiai-eval-harness.test.mjs` executes all scenarios, proves candidate partition completeness, verifies omitted contributions have no layout trace, checks queue occupancy, and compares deterministic layouts. The viewport matrix covers minimum, standard, productive, wide/reference, macOS, Windows, Linux, high contrast, scaling, and reduced motion. The thirteen proof definitions are in `tests/fixtures/spec135-thirteen-no-dead-chrome-proofs.json`.

## Pi-native reference and live evidence

- `docs/evidence/spec135-pi-native-reference-renders.v1.json`
- `docs/evidence/spec135-pi-native-reference-renders.png`
- `docs/evidence/spec135-pi-native-live-capture.png`
- `docs/evidence/spec135-pi-native-off-restoration.ansi`

These prove the current-terminal host, activity/profile recomposition, actual Work Surface strip, semantic contribution cards, sparse-state omission, same-session transcript/editor, queue occupancy, responsive layout, and OFF restoration.

## UIAI Engine evaluation

The local UIAI daemon was restarted from the checked-out engine config with `vision.allow_private_urls: true`, a local writable storage directory, and its one-page pool reset. Only four non-sensitive proof images were copied to an isolated temporary directory. That directory was served through an ephemeral Cloudflare quick tunnel; no repository, daemon endpoint, token, or secret was exposed.

UIAI opened the proof page in session `k1G5cirJ` and loaded:

1. authoritative activity-mode reference;
2. authoritative vertical-profile reference;
3. deterministic Pi-native reference contact sheet;
4. live installed-Pi Canvas capture.

Evidence:

- screenshot: `docs/evidence/spec135-uiai-reference-comparison.png`
- diagnostics: `uiai-diagnostics:session=k1G5cirJ:seq=6`
- browser read: `uiai-browser:session=k1G5cirJ:read:1`
- console errors: `0`
- JavaScript exceptions: `0`
- HTTP 5xx: `0`
- one non-material favicon HTTP 404

The UIAI session, public tunnel, and temporary proof server were closed immediately after capture.

## Promoted artifacts

- `receipt:spec135:p10:fixture-harness:v1`
- `receipt:spec135:p10:thirteen-no-dead-chrome:v1`
- `evidence:uiai:populated`
- `evidence:uiai:empty-optionals`
- `evidence:uiai:single-queue`
- `evidence:uiai:zero-queues`
- `uiai-diagnostics:session=k1G5cirJ:seq=6`
- `docs/evidence/spec135-uiai-reference-comparison.png`

Status: **verified**. The prior loopback-policy blocker is resolved on this Mac.

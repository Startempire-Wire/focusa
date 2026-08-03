# Spec 135 Rich Host Hardening Evidence and Receipts

## Continuity and recovery

- Exact attachment identity remains stable across ON/focus/OFF cycles.
- Pi and Canvas drafts use separate durable draft revisions.
- SQLite event IDs deduplicate replay and revision guards reject stale writers.
- Focus, semantic selection, transcript scroll, layout memory, capability suspension, and contribution return are preserved by reducers and renderer state.

Receipt: `receipt:spec135:continuity-recovery:v1`

## Security and supply chain

- Threat model: `docs/security/spec135-rich-host-threat-model.md`.
- CycloneDX SBOMs: `spec135-pi-extension-sbom.cdx.json`, `spec135-a2ui-renderer-sbom.cdx.json`.
- Production dependency audit: both npm audit reports contain zero known vulnerabilities.
- CSP, trusted loopback origin, private one-time handshake, exact scope, signed assets, and A2UI/UIAI trust boundaries are gate-tested.

Receipt: `receipt:spec135:rich-host-security:v1`

## Accessibility and responsive behavior

- Keyboard access covers navigation, dialogs, profile/activity switching, prompt delivery, and direct-manipulation alternatives.
- Semantic regions, labels, live status, focus restoration, forced-colors, reduced-motion, 200% text fixtures, and 40px minimum targets are enforced.
- Responsive vectors cover 1024×720 through 1920×1080 on macOS, Windows, and Linux.

Receipt: `receipt:spec135:rich-host-accessibility:v1`

## Performance

- Resolver benchmark: 10,000 candidates under two seconds in debug tests (observed suite total under one second after link).
- Transcript, rail, evidence, and surface lists use bounded virtual windows and browser content visibility.
- Recomposition performs one detached-tree replacement and interruptible transition.
- A 1,000-cycle host stress test asserts singleton identity, cleanup, under-10-second execution, and under-64MiB heap growth.

Receipt: `receipt:spec135:rich-host-performance:v1`

## Q1–Q6 rich-host revalidation

| Gate | Rich-host result |
|---|---|
| Q1 canonical authority | Projection and mutations remain Focusa Core owned; host is a renderer/client. |
| Q2 no dead chrome | Eligibility omissions plus no-dead-DOM/layout invariants pass. |
| Q3 lifecycle continuity | ON/OFF singleton, drafts, focus, replay, and reconnect tests pass. |
| Q4 portability | macOS/Windows/Linux adapters, scaling fixtures, TUI, and headless fallbacks pass. |
| Q5 accessibility/security | Keyboard, semantics, contrast, text scaling, CSP, scope, secrets, and generated-UI boundaries pass. |
| Q6 performance/recovery | Resolver, virtualization, 1,000-cycle stress, cleanup, stale-write, and restart tests pass. |

## Commands

```bash
cargo test -p focusa-core mission_canvas --lib
cargo test -p focusa-api mission_canvas::tests
cd apps/pi-extension && npm run typecheck && npm run test:rich-host
python3 tests/spec135_rich_host_hardening_test.py
```

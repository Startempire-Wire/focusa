# Focusa Menubar (Preview)

**Status: preview / not flagship.** Svelte + Tauri menubar app for macOS that
shows live focus, workpoint, and trajectory state from a paired Focusa daemon.

## What works in v0.9.142 (preview)

- SvelteKit/Tauri build with type-checked Settings, pairing, project, Mission Canvas, Work Rail, and runtime surfaces
- OAuth-like device pairing with the Focusa daemon (CLI / QR + phone / QR + VPS), including revocation-aware device state
- HTTP/WebSocket binding to a paired daemon over loopback or Tailscale
- Scope-aware status surfaces for project identity, Trajectory, Workpoint, evidence, predictions, and bounded live refresh
- Release packaging and static runtime gates remain covered; this app is still a preview surface, not the flagship deployment path

### macOS incoming-network prompt

The phone-bridge LAN callback is disabled by default. Pairing uses bounded room-status polling, so a normal install does not need to accept incoming network connections. Operators who explicitly want the optional low-latency LAN callback may launch with `FOCUSA_PHONE_BRIDGE_LAN_CALLBACK=1`; macOS will then show its expected firewall prompt because Focusa binds a temporary LAN listener for at most 30 seconds.

## Tracked testing work (NOT launch-ready)

- Native macOS `.app` lifecycle (launchd persistence, restart, screenshot/log capture)
- Keychain persistence for tokens, device IDs, and pairing secrets
- Real notarization / Developer ID signing (currently uses ad-hoc / Personal Team)
- Apple silicon launch + Gatekeeper + TCC acceptance on first run

## Do NOT promote as flagship

Phase 1 of the MVP launch uses the daemon / CLI / TUI / Operator Preview as the
main surface. This menubar app is intentionally positioned as a **preview** so
public-launch messaging does not over-claim macOS readiness.

See `docs/PHASE2_OPERATOR_PREVIEW.md` for the controlled preview cohort plan
and `docs/PHASE3_PRODUCT_HUNT_READINESS.md` (TBD) for the launch surface.

# ADR — Mission Canvas uses the current Pi TypeScript TUI

**Status:** accepted  
**Authority:** operator clarification, authoritative adaptive-composition handoff, Spec 135A/135G

## Decision

Mission Canvas mounts through `ctx.ui.custom()` inside the current Pi terminal and current Pi process. Canvas ON replaces only Pi's visible root; Canvas OFF disposes the custom component and restores stock Pi. The same `ExtensionContext`, `SessionManager`, model stream, transcript, tools, attachments, prompt target, and canonical Focusa runtime remain active.

Browser, webview, Tauri, sidecar, and remote-host Canvas launch paths are forbidden for the primary Pi Canvas. Menubar, PWA, and UIAI remain bounded clients with separate responsibilities.

## Consequences

- `focusa_mission_canvas` delegates to the same controller as `/mission-canvas` and reports renderer `pi_tui`.
- `MissionCanvasShell` is authoritative presentation, not a compatibility placeholder.
- The adaptive handoff images are structural/visual references resolved responsively within terminal cell geometry.
- Browser-based rich-host code may remain only as historical or separately governed fallback code; Canvas control cannot import or launch it.
- Visual proof includes deterministic ANSI/PNG renders and a live current-Pi capture after `/reload`.

# Agent TUI Integration Contract

Status: live xterm frontend complete; native PTY bridge pending

Browser Evidence: `docs/contracts/evidence/spec158-desktop-agent-tui-integrated.png` and `uiai-diagnostics:session=b0R1Oebd:seq=0` (zero console errors, warnings, exceptions, or failed requests).

## Product boundary

Agent TUI is a complete Focusa Desktop inner surface, not a generic terminal panel and not a second canonical state store. It preserves authentic Pi interaction while integrating Focusa Workstream context, Work Rail, Evidence posture, steering, lifecycle, and recovery presentation.

The outer Desktop titlebar and Mission Canvas/Agent TUI switch remain stable. Agent TUI has no application sidebar.

## Exact attachment identity

Native input is unavailable until the generated identity chain is present:

1. `ScopeRef` / `ProjectRootKey`
2. `WorkstreamId`
3. optional `ContinuityId` lineage
4. `AttachmentKey`
5. `SessionId` / `InstanceId`
6. `WorkspaceBindingId`
7. runtime object identity
8. `WorkSurfaceId`

The typed contract is implemented in `src/lib/shell/pi-attachment-contract.ts`. Project root, current tab, CWD, remembered workspace selection, latest session, and daemon-global state are not substitutes.

## Frontend surface

`src/lib/shell/AgentTuiSurface.svelte` and `PtyTerminal.svelte` provide:

- integrated Agent TUI identity and verified runtime status;
- a real xterm terminal renderer rather than a fabricated transcript;
- exact Attachment-gated input, resize, output subscription, interrupt, attach, and detach commands;
- responsive terminal fitting through `ResizeObserver`;
- bounded scrollback and authentic ANSI terminal rendering;
- truthful unbound, disconnected, bridge-unavailable, and error presentation;
- no sidebar and no fake terminal controls.

The visual treatment follows the useful pi.dev terminal principles—content-first dark terminal, monospaced typography, compact padding, and no decorative browser chrome—while using a live PTY stream instead of pi.dev's prerecorded Asciinema casts.

## Native bridge boundary

The future Tauri bridge accepts only typed `PiNativeCommand` envelopes:

- `attach` with exact identity and terminal geometry;
- `input` with the exact generated AttachmentKey's `attachment_id` handle;
- `resize` with the exact generated AttachmentKey's `attachment_id` handle and geometry;
- `interrupt` with the exact generated AttachmentKey's `attachment_id` handle;
- `detach` with the exact generated AttachmentKey's `attachment_id` handle.

The bridge must own PTY spawn, process-group lifecycle, resize, byte-stream output, backpressure, reconnect, and shutdown. Frontend state cannot mint or repair an Attachment. Native events must return the exact Attachment identity they belong to; mismatched or stale output is rejected rather than routed to the visible tab.

## Acceptance gates

Frontend gate:

- complete Mission Canvas replacement;
- no sidebar;
- readable terminal at wide and responsive sizes;
- keyboard-accessible tabs and lifecycle controls;
- disabled controls are visibly unavailable;
- reduced motion has a static cursor/orb fallback;
- zero browser console errors, exceptions, and failed requests.

Native gate:

- one resource-bounded development build;
- real Pi process under native PTY ownership;
- exact Attachment verification before input;
- terminal resize and UTF-8/ANSI stream proof;
- interrupt and detach process-group proof;
- session resume/reconnect proof;
- stale output rejection proof;
- native screenshot and bounded Evidence references.

Release remains prohibited until the canonical transition and parity gates are complete.

# Agent TUI Integration Contract

Status: frontend integration shell browser-accepted; native PTY bridge pending

Browser Evidence: `docs/contracts/evidence/spec158-desktop-agent-tui-integrated.png` and `uiai-diagnostics:session=b0R1Oebd:seq=0` (zero console errors, warnings, exceptions, or failed requests).

## Product boundary

Agent TUI is a complete Focusa Desktop inner surface, not a generic terminal panel and not a second canonical state store. It preserves authentic Pi interaction while integrating Focusa Workstream context, Work Rail, Evidence posture, steering, lifecycle, and recovery presentation.

The outer Desktop titlebar and Mission Canvas/Agent TUI switch remain stable. Agent TUI has no application sidebar.

## Exact attachment identity

Native input is unavailable until all identity fields are present:

1. `ScopeRef`
2. `WorkstreamId`
3. `ContinuityId`
4. `AttachmentKey`
5. `SessionId`
6. `InstanceId`
7. `WorkSurfaceId`

The typed contract is implemented in `src/lib/shell/pi-attachment-contract.ts`. Project root, current tab, CWD, remembered workspace selection, latest session, and daemon-global state are not substitutes.

## Frontend surface

`src/lib/shell/AgentTuiSurface.svelte` provides:

- integrated Agent TUI identity and verified runtime status;
- horizontal Workstream/Continuity/Attachment/Work Surface authority strip;
- Transcript, Work Rail, Evidence, and Context views;
- terminal geometry reserved for the native PTY renderer;
- scoped steering composer;
- interrupt, resume, and detach lifecycle controls;
- truthful unbound, disconnected, and error presentation;
- system/full/reduced motion behavior;
- no sidebar.

All mutation controls are disabled in the unbound projection. The current transcript is explicitly an integration shell and does not pretend that a Pi process exists.

## Native bridge boundary

The future Tauri bridge accepts only typed `PiNativeCommand` envelopes:

- `attach` with exact identity and terminal geometry;
- `input` with exact `AttachmentKey`;
- `resize` with exact `AttachmentKey` and geometry;
- `interrupt` with exact `AttachmentKey`;
- `detach` with exact `AttachmentKey`.

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

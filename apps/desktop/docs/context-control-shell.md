# Context Control Shell

Status: truthful unbound/recovery shell browser-accepted; canonical connection remains blocked

`src/lib/shell/ContextControlPanel.svelte` presents the complete identity ladder required before Focusa Desktop can bind a Workstream or Pi runtime:

1. ScopeRef
2. WorkstreamId
3. ContinuityId
4. AttachmentKey
5. SessionId / InstanceId
6. WorkSurfaceId

The shell reads existing daemon infrastructure health but does not create authority. Its connection control remains disabled until the typed identity, ownership, and reducer contracts are available.

Forbidden fallback inputs:

- current tab;
- CWD or project-root inference;
- remembered workspace selection;
- latest record or session;
- daemon-global state.

Interaction contract:

- Context Control opens from the expanded or compact sidebar;
- focusable close control and Escape dismiss it;
- daemon status remains distinct from Workstream authority;
- blocked state explains the recovery boundary;
- no local form or visual selection mints identity.

Evidence:

- `docs/contracts/evidence/spec158-desktop-context-control-unbound.png`
- `uiai-diagnostics:session=mbZEfNhs:seq=103` with zero console errors, warnings, exceptions, and failed requests.

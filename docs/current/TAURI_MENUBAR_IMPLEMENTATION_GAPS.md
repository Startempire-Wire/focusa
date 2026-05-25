# Tauri Menubar Implementation Gaps

**Source spec:** [`TAURI_MENUBAR_UP_TO_SPEED_SPEC.md`](TAURI_MENUBAR_UP_TO_SPEED_SPEC.md)  
**Audit source:** [`TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md`](TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md)
**Last updated:** after `git:9de260c`.

## Implemented in the current menubar slice

- Shared API client now includes `postJson`, `requestJson`, result-envelope normalization, degradation detection, and shared error summarization.
- `SyncPanel` and `AddPeerModal` use the shared API base; only the default/placeholder URL remains hardcoded.
- `bun run check` now runs `svelte-kit sync` before `svelte-check`, so clean check no longer depends on a prior build.
- Menubar version metadata and visible About copy now use `0.9.12-dev`.
- Cockpit/Now surface polls current runtime hot surfaces: project identity, trajectory view, Workpoint resume, work-loop health/checkpoints, doctor, memory telemetry, prediction stats/recent, metacognition evaluations, snapshots, and lineage head.
- Cockpit cards show calm status chips for canonical/degraded/status/evidence/readiness/memory posture.
- Read-only peeks exist for Trajectory, Workpoint, Proof, and Work Loop.
- Default Focus view now restores original bubble/cloud spirit: current bubble, background clouds, ambient stack depth, and quieter empty states.
- Gate/Sync copy now frames signals and peers as ambient awareness, not focus-changing control UI.
- Static guard exists at `tests/spec96_menubar_cockpit_foundation_static_test.sh`.

## Remaining Phase 0 gaps — foundation

- `npm ci` still fails because npm/package-lock dependency resolution conflicts with Vite/Svelte plugin versions; Bun path is the validated path.
- Tauri config still uses `csp: null`; security posture needs a compatibility pass.
- Full Tauri packaging (`bun tauri build`) has not been proven in this slice.

## Remaining Phase 1 gaps — ambient cockpit MVP

- Navigation is visually calmer but still implemented as tabs; future work should evolve this into true peek drawers while keeping keyboard/accessibility behavior.
- Ambient menubar icon state is not yet driven by daemon/project/workpoint/work-loop readiness.
- Release proof remains a placeholder card instead of reading a real release/proof artifact state.

## Remaining Phase 2+ gaps — depth

- Trajectory and Workpoint peeks are read-only; assess/propose/checkpoint/drift/evidence-link flows are not exposed yet.
- Proof peek is read-only and summary-only; it does not yet support prediction evaluation or snapshot diff.
- Work-loop peek is read-only; preflighted controls are not exposed yet.
- Add Peer still uses a modal-style component; spec target is non-modal drawer/inline confirmation.
- Focus/Gate mutation flows remain minimal; suppress/pin/surface controls need confirmation/result-envelope display before expansion.

## Next recommended implementation slices

1. **Tauri packaging/security pass:** verify or document Bun-only install, test `bun tauri build` where platform dependencies allow, and evaluate a minimal CSP.
2. **True drawer shell:** keep the current peek labels but move secondary surfaces into a drawer/peek interaction model rather than dense tabs.
3. **Ambient tray state:** map daemon/project/workpoint/work-loop posture to the tray icon states from the original spec.
4. **Safe read-to-write affordances:** add confirmation-gated drift check and evidence link first; defer work-loop mutations until writer preflight UX is explicit.

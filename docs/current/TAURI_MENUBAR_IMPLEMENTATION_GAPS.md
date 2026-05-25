# Tauri Menubar Implementation Gaps

**Source spec:** [`TAURI_MENUBAR_UP_TO_SPEED_SPEC.md`](TAURI_MENUBAR_UP_TO_SPEED_SPEC.md)  
**Audit source:** [`TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md`](TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md)

## Phase 0 gaps — foundation

- Shared API client is too small: no `postJson`, no envelope normalization, no shared error summarizer.
- `SyncPanel` and `AddPeerModal` still hardcode `http://127.0.0.1:8787` instead of the configured API base.
- Clean `npm ci` fails because package-lock/npm dependency resolution conflicts with Vite/Svelte plugin versions; Bun path works.
- `bun run check` can fail on clean checkout before `.svelte-kit/tsconfig.json` exists.
- Menubar version is stale (`0.9.9`) compared with current public snapshot language (`v0.9.12-dev`).
- Tauri config uses `csp: null`; security posture needs a later compatibility pass.

## Phase 1 gaps — ambient cockpit MVP

- Current Mission panel does not fetch project identity, trajectory view, Workpoint resume, work-loop health, doctor, or memory telemetry.
- Workpoint card uses `/v1/workpoint/current`; canonical continuation UI needs `/v1/workpoint/resume`.
- Work-loop card uses summary status but not hot-path `/v1/work-loop/health` dispatch readiness.
- There is no normalized display for `status`, `canonical`, `degraded`, `failure_class`, `retry`, `evidence_refs`, or `next_tools`.
- UI is still tab-like; spec target is ambient bubble + progressive drawers.

## Phase 2+ gaps — depth

- Trajectory drawer absent.
- Workpoint drawer absent.
- Proof drawer absent: evidence, predictions, metacognition evaluations, lineage/snapshots.
- Work-loop advanced drawer absent: writer ownership, pause flags, checkpoints, preflighted controls.
- Focus/Gate still uses old tab language and Add Peer still uses modal UI; later work should convert to non-modal drawers.

## First implementation slice

1. Expand shared API client.
2. Move Sync/AddPeer to shared API base.
3. Add runtime store fields for project, trajectory, Workpoint resume, work-loop health, doctor, and memory telemetry.
4. Expand Mission panel into an initial Cockpit view while preserving calm card styling.
5. Add validation for build/check and no hardcoded API base outside defaults/docs.

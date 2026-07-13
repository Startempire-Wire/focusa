# Spec104 Pi singleton closure evidence — 2026-07-13

Scope: `apps/pi-extension` only, from `168e96ee`.

## Closed beads / Annex rows

- `focusa-fsrc.1` / `PI-01`: removed exported `S` singleton symbol and direct `S.` reads/writes from Pi extension source.
- `focusa-fsrc.2` / `PI-02`: root/identity shadows remain behind typed scoped accessors rather than the old singleton name.
- `focusa-fsrc.3` / `PI-03`: active Workpoint packet/summary are not fields in the runtime object; access goes through `TypedScopeStore` accessors.
- `focusa-fsrc.4` / `PI-04`: trajectory clarity is scope-store-backed and restore paths check root/session/workstream before adoption.
- `focusa-fsrc.5` / `PI-05`: project identity/verify shadows are scope-store-backed.
- `focusa-fsrc.6` / `PI-08`: tool/action bridge imports `runtimeState` plus typed scoped helpers; no `S` import/access remains. `focusa_session_transfer` now carries explicit `source_scope`, `target_scope`/`target_continuity_id`, source/target session ids, packet refs, and rollover action for Spec130 rotating continuity without fingerprint-derived continuity.
- `focusa-fsrc.7` / `PI-09`: turn authority bridge has no direct `S` access.
- `focusa-fsrc.8` / `PI-10`: commands/prompt bridge has no direct `S` access and rollover/context paths build explicit `WorkstreamKey` objects.
- `focusa-fsrc.9` / `PI-12`: persisted continuity restore remains root/session/workstream-checked and no longer depends on `S` direct authority.
- `focusa-fsrc.10` / `PI-13`: frame recovery/compaction paths use typed packet scope and blocked mismatch behavior.

## Code changes

- Removed the rejected `runtimeState` export/name as well as `export const S`; mutable attachment runtime is now created per typed `AttachmentKey` by `AttachmentRuntimeRegistry`, with `AsyncLocalStorage<AttachmentKey>` binding at Pi event/tool/command/shortcut entrypoints. `getAttachmentRuntime()` requires an explicit key or bound attachment key and throws when absent; no module-global one-slot attachment runtime object remains.
- Kept Workpoint, Trajectory, identity, report summary, turn, and tool batch authority behind `TypedScopeStore` accessors.
- Fixed compaction resume packet refresh to construct a local `WorkstreamKey` once and pass that explicit `scope` through the request body.
- Added `tests/spec104-pi-runtime-isolation.test.mjs` to statically/runtime-model assert:
  - no `export const S`, `S.` access, or `S` imports remain in Pi extension source;
  - active Workpoint, trajectory, and identity authority are absent from the mutable runtime object;
  - `ScopeRef`, `WorkstreamKey`, `AttachmentKey`, and CRDT reconciliation contracts remain present;
  - compaction carries typed `WorkstreamKey` scope;
  - `focusa_session_transfer` declares and forwards explicit source/target scope, target continuity, session ids, packet refs, and rollover action, and does not call `ensureContinuityId`;
  - `/focusa-rollover execute` lifecycle includes `ctx.waitForIdle()`, checkpoint/trajectory/compaction packet preparation, typed transfer save, bounded source migration/seal, `ctx.newSession({ parentSession, setup })` target bootstrap injection, target Workpoint/resume verification, and verify-target receipt POST;
  - `focusaFetchDetailed` adds API ScopeContext-compatible typed scope headers (`x-scope-project-root`, `x-scope-continuity-id`, `x-scope-session-id`, `x-scope-id`, `x-scope-kind`, `x-scope-attachment-id`) from the bound `AttachmentKey` only.
- Updated `config/spec104-scoped-state-inventory.json` with Pi Annex C/D/E closure evidence rows.

## Proof run

From `apps/pi-extension`:

```text
npm run typecheck
npm run lint
npm run test:spec104
npm run test:spec104-attachment
npm run test:spec130-rollover
npm run format
```

All passed. No Rust/cargo, release, tag, deploy, push, or remote commands were run.

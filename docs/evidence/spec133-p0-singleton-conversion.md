# Spec 133 pre-MVP singleton conversion evidence

Date: 2026-07-13

Scope: `apps/pi-extension/src/state.ts`

## Changed

- Removed `S` singleton storage for active Workpoint packet/summary.
- Removed `S` singleton storage for trajectory clarity shadow.
- Removed `S` singleton storage for project identity and project verify shadows.
- Converted PI-02/PI-03/PI-04/PI-05 accessors to read/write the typed `TypedScopeStore` only, with no `S` fallback.
- Replaced remaining direct `S.lastTrajectoryClarity` and `S.lastProjectIdentity` uses with typed accessors.

## Static proof

```text
python3 singleton ref scan:
activeWorkpointPacket S_refs []
activeWorkpointSummary S_refs []
lastTrajectoryClarity S_refs []
lastProjectIdentity S_refs []
lastProjectVerify S_refs []
```

## Blocked proof

`npm --prefix apps/pi-extension run check` could not execute because `tsc` is not installed in this worktree.

`npm --prefix apps/pi-extension run lint` could not execute because `@typescript-eslint/parser` is not installed in this worktree.

No cargo build/check/test, release build, tags, deploys, pushes, or remotes were run.

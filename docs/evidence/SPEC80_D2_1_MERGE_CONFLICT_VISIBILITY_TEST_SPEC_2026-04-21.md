# SPEC80 D2.1 — Merge Conflict Visibility Test Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.2.1`
Label: `documented-authority`

Purpose: verify `restore_mode=merge` behavior always surfaces explicit conflicts and never silently overwrites state.

## Scenario ID

`gateC.merge_conflict_visibility.v1`

## Preconditions

1. Base branch node `N0` snapshot `S0` exists.
2. Branch `B1` and `B2` diverge from `N0` with conflicting updates to same semantic fields.
3. Restore/diff interfaces available (or deterministic harness stubs until endpoint implementation).

## Replay sequence

1. On `B1`, set constraint `C = value_A`; snapshot `SB1`.
2. On `B2`, set constraint `C = value_B`; snapshot `SB2`.
3. From `B1` context, run restore with `restore_mode=merge` targeting `SB2`.
4. Capture restore response and resulting state.

## Assertions

1. Response includes non-empty `conflicts[]` for each conflicting field.
2. Each conflict entry includes `field`, `base_value`, `incoming_value`, `resolution`.
3. No conflicting field is silently replaced without conflict record.
4. Non-conflicting fields merge deterministically.
5. Mutation audit includes explicit restore command/tool/event path.

## Failure signatures

- `MISSING_CONFLICT_ENTRY`: conflict exists but `conflicts[]` omits it.
- `SILENT_OVERWRITE`: conflicting value changed without conflict record.
- `NON_DETERMINISTIC_MERGE`: repeated merge yields divergent outputs.

## Required evidence outputs

- pre-merge branch snapshots (`SB1`, `SB2`) checksums
- restore response payload with `conflicts[]`
- post-merge state digest
- mutation event log demonstrating explicit merge operation

## Gate linkage

Contributes to Gate C scenario 3 (`Appendix D`) and `§20.3` criterion 1.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17 scenario 3, §10 Gate C)
- docs/17-context-lineage-tree.md

# SPEC80 D1.2 — Tree Navigation Restore Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.1.2`
Purpose: specify deterministic replay validation for Appendix D scenario 17.2 (tree navigation restore checksum stability).

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17.2, §17 Gate C)
- docs/79-focusa-governed-continuous-work-loop.md

## Scenario definition (Appendix D §17.2)

Test narrative:
1. Prepare two branches `B1` and `B2` with known snapshot anchors.
2. Navigate `B1 -> B2 -> B1` repeatedly for N cycles.
3. On each restore, compute canonical state checksum.
4. Compare checksum to branch baseline snapshot checksum.
5. Assert each branch restores to its own canonical checksum every cycle.

## Deterministic replay inputs

Required fixtures:
- fixed seed/session id
- branch ids `B1`, `B2`
- baseline snapshot ids for both branches
- cycle count `N` (minimum 3)

Replay event requirements:
- explicit `branch_navigate` events with source/target branch ids
- explicit `restore_applied` events with checksum payloads
- explicit replay transcript id for sequence reproducibility

## Invariants

Hard invariants:
1. **Per-branch checksum stability**: each restore to `B1` equals `B1` baseline checksum; same for `B2`.
2. **Cycle invariance**: checksums are identical across all cycles for a given branch.
3. **No cross-branch bleed**: restoring to `B1` never yields `B2` checksum and vice versa.

Failure conditions:
- any cycle checksum mismatch versus baseline.
- checksum drift across cycles on same branch.
- missing navigation/restore event evidence in replay log.

## Pass criteria

Scenario passes only when all cycles for both branches satisfy baseline checksum equality with zero drift.

Gate linkage:
- contributes to Spec80 Gate C branch correctness (`all Appendix D scenarios pass with stable checksums and zero silent mutation events`).

## Evidence outputs expected from implementation lane

- baseline checksums for `B1` and `B2`
- per-cycle checksum table
- replay transcript id
- mismatch count (`must be 0`)

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md

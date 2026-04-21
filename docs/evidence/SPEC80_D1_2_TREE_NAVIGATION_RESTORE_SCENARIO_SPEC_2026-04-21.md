# SPEC80 D1.2 — Tree Navigation Restore Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.1.2`
Label: `documented-authority`

Purpose: define deterministic replay scenario proving repeated `/tree` navigation restores branch-correct state checksums.

## Scenario ID

`gateC.tree_navigation_restore.v1`

## Preconditions

1. Two descendant branches from common ancestor: `B1`, `B2`.
2. Saved snapshot per branch head: `SB1`, `SB2`.
3. Distinct state markers per branch (decisions/constraints).

## Replay sequence

1. Restore `B1` from `SB1`; record checksum `C1a`.
2. Navigate to `B2`; restore from `SB2`; record checksum `C2a`.
3. Navigate back to `B1`; restore from `SB1`; record checksum `C1b`.
4. Navigate to `B2`; restore from `SB2`; record checksum `C2b`.
5. Repeat sequence for N cycles (minimum N=5).

## Assertions

1. `C1a == C1b == C1...` across all B1 cycles.
2. `C2a == C2b == C2...` across all B2 cycles.
3. B1 markers never appear in B2 and vice versa.
4. No hidden writes or silent mutation events during navigation.

## Failure signatures

- `RESTORE_DRIFT_B1` / `RESTORE_DRIFT_B2`: checksum drift across repeated restores.
- `CROSS_BRANCH_CONTAMINATION`: marker from sibling branch appears.
- `SILENT_MUTATION_EVENT`: non-attributed mutation event detected.

## Required evidence outputs

- cycle-by-cycle checksum table
- branch marker presence matrix
- replay operation log
- mutation audit report

## Gate linkage

Contributes to Gate C (`Appendix D`) and utilization criterion 1 (`§20.3`).

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17, §10 Gate C)
- docs/17-context-lineage-tree.md

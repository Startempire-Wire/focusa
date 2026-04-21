# SPEC80 D4.1 — Compaction Survival Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.4.1`
Purpose: specify deterministic validation for Appendix D §17.4 to ensure compaction preserves snapshot-restore equivalence.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17.4, §17 Gate C)
- docs/79-focusa-governed-continuous-work-loop.md

## Scenario definition (Appendix D §17.4)

Test narrative:
1. Create branch snapshot `S0` with canonical decision/constraint sets.
2. Execute compaction over branch artifacts.
3. Restore branch state from snapshot `S0` post-compaction.
4. Compare restored canonical sets with pre-compaction baseline.
5. Assert restored decision and constraint sets are identical.

## Deterministic replay inputs

Required fixtures:
- fixed seed/session id
- snapshot id `S0`
- baseline canonical decision set hash
- baseline canonical constraint set hash

Replay event requirements:
- explicit `snapshot_created` event before compaction
- explicit `compaction_executed` event with artifact counts
- explicit `snapshot_restored` event after compaction
- replay transcript id for reproducibility

## Invariants

Hard invariants:
1. **Decision-set equivalence**: restored decision-set hash equals baseline decision-set hash.
2. **Constraint-set equivalence**: restored constraint-set hash equals baseline constraint-set hash.
3. **No compaction loss**: compaction must not drop branch-keyed artifacts needed for restore.

Failure conditions:
- decision or constraint hash mismatch after restore.
- missing snapshot/compaction/restore event evidence.
- artifact count regression without explicit conflict/error disclosure.

## Pass criteria

Scenario passes only when post-compaction restore reproduces canonical baseline sets with zero mismatch.

Gate linkage:
- contributes to Spec80 Gate C (`stable checksums` and `zero silent mutation events`) by proving compaction-safe branch restore.

## Evidence outputs expected from implementation lane

- baseline decision/constraint hashes
- post-restore decision/constraint hashes
- artifact-count pre/post compaction summary
- replay transcript id
- mismatch count (`must be 0`)

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_RESTORE_COMPACTION_PERFORMANCE_BUDGETS_2026-04-21.md
- docs/evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md

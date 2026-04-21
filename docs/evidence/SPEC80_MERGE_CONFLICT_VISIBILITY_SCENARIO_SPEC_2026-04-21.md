# SPEC80 D2.1 — Merge Conflict Visibility Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.2.1`
Purpose: specify deterministic validation for Appendix D scenario 17.3 to guarantee explicit `conflicts[]` reporting and forbid silent overwrite during merge restore.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17.3, §17 Gate C)
- docs/79-focusa-governed-continuous-work-loop.md

## Scenario definition (Appendix D §17.3)

Test narrative:
1. Prepare baseline snapshot for branch `B1` with constraint set `C_base`.
2. Create divergent branch `B2` with conflicting constraint updates `C_conflict`.
3. Invoke restore on target branch using `restore_mode=merge`.
4. Capture restore response payload and post-restore state.
5. Assert response includes explicit `conflicts[]` entries for each detected conflict.
6. Assert no conflicting field is silently overwritten without conflict disclosure.

## Deterministic replay inputs

Required fixtures:
- fixed seed/session id
- branch ids `B1`, `B2`
- snapshot ids for pre-merge state
- deterministic conflicting keys list

Replay event requirements:
- explicit `snapshot_created` and `restore_requested` events
- restore event payload includes mode (`merge`) and conflict metadata
- replay transcript id for reproducibility

## Invariants

Hard invariants:
1. **Conflict visibility**: merge restore response must contain non-empty `conflicts[]` when conflicts exist.
2. **No silent overwrite**: every changed conflicting field must map to a corresponding conflict entry.
3. **Mode fidelity**: response must indicate `restore_mode=merge` semantics were applied.

Failure conditions:
- `conflicts[]` missing or empty when conflicting updates are present.
- post-restore state differs from baseline/conflict policy without reported conflict.
- merge-mode restore path emits exact-mode behavior without disclosure.

## Pass criteria

Scenario passes only when all conflict keys are explicitly disclosed and overwrite behavior is fully auditable.

Gate linkage:
- contributes to Spec80 Gate C (`stable checksums` + `zero silent mutation events`) by enforcing explicit conflict surfacing.

## Evidence outputs expected from implementation lane

- restore response payload snapshot containing `conflicts[]`
- conflict key coverage map (`expected_conflicts` vs `reported_conflicts`)
- replay transcript id
- silent-overwrite count (`must be 0`)

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md

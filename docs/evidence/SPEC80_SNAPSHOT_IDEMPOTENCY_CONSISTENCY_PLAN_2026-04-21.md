# SPEC80 D2.2 — Snapshot API Idempotency + Consistency Plan

Date: 2026-04-21
Bead: `focusa-yro7.4.2.2`
Purpose: define idempotency and consistency checks for snapshot lifecycle semantics under Spec80 Epic D.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17, §20.2)
- docs/79-focusa-governed-continuous-work-loop.md

## Lifecycle surface under validation

Planned snapshot lifecycle endpoints:
- `POST /v1/focus/snapshots`
- `POST /v1/focus/snapshots/restore`
- `POST /v1/focus/snapshots/diff`

## Idempotency requirements

1. **Create idempotency (same request fingerprint)**
   - repeated identical create request must not create divergent logical state.
   - allowed outcomes:
     - return same snapshot identity, or
     - return new snapshot id with identical checksum + explicit idempotent metadata.

2. **Restore idempotency (same target + mode + snapshot)**
   - repeated restore request must converge to same post-restore checksum.

3. **Diff idempotency (same operands)**
   - repeated diff request returns same deltas and ordering guarantees for canonical keys.

## Consistency requirements

1. **Checksum consistency**
   - snapshot checksum equals canonical state hash at creation point.
2. **Reference consistency**
   - restore references valid snapshot id and branch lineage anchor.
3. **Temporal consistency**
   - restore/diff against missing or stale snapshot must return typed error (no silent fallback).
4. **Conflict consistency**
   - merge restores must produce deterministic conflict set for fixed inputs.

## Typed error expectations

Required typed errors for lifecycle consistency failures:
- `SNAPSHOT_NOT_FOUND`
- `SNAPSHOT_CONFLICT`
- `RESTORE_CONFLICT`
- `DIFF_INPUT_INVALID`

## Test lanes

| Lane | Scenario | Required assertion |
|---|---|---|
| I1 | create replayed with same payload | idempotent state outcome + stable checksum |
| I2 | restore replayed same target/mode | post-restore checksum identical |
| I3 | diff replayed same operands | identical delta payload |
| C1 | restore references missing snapshot | typed `SNAPSHOT_NOT_FOUND` |
| C2 | merge conflict deterministic replay | identical `conflicts[]` across runs |

## Gate linkage

- Supports Appendix D branch-correctness semantics by preventing nondeterministic snapshot lifecycle behavior.
- Supports §20.2 performance/robustness lane by enabling deterministic replay baselines before latency tuning.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md
- docs/evidence/SPEC80_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md

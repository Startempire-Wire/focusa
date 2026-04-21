# SPEC80 D2.2 — Snapshot API Idempotency and Consistency Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.2.2`
Label: `planned-extension`

Purpose: define idempotency and consistency checks for snapshot create/restore/diff lifecycle.

## Scenario ID

`gateC.snapshot_idempotency_consistency.v1`

## Preconditions

1. Snapshot API surfaces planned: `/v1/focus/snapshots`, `/restore`, `/diff`.
2. Stable lineage fixture with known branch graph and deterministic initial state.

## Test groups

### Group A — Create idempotency

1. Send duplicate snapshot-create requests with same `idempotency_key`.
2. Assert same `snapshot_id` returned (or explicit duplicate reference).
3. Assert no duplicate state object writes.

### Group B — Exact restore consistency

1. Restore same snapshot repeatedly with `restore_mode=exact`.
2. Assert checksum equality across all restores.
3. Assert no drift in canonical decision/constraint/failure sets.

### Group C — Diff determinism

1. Run diff on fixed pair `(from,to)` multiple times.
2. Assert identical deltas and ordering for repeated runs.
3. Assert `diff(a,b)` and `diff(b,a)` remain directionally consistent (inversion rule documented).

### Group D — Cross-operation coherence

1. Create snapshot -> restore -> diff back to baseline.
2. Assert resulting diff is empty for unchanged state.
3. Assert operation metadata chain is complete and auditable.

## Failure signatures

- `IDEMPOTENCY_KEY_BROKEN`
- `EXACT_RESTORE_DRIFT`
- `DIFF_NON_DETERMINISTIC`
- `AUDIT_CHAIN_GAP`

## Required evidence outputs

- request/response logs with idempotency keys
- repeated-run checksum table
- diff snapshot outputs and ordering checks
- operation audit chain report

## Gate linkage

Supports Gate C scenario integrity and `§20.2` performance/coherence prerequisites.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§14.1, §15, §17)
- docs/17-context-lineage-tree.md

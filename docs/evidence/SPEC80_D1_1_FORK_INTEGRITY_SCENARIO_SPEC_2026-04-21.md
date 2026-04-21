# SPEC80 D1.1 — Fork Integrity Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.1.1`
Label: `documented-authority`

Purpose: define deterministic replay scenario proving post-fork mutations do not leak into pre-fork restore state.

## Scenario ID

`gateC.fork_integrity.v1`

## Preconditions

1. Lineage route surface available (`/v1/lineage/*`).
2. Snapshot routes planned (`/v1/focus/snapshots*`) and represented by deterministic test harness shims until implemented.
3. Clean test workspace with isolated session + CLT lineage seed.

## Replay sequence

1. Create frame `A` at node `N0`.
2. Record decision `D1` on `N0`.
3. Create snapshot `S0` bound to `N0`.
4. Fork to branch node `N1`.
5. On `N1`, record decision `D2`.
6. Restore branch context to `N0` using `restore_mode=exact` from `S0`.

## Assertions

1. `D1` present after restore.
2. `D2` absent after restore.
3. Restored checksum equals snapshot checksum for `S0`.
4. No silent mutation events in replay log.

## Failure signatures

- `BRANCH_LEAK_DETECTED`: fork-only artifact appears in restored pre-fork branch.
- `CHECKSUM_MISMATCH`: exact restore checksum diverges from snapshot checksum.
- `SILENT_MUTATION_EVENT`: mutation occurred without explicit tool/command/event trace.

## Required evidence outputs

- replay transcript (ordered operations)
- pre/post restore state digest
- checksum comparison record
- mutation event audit list (must be empty)

## Gate linkage

Contributes to Gate C (`Appendix D`) and full utilization criterion 1 (`§20.3`).

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17, §10 Gate C)
- docs/17-context-lineage-tree.md

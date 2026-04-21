# SPEC80 D4.1 — Compaction Survival Test Pack Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.4.1`
Label: `documented-authority`

Purpose: verify compaction preserves branch-keyed snapshot restore equivalence.

## Scenario ID

`gateC.compaction_survival.v1`

## Preconditions

1. Multi-branch lineage with snapshots on each active branch.
2. Canonical state digest function for decisions/constraints/failures/open questions.

## Test sequence

1. Capture per-branch pre-compaction digests + snapshot checksums.
2. Execute compaction.
3. Restore each branch from its snapshot.
4. Capture post-compaction digests + checksums.
5. Compare pre/post equivalence.

## Assertions

1. For each branch, restored digest is identical pre vs post compaction.
2. Snapshot checksum identity preserved for exact restore mode.
3. No branch artifact loss in decision/constraint sets.
4. No cross-branch contamination introduced by compaction.

## Failure signatures

- `COMPACTION_EQUIVALENCE_FAIL`
- `SNAPSHOT_CHECKSUM_REGRESSION`
- `BRANCH_ARTIFACT_LOSS`

## Evidence outputs

- branch digest table (pre/post)
- checksum comparison table
- compaction operation log

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17 scenario 4, §9 invariant 4)
- docs/17-context-lineage-tree.md

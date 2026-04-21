# SPEC80 D1.1 — Fork Integrity Scenario Spec

Date: 2026-04-21
Bead: `focusa-yro7.4.1.1`
Purpose: specify deterministic replay validation for Appendix D scenario 17.1 (fork snapshot integrity).

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§17.1, §17 Gate C)
- docs/79-focusa-governed-continuous-work-loop.md

## Scenario definition (Appendix D §17.1)

Test narrative:
1. Start on branch A.
2. Record decision `D1` on branch A.
3. Fork at checkpoint `T` to branch B.
4. Record decision `D2` on branch B only.
5. Restore branch A state at pre-fork lineage.
6. Assert `D2` is absent from restored A state.

## Deterministic replay inputs

Required fixtures:
- deterministic seed/session id
- canonical branch ids (`A`, `B`)
- checkpoint id `T`
- decision payload fingerprints for `D1` and `D2`

Replay event requirements:
- explicit `fork_created` event with parent lineage id
- explicit state snapshot artifact at `T`
- explicit restore event targeting branch A

## Invariants

Hard invariants:
1. **Branch isolation**: `D2` must not appear in restored branch A.
2. **Snapshot identity**: restored A checksum equals checksum captured before fork mutation on B.
3. **No silent merge**: any conflict must be explicit; absence of conflict must not imply hidden overwrite.

Failure conditions:
- `D2` present in restored A decisions.
- checksum mismatch between pre-fork A snapshot and restored A snapshot.
- missing fork/snapshot/restore event in replay log.

## Pass criteria

Scenario passes only if all invariants hold in one deterministic run and one replay run over identical fixture seed.

Gate linkage:
- contributes to Spec80 Gate C branch correctness (`all Appendix D scenarios pass with stable checksums and zero silent mutation events`).

## Evidence outputs expected from implementation lane

- replay transcript handle (trace/ref id)
- pre-fork snapshot checksum
- post-restore checksum
- decision membership proof (`contains D1`, `not contains D2`)

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_ENDPOINT_FALLBACK_BINDING_MATRIX_2026-04-21.md

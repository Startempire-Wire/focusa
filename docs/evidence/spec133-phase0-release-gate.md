# Spec 133 Phase 0 release/deploy dependency proof

Date: 2026-07-18

## Release/deploy freeze

Spec 133 remains local on `local/work-loop-completion`. No Spec 133 tag, release, deploy, live sync, push, merge, or canonical release artifact was created. The branch HEAD has no tag and is 17 local commits ahead of the locally known `origin/main` reference.

Build and test commands are permitted for local proof after Phase 0; they do not constitute release authority.

## Canonical release gate

Spec 133 implementation must not be released until:

1. Spec 132 final proof is complete and closed.
2. Spec 133 Phase 0 gate is closed with baseline, legacy freeze, traceability, and dependency proof.
3. Phase 1+ implementation proceeds in dependency order.
4. Every Spec 133 normative MUST is verified by the fail-closed Work Loop conformance gate.
5. The canonical signed release pipeline is used only after project policy permits release.

The tag workflow now invokes:

```text
python3 scripts/work_loop_conformance.py --mode release
```

That command currently returns exit 3 because Spec 133 implementation coverage is incomplete.

## Prior blocker resolved

The prior Spec 132 blocker is closed:

- `focusa-slxpz.6.6` — `132 final gate: every binding requirement proven` — `closed`

Phase 0 may therefore close after its static proof and ledger updates pass. This does **not** authorize Phase 1+ release; it only unblocks dependency-ordered implementation.

## Dependency chain

- Spec132 final proof: `focusa-slxpz.6.6` — closed.
- Spec133 Phase 0 tasks: `focusa-a6yq6.1.1` → `.1.2` → `.1.3` → `.1.4`.
- Spec133 Phase 0 gate: `focusa-a6yq6.1.5`.
- Spec133 Phase 1 starts only after `.1.5` is truly closed.
- Work Loop scheduler integration `.7` remains downstream of Spec133 Phase 1–4 and `focusa-a6yq6.6.3`.

## Evidence commands

```bash
bd --no-db show focusa-slxpz.6.6 --json
bash tests/spec133_phase0_static_test.sh
python3 tests/work_loop_conformance_manifest_test.py
python3 scripts/work_loop_conformance.py --mode release  # expected exit 3 until all MUSTs are verified
git branch --show-current
git tag --points-at HEAD
git rev-list --left-right --count origin/main...HEAD
```

# Spec 133 Phase 0 release/deploy dependency proof

Updated: 2026-07-17
Bead: `focusa-a6yq6.1.4`

## Authorization chronology

The original Phase 0 slice ran under a strict release/deploy freeze and produced only local baseline, legacy-wrapper, and traceability evidence.

Afterward, the operator explicitly directed completion of the pre-MVP OTA and Spec132 gates. The repository was fully linted and tested locally before publication. Spec132 closed, then the canonical release workflow published `v0.9.120-dev` and its authorized deploy completed:

- local rustfmt, Clippy `-D warnings`, workspace tests, Svelte and Pi checks: PASS;
- final CI: `29551035631`, PASS;
- Spec132 platform/release-target matrix: `29551308143`, PASS;
- signed release: `29551308132`, PASS, 58 assets;
- deploy: `29552019926`, PASS;
- live Bash/PowerShell parity deploy: `29552998591`, PASS.

That release closed OTA and Spec132 work. It did **not** claim Spec133 completion or authorize bypassing the remaining Spec133 dependency chain.

## Canonical Spec133 release gate

Any later release claiming Spec133/MVP readiness requires:

1. Phase 0 baseline, legacy freeze, traceability, and this authorization record closed.
2. Phases 1–9 implemented and closed in dependency order.
3. The §32 acceptance criteria and §33 gap matrix proven at the exact candidate commit.
4. Full local lint/tests before the first push and one bounded cross-platform acceptance sequence.
5. Explicit operator authorization and the canonical signed release/deploy pipelines.

## Dependency chain

- Spec132 final proof: closed (`focusa-slxpz`).
- Spec133 Phase 0 tasks: `.1.1` → `.1.2` → `.1.3` → `.1.4`.
- Spec133 Phase 0 gate: `focusa-a6yq6.1.5`.
- Phase 1 starts only after `.1.5` closes.
- No Phase 8 or Phase 9 task may bypass the earlier phase gates.

## Result

No unauthorized release/deploy occurred. The previously recorded Spec132 blocker is closed, authorization provenance is explicit, and the remaining Spec133 release freeze is bound to its full dependency and acceptance chain.

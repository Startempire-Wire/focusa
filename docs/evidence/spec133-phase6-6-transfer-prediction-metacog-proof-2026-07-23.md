# Spec 133 Phase 6.6 — Session Transfer, prediction, and metacognition

Date: 2026-07-23
Bead: `focusa-a6yq6.7.6`
Scope: Spec 133 §19.13, §19.16, §19.17

## Existing Session Transfer integration

`SilentSessionTransferProjection` routes pause, orphan recovery, handoff, model switch, and foreground takeover through an explicit existing Session Transfer ref. It binds session/run/generation, ProjectIdentity, Continuity, Workpoint, runtime checkpoint, and event cursor.

- handoff and foreground takeover require both writer-handoff and operator-authorization refs;
- model switch requires distinct source and target model refs;
- every projection reports `persist_via_existing_session_transfer = true`;
- no Silent Session transfer store or competing writer authority is introduced.

## Prediction ordering

`bind_prediction_before_action` reuses an existing prediction ref and requires:

- one of the required uncertain-action classes: model fallback, broad refactor, flaky-test repair, dependency upgrade, risky integration, or recovery strategy;
- exact session/run scope;
- prediction event sequence strictly before the planned action sequence;
- bounded outcome, confidence, recommendation, and rationale;
- no pre-filled actual outcome/evaluation.

`bind_prediction_evaluation` accepts only the exact planned action event, a later evaluation event, a score in `[0,1]`, actual outcome, and evidence refs.

## Evidence-backed metacognition

`prepare_learning_candidate` accepts only an evaluated prediction and a `completed` or `failed` session outcome. It requires content, rationale, and evidence refs, deduplicates evidence, and emits:

- `capture_via_existing_metacog = true`;
- `advisory_only = true`;
- `governance_authority = false`.

The candidate therefore cannot alter Workpoint, Work Loop, closure, or operator authority.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_continuation.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_continuation -- --nocapture
cargo test -p focusa-core prediction -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove takeover/handoff authority requirements, model-switch binding, existing-transfer routing, prediction-before-action ordering, exact action evaluation, evidence requirements, completed/failed learning candidates, existing-metacog routing, and advisory non-authority.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

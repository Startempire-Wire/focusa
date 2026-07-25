# Spec 133 Phase 4.6 — failure/recovery envelope proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.6`
Scope: Spec 133 §28

## Implemented contract

`focusa.silent_session_failure.v1` is the closed canonical failure envelope for all 42 required Spec 133 classes, from `scope_mismatch` through `retention_blocked_by_hold`.

Every constructed envelope requires and exposes:

- the typed failure class;
- a concrete nonempty `why`;
- current canonical lifecycle state;
- canonical posture (`intact`, `degraded`, or `blocked`);
- runtime posture (`healthy`, `degraded`, `blocked`, `waiting_input`, `paused`, `stopped`, or `unavailable`);
- retry posture (`safe_after_recovery`, `safe_with_fresh_approval`, `safe_with_new_run_generation`, `wait_for_operator`, `exhausted`, or `not_retryable`);
- typed side effects already performed, with optional artifact refs;
- exact recovery tools;
- whether operator action is required.

A deterministic exhaustive match maps every class; no default/unknown branch can fabricate policy. Empty why, missing recovery tools, invalid schema, or untyped side effects fail validation.

Representative truth rules include:

- authority/scope/config/evidence failures block canonical progress;
- runner/process/transport failures preserve canonical state while reporting runtime loss;
- waiting input requires operator action and unchanged retry is unsafe;
- retry exhaustion remains exhausted rather than restarting another budget;
- process/orphan failures require a new run generation;
- unsupported capability/protocol failures are not retryable;
- retention holds preserve healthy runtime but block destructive retention action.

## Proof

The exhaustive test asserts exactly 42 unique classes, constructs and validates every envelope, serializes every class name, and confirms recovery tools are present. Focused tests verify retry exhaustion, waiting input, and invalid side-effect rejection.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 395 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

API/CLI surfaces may project this canonical envelope, but may not weaken or replace its taxonomy. The combined Phase 4 gate remains `.5.7` and must prove all Phase 4 slices together.

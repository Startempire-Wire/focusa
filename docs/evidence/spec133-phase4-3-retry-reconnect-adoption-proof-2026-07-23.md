# Spec 133 Phase 4.3 — retry, reconnect, and orphan-adoption proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.3`
Scope: Spec 133 §13.6–§13.7

## Typed retry budgets

Silent Session supervision now carries seven explicit, independently validated retry classes:

- provider;
- transport reconnect;
- harness restart;
- tool/environment recovery;
- model fallback;
- runner reconnect;
- work-item retry.

`focusa.silent_session_retry_budget.v1` stores one policy and state per class. Each class has its own retry count, base backoff, bounded exponential maximum, and next-retry timestamp. Success resets only the selected class. Missing classes, zero retries, zero backoff, invalid maxima, timestamp overflow, and counter overflow fail closed.

Backward deserialization uses a complete typed default set. New and revised configs are validated so legacy aggregate restart/transport fields cannot substitute for missing typed classes.

Runtime proof exhausts runner-reconnect retries through the bounded 250 ms → 10 s backoff sequence while provider state remains untouched, then proves class-local reset.

## Runner reconnect and orphan adoption

The existing adoption barrier already compared all authority-bearing fields:

- authenticated runner and protocol identity;
- session ID, run ID, and generation;
- project root and project identity reference;
- workspace root;
- OS user and UID;
- executable and launch-manifest SHA-256;
- process-instance identity;
- heartbeat freshness.

This slice adds typed reconciliation:

- daemon-to-runner orphan reconciliation queries are signed and exactly addressed;
- a stream cursor is restored only when adoption is accepted and carries a signed runner-record reference;
- accepted runs transition to the explicit `recovering` reconciliation posture;
- mismatched or unknown processes remain `orphaned` and receive no restored cursor;
- malformed/empty-cursor or cross-run decisions fail as invalid protocol frames.

The POSIX supervisor and direct backend expose this reconciliation without accepting unknown processes or bypassing the existing live process-tree validation.

## Commands and results

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 388 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-session-runner
```

Result: 31 unit tests and 1 protected-runner E2E passed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-harness-adapters
```

Result: 6 adapter-contract and 5 model-safety tests passed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core -p focusa-session-runner \
    -p focusa-harness-adapters --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

This slice does not claim machine-reboot survival or relaunch policy; those remain `.5.4`. Resource admission/limits and complete failure envelopes remain `.5.5`–`.5.7`.

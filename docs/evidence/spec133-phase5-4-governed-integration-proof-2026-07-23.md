# Spec 133 Phase 5.4 — governed integration proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.4`
Scope: Spec 133 §18.7

## Required flow

`focusa.integration_preflight.v1` blocks execution until all eight gates are present:

1. tests passed;
2. verification evidence;
3. final Workpoint checkpoint;
4. diff evidence;
5. commit evidence;
6. integration preview;
7. Context Authority preflight;
8. target writer lease and fencing token.

The authorized preflight hashes the exact session/run/generation, source/target workspace and revisions, method, evidence, authority, lease, and unrelated dirty paths into an action digest. Execution observations must match that digest.

Supported governed methods are explicit: verified fast-forward, governed merge, governed rebase, and governed cherry-pick. The protocol does not execute git or silently select a method.

## Conflict and dirty-state law

- any conflict yields `blocked_conflict` and no integration receipt;
- conflicts never trigger cleanup;
- any reported destructive cleanup is rejected;
- every unrelated dirty path from preflight must remain in the execution observation;
- missing preserved dirty state fails the outcome rather than claiming success.

## Receipt

A conflict-free observation requires a valid resulting revision and emits `focusa.integration_receipt.v1` containing session/run/generation, method, source/target/result revisions, Workpoint/diff/commit/preview evidence, Context Authority, lease/fencing, executed-command reference, and preserved dirty paths.

## Proof

Focused tests prove all eight missing gates, conflict blocking without cleanup, dirty-state loss rejection, successful complete receipt, and destructive-cleanup rejection.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 408 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

Foreground/background multi-session isolation and the combined Phase 5 gate remain `.6.5`–`.6.6`. Official integration execution remains governed by operator/VPS authority.

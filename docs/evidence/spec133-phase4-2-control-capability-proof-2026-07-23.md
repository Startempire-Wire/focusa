# Spec 133 Phase 4.2 — control and capability truth proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.2`
Scope: Spec 133 §13.4 and control/input portions of the backend and harness capability contracts

## Implemented contract

### Portable soft pause

The canonical daemon pause/resume routes remain portable lifecycle controls. They reduce through the canonical state machine, require approval and exact expected generation, persist with CAS, and reject stale generation. Soft pause does not pretend to suspend an operating-system process tree.

### POSIX hard pause

The direct POSIX backend now declares `HardPause = Native` and exposes typed hard-pause/hard-resume operations.

Before `SIGSTOP` or `SIGCONT`, the supervisor verifies:

- exact owned run ID;
- exact run generation;
- live child status;
- unchanged process-instance, process-group, and OS-session identity.

The signal targets the complete owned process group, never only the leader PID. Every successful operation returns `focusa.process_control_report.v1` with session/run/generation, process-instance ID, process-group ID, action, and observation time. A stale generation returns `RunGenerationMismatch` without signaling the process.

### Input and key capability truth

The Pi RPC adapter continues to provide deterministic prompt, steering, and follow-up delivery using an exact `RunRef { run_id, generation }`. Zero/stale-shaped generations fail before transport. Special-key delivery remains explicitly unsupported, and Pi hard pause remains unsupported at the harness layer even though the selected POSIX process backend can hard-pause its owned tree. No unsupported operation reports success.

## Runtime proof

`hard_pause_is_generation_fenced_and_suspends_the_complete_group` proves:

1. a live child tree advances an observable counter;
2. exact-generation hard pause stops progress;
3. stale-generation resume is rejected and progress remains stopped;
4. exact-generation resume restarts progress;
5. final process-group cleanup succeeds.

The existing API test `lifecycle_transition_routes_durably_reduce_and_reject_stale_generation` proves portable pause/resume lifecycle CAS and stale-generation rejection.

The strengthened Pi adapter test proves both steering and follow-up frames carry the exact run ref, invalid generation is rejected before transport, and special keys/hard pause remain unsupported in the declared harness contract.

## Commands and results

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-session-runner
```

Result: 31 unit tests and 1 protected-runner E2E passed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-harness-adapters
```

Result: 6 adapter-contract tests and 5 model-safety tests passed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-api lifecycle_transition_routes_durably_reduce_and_reject_stale_generation
```

Result: 1 targeted API test passed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-session-runner -p focusa-harness-adapters --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

This slice does not claim PTY/special-key support on the direct backend or Pi adapter. Those capabilities remain unsupported until a separately negotiated backend proves them. Retry budgets, orphan/reboot recovery, resource admission, and complete failure envelopes remain dependency-ordered in `.5.3` through `.5.7`.

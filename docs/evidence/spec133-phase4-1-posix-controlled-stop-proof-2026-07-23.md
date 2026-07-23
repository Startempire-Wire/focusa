# Spec 133 Phase 4.1 — POSIX controlled-stop proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.1`
Scope: Spec 133 §13.1 and §13.5
Call-stack design: `019f8da9-0331-76d2-b676-788a9a0f519e`

## Implemented contract

The direct POSIX backend now exposes one runner-owned controlled-stop operation over the already verified process-group identity.

The operation emits ordered, serializable `focusa.controlled_stop_report.v1` stage events for:

1. harness-native abort request and accepted/unavailable/failed disposition;
2. native-abort grace expiry;
3. process-group `SIGTERM` request;
4. graceful-termination grace expiry;
5. process-group `SIGKILL` request;
6. process-group leak verification;
7. terminal stopped, already-exited, or leak-detected verdict.

A nonzero bounded polling interval is mandatory. Process-group liveness uses signal-zero probing and treats `EPERM` as alive and `ESRCH` as absent. The supervisor retains ownership when a leak verdict occurs so a caller may continue remediation; clean terminal outcomes remove the run.

Existing authority and isolation remain intact:

- project-owner identity and workspace are revalidated before spawn;
- each run receives a dedicated process group;
- cwd and environment are explicit;
- reserved runner identity variables cannot be overridden;
- force-only termination remains available for compatibility;
- no shell-composed stop command or PID-only kill path was introduced.

## Focused runtime proof

`controlled_stop_prefers_harness_native_abort_and_events_every_stage` proves a native abort exits without TERM or force escalation and emits the exact five-stage successful sequence.

`controlled_stop_force_kills_term_resistant_descendants_and_verifies_no_leak` creates a TERM-resistant child tree, advances through both grace periods, kills the complete process group, and records a passed leak verdict.

## Commands and results

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 \
CARGO_INCREMENTAL=0 \
cargo test -p focusa-session-runner -- --nocapture
```

Result: 30 unit tests passed, 1 protected-runner E2E passed, 0 failed.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 \
CARGO_INCREMENTAL=0 \
cargo clippy -p focusa-session-runner --all-targets -- -D warnings
```

Result: passed with zero warnings.

```bash
cargo fmt --all -- --check
git diff --check
```

Result: passed.

The isolated Cargo target is deliberate: a first attempt against the shared incremental target encountered target-directory contention before compiling the changed crate. The isolated non-incremental rerun is the authoritative proof.

## Remaining Phase 4 work

This slice does not claim hard pause, typed retry budgets, orphan/reboot recovery, resource admission, or complete failure envelopes. Those remain separately dependency-ordered in `focusa-a6yq6.5.2` through `.5.7`.

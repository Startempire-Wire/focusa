# Spec 133 Phase 4.5 — resource admission and backpressure proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.5`
Scope: Spec 133 §21

## Admission

`focusa.resource_admission_decision.v1` evaluates before launch:

- global, user, project, and provider concurrency quotas;
- writer lease, worktree, runner, model entitlement, Context Authority, and Workpoint readiness;
- current ResourceMode;
- available CPU, memory, disk, and durable stream-spool capacity;
- requested memory/disk/output limits;
- explicit native-enforcement requirements against truthful backend capability declarations;
- complete policies for turn, run, session, work-item, project, user, provider/model, and global-host budget levels.

Zero quotas, missing prerequisites, emergency ResourceMode, unavailable capacity, incomplete/invalid budget levels, or unsupported required enforcement reject admission. Constrained/LowMem admission is marked degraded instead of healthy.

## Usage and pressure

`focusa.resource_pressure_decision.v1` consumes typed usage for CPU, memory, PIDs, open files, I/O, disk, wall time, output, input/output/cache tokens, estimated/provider cost, subscription usage, context pressure, retry waste, and turns.

A bounded basis-point threshold emits warnings before hard limits. Hard-limit policy explicitly chooses checkpoint-and-pause or cancel. Decisions retain all usage truth and expose warning/hard reason sets; unsupported metrics remain absent rather than fabricated.

## Output backpressure

`NonBlockingStreamCapture` now requires a durable overflow JSONL spool. Queue-full and consumer-disconnected records are fsynced to the spool without blocking on subscriber consumption. The status still distinguishes live backpressure from disconnected consumers; spool failure is a separate `DurabilityFailed` verdict. A slow or absent UI therefore cannot block the process or erase the record that triggered pressure.

## Proof

Focused tests cover complete admission, simultaneous quota/prerequisite/emergency/capability denial, missing budget levels, warning thresholds, checkpoint/pause and cancel actions, and durable ordered overflow for full/disconnected queues.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 393 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Enforcement boundary

This policy layer does not overclaim cgroup, rlimit, I/O-priority, or disk-quota enforcement. Backends must declare each dimension Native, Advisory, or Unsupported; admission fails if policy requires Native and the selected backend cannot prove it. VPS/platform enforcement and pressure E2E remain part of later Phase 4/final runtime gates.

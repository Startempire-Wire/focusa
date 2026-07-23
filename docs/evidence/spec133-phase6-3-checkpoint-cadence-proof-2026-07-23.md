# Spec 133 Phase 6.3 — checkpoint cadence proof

Date: 2026-07-23
Bead: `focusa-a6yq6.7.3`
Scope: Spec 133 §20.1–§20.2

## Runtime checkpoint policy

`RuntimeCheckpointPolicy` supports both configured interval and semantic-event cadence. `evaluate_runtime_checkpoint` combines all simultaneously due reasons into one bounded directive:

- time interval;
- semantic-event interval;
- tool completion with durable project change;
- before retry escalation;
- after retry escalation;
- before pause;
- before process restart;
- before daemon upgrade;
- runner disconnect.

A due runtime checkpoint requires a stream cursor, non-empty resource counters, and retry state. Clock regression and zero cadence fail closed. Runtime directives can target only `SilentSessionRuntimeStore`.

## Canonical Workpoint checkpoint policy

`MeaningSnapshot` tracks the existing canonical Workpoint plus mission, ActionIntent, active objects, blockers, verified evidence, next slice, work item, operator direction, model binding, and completion-evaluation state.

`evaluate_workpoint_checkpoint` emits the existing Workpoint ref only when one or more meaning changes occur:

1. mission or ActionIntent;
2. active object set;
3. blockers;
4. evidence;
5. next slice;
6. work-item advancement;
7. operator steering direction;
8. model switch;
9. completion evaluation begins.

Initial canonical binding is explicit. A changed Workpoint identity is rejected rather than minted by the policy. Unchanged state and set-order-only changes return `MeaningUnchanged`, preventing heartbeat or ordering spam.

Canonical directives can target only `CanonicalWorkpointPath`. There are no transcript, log, or Focus State sinks, so the policy cannot impersonate those authority surfaces.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or test execution was performed.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_checkpoint_policy.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_checkpoint_policy -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove all runtime boundaries, combined due reasons, runtime state requirements, all nine semantic triggers, unchanged-state rejection, order-insensitive sets, and preservation of the existing canonical Workpoint ref.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

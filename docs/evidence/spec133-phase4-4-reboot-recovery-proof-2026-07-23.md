# Spec 133 Phase 4.4 — reboot recovery proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.4`
Scope: Spec 133 §13.8

## Implemented contract

`focusa.reboot_recovery_decision.v1` is a deterministic core recovery planner over durable session/run state.

It requires distinct nonempty previous/current boot identities and valid UUIDv7 session/run IDs. After a verified boot change:

- `original_process_survived` is always `false`;
- every unfinished lifecycle is classified `orphaned`;
- terminal completed/failed/cancelled runs remain terminal and never relaunch;
- the latest runtime checkpoint and latest nonempty Workpoint checkpoint are loaded as explicit decision refs;
- missing either checkpoint blocks relaunch;
- `never`, bounded checkpointed, and operator-acknowledged policies are explicit;
- bounded policy enforces `max_process_restarts` against prior generations;
- operator policy requires a nonempty acknowledgment ref;
- only a permitted decision creates a fresh UUIDv7 run ID and checked `generation + 1`;
- unknown policies, unchanged boot identity, invalid IDs, and generation overflow fail closed.

The planner never mutates or reuses the old process/run identity. Daemon persistence/routes may consume the decision in later integration slices without reinterpreting reboot truth.

## Proof

Focused tests cover:

1. running session → orphaned → checkpointed fresh generation;
2. missing runtime checkpoint;
3. `never` policy;
4. exhausted restart budget;
5. required operator acknowledgment and approved relaunch;
6. terminal no-relaunch;
7. unchanged boot identity rejection.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 391 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

Resource admission/limits, complete failure envelopes, and the combined Phase 4 gate remain `.5.5`–`.5.7`. Official reboot/runtime deployment proof remains part of later exhaustive VPS release gates.

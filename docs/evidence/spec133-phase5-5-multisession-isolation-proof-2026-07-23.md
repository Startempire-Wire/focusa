# Spec 133 Phase 5.5 — foreground/background and multi-session isolation proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.5`
Scope: Spec 133 §18.1–§18.7 isolation behavior

## Adversarial matrix

`tests/spec133_phase5_isolation_gate.sh` is wired into strict CI and covers:

- foreground/Work Loop/Silent Session claim conflicts;
- dirty sole-owner renewal and second-writer rejection;
- expired lease replacement with a higher fencing token;
- read-only shared inspection;
- explicit shared approval, path overlap rejection, and visible warning;
- collision-safe isolated worktree planning/materialization;
- one canonical Work Loop dependency scheduler with alternate-ready selection;
- governed integration conflict blocking and dirty-state preservation;
- runner identity and owner-safe mutation barriers.

## Real git concurrency proof

The runtime test creates a real primary repository, commits a base, then leaves the primary tracked file dirty. Two different Silent Session plans create separate branches and worktrees. Each session stages and commits a same-named file independently.

The proof verifies:

- primary HEAD never moves;
- primary dirty content remains byte-for-byte unchanged;
- primary index/status never gains either session file;
- both session branches receive distinct commits;
- session worktree roots/branches are distinct and bound to their sessions;
- cleanup uses explicit git worktree removal only after assertions.

This test exposed and fixed a real collision defect: the first eight UUIDv7 characters contain only timestamp material. Session short IDs now combine timestamp and random UUID components; deterministic hash suffixing remains the secondary collision fallback.

## Commands and results

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  bash tests/spec133_phase5_isolation_gate.sh
```

Result: writer 4, workspace 4, scheduler 3, integration 3, identity 6, mutation 4 tests passed.

```bash
cargo test -q -p focusa-core
cargo test -q -p focusa-session-runner
```

Result: 409 core tests, 31 runner unit tests, and 1 protected-runner E2E passed; one unrelated core integration test remained ignored.

```bash
cargo clippy -p focusa-core -p focusa-session-runner --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed. CI workflow YAML and gate shell syntax also passed.

## Remaining boundary

The combined Phase 5 gate remains `.6.6`. Official root/wirebot and VPS platform proof remains part of later exhaustive release gates; local code never claims those identities.

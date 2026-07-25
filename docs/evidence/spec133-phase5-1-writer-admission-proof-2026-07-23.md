# Spec 133 Phase 5.1 — writer admission and lease proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.1`
Scope: Spec 133 §18.1–§18.4

## Lease scope

`SilentSessionLease` now includes typed mutation mode in addition to project root, continuity, work item, workspace, path intents, owner, role, fencing token, and expiry. Unsafe/empty scope and path-intent traversal fail validation.

## Conflict analysis

`focusa.writer_admission_decision.v1` evaluates candidates against active foreground, Work Loop, and Silent Session claims.

Rules proven:

- read-only shared inspection does not acquire writer authority;
- a dirty workspace remains valid for its sole exact owner and lease renewal;
- a second writer in the same dirty workspace is blocked;
- same-work-item ownership conflicts even across writer kinds;
- isolated worktrees with distinct work items avoid false conflicts;
- explicit shared mode requires approval and non-overlapping path intents;
- expired claims do not block admission;
- scope/path/actor/workspace omissions fail closed;
- conflict results identify actors/lease IDs and when isolation is required.

## Fencing and durability

`WriterLeaseRegistry` issues positive process-monotonic fencing tokens. Renewals require exact owner, unexpired lease, and current token, then rotate to a new token. Release requires the rotated token; stale renew/release attempts fail.

The registry has a canonical SQLite singleton projection with revision CAS:

- first load returns revision 0 and a valid empty registry;
- writes validate schema/token monotonicity;
- stale expected revision is rejected;
- registry/token state survives database reopen;
- transaction serialization prevents two successful writers at one revision.

This removes lease authority from plugin/process memory while retaining current Work Loop claims as external conflict inputs.

## Proof

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 399 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-session-runner
```

Result: 31 units and 1 protected-runner E2E passed.

Targeted API lifecycle generation test passed, proving existing lifecycle controls still compile and reject stale generation with the expanded lease schema.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core -p focusa-session-runner -p focusa-api \
    --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

Worktree creation/default naming is Phase 5.2. Scheduler integration, governed integration, and multi-session isolation remain `.6.3`–`.6.6`.

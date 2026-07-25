# Spec 133 Phase 7.1 — persistent daemon-backed dashboard

Date: 2026-07-23
Bead: `focusa-a6yq6.8.1`
Scope: Spec 133 §22.2 and §22.5

## Dashboard endpoint

The daemon now serves:

```text
GET /v1/silent-sessions/dashboard?limit=<bounded>
```

The projection is built entirely from durable SQLite sessions, runs, and events and reports:

- all visible bounded sessions;
- lifecycle state and health;
- project root/identity, continuity, work item, worktree;
- requested/effective/observed model;
- elapsed time;
- current semantic activity;
- twenty most recent bounded structured events;
- latest resource/usage payload;
- last Workpoint/runtime checkpoint;
- operator-attention reason for waiting, blocked, orphaned, failed, or unhealthy state;
- output/checkpoint/receipt evidence refs;
- completion status;
- lifecycle-safe controls.

The response declares `restart_safe = true` and `source = daemon_sqlite`; it does not depend on Pi/plugin process memory.

## Control honesty

Controls are derived from lifecycle state. Terminal states expose only open-worktree/evidence/receipt actions. Running, waiting, paused, orphaned, blocked, and transitional states receive bounded applicable controls.

Hard pause is intentionally omitted because the dashboard projection does not yet carry a verified per-run capability binding. It must not advertise an unsupported hard-pause control.

## Boundedness and redaction boundary

List limit uses the existing strict observation bounds. Event loading is capped at the global bounded maximum and then reduced to the latest twenty events. Payloads come from the existing durable event path and preserve its redaction boundary.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-api/src/routes/silent_sessions.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-api list_and_output_routes_are_bounded_exact_and_cursor_preserving -- --nocapture
cargo test -p focusa-api silent_sessions -- --nocapture
cargo test -p focusa-api
cargo clippy -p focusa-api --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server proof must verify daemon restart persistence, bounded list/event behavior, project/worktree/model/activity/resource/checkpoint/evidence/completion fields, lifecycle controls, and route-contract parity.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

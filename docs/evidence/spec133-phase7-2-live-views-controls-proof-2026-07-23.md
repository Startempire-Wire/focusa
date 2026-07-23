# Spec 133 Phase 7.2 — live views, cursors, and operator controls

Date: 2026-07-23
Bead: `focusa-a6yq6.8.2`
Scope: Spec 133 §22.3–§22.5

## Live view modes

`focusa silent watch` and `focusa silent output` now support:

- `summary`;
- `agent-text`;
- `tools`;
- `stdout`;
- `stderr`;
- `events`;
- `raw`;
- `evidence`.

Watch filtering occurs after each daemon SSE frame is validated and its opaque event cursor consumed. Filtered events therefore cannot stall or corrupt cursor-follow state. One-shot output filtering preserves the daemon `next_cursor` while updating visible event count.

Legacy `--tools` and `--stderr` remain union filters; with default `events` they preserve their historical filter-only behavior.

## Complete controls

CLI and API surfaces now cover:

- text, follow-up, steering, special keys;
- pause, resume, interrupt;
- controlled stop;
- force cancel;
- restart and adopt;
- governed handoff;
- open/show exact worktree;
- evidence and receipt reads.

Controlled stop and force cancel are distinct lifecycle actions and receipt/action-digest identities.

All mutating controls continue through exact session/run/generation, durable approval, writer lease, authorization, reducer, event append, and receipt paths.

## Handoff authority

Handoff fails closed unless both an existing Session Transfer ref and explicit writer-handoff ref are supplied. Those refs are persisted into the canonical lifecycle event payload and artifact refs. The approval ID remains the operator authorization binding.

## Worktree and proof reads

`GET /v1/silent-sessions/{session_id}/worktree` requires an exact run ID and optional expected generation and returns durable project/workspace strategy/branch data. Evidence and receipt endpoints retain bounded exact-run cursor semantics.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-api/src/routes/silent_sessions.rs \
  crates/focusa-cli/src/commands/silent.rs
git diff --check
```

A static contract check verified all eight modes, fifteen required control/read routes and CLI commands, and explicit handoff refs.

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-cli commands::silent -- --nocapture
cargo test -p focusa-api silent_sessions -- --nocapture
cargo test -p focusa-cli
cargo test -p focusa-api
cargo clippy -p focusa-cli -p focusa-api --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove all modes, legacy filter behavior, cursor advancement across hidden events, output cursor preservation, every lifecycle control, stop/cancel distinction, handoff ref rejection/persistence, worktree run guards, and bounded evidence/receipt reads.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

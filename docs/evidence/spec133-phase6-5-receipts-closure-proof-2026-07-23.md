# Spec 133 Phase 6.5 — receipts and closure authority proof

Date: 2026-07-23
Bead: `focusa-a6yq6.7.5`
Scope: Spec 133 §19.14–§19.15

## Spec 119 receipt projection

`SilentSessionReceiptProjection` maps all required receipt uses:

- `work_session`;
- `risky_mutation`;
- `blocked_claim`;
- `handoff`;
- `bootstrap_delivery`;
- `work_item_closure`;
- `final_report`.

Every projection has `execution_mode = silent_session` and binds session/run, ProjectIdentity, Continuity, Workpoint, work item, target claim, evidence refs, event cursor, and bounded payload.

## No second ledger

The module owns no durable store or event hash calculation. `into_existing_event_append` emits an idempotent `receipt_commit_requested` payload for the existing Silent Session event/hash chain.

The append binding requires:

- typed `ExistingSilentSessionEventChain` kind;
- existing append target ref;
- exact matching event cursor;
- valid 64-hex previous event hash.

The append request explicitly reports `creates_new_ledger = false`; the existing event store remains responsible for sequence, event hash, and durable commit.

## Governed closure

A Silent Session may create a closure proposal only. Closure then follows the exact state machine:

```text
proposed
  → validated
  → authorized
  → provider_submitted
  → reconciled
  → audited
```

Each transition requires its own evidence ref. Authorization by the same session actor is rejected. Stages cannot be skipped, repeated, or reordered. Audit is rejected unless provider reconciliation reports the work item closed.

A `work_item_closure` receipt can be projected only after all six stages and carries validation, external authority, provider submission, provider reconciliation, and audit refs. Agent final text is not closure evidence or authority.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_receipts.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_receipts -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove all receipt mappings, existing-chain-only append requests, invalid prior-hash/cursor rejection, skipped-stage rejection, self-closure rejection, provider-observation requirement, audited closure receipt evidence, and strict lint.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

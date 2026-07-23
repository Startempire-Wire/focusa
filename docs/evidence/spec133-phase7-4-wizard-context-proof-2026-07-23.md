# Spec 133 Phase 7.4 — creation wizard and context-flood protection

Date: 2026-07-23
Bead: `focusa-a6yq6.8.4`
Scope: Spec 133 §22.7–§22.8

## Exact 13-step wizard

`SilentSessionWizardDraft` enforces this exact sequence:

1. verified ProjectIdentity;
2. Continuity and Workpoint;
3. work item and mission;
4. workspace strategy/root;
5. harness profile;
6. exact provider/model/thinking;
7. verified authentication and entitlement;
8. policy preset;
9. non-zero resource and cost budgets;
10. Context Authority and writer lease;
11. effective configuration preview;
12. explicit approval and launch action digest;
13. open live watch.

Steps cannot be skipped, repeated, or reordered. Invalid identity, provider access, budgets, authority/lease, config digest, or approval fail closed.

## Provider/model visibility before mutation

The selected exact model is durable at step 6. Step 11 requires an `EffectiveConfigurationPreview` containing the same provider/model/thinking and workspace plus config ref/hash.

The preview must report `mutation_allowed = false`; the wizard also remains mutation-disabled through step 11. Only explicit approval and a valid action digest at step 12 enable mutation. Live watch is required as the final step.

## Context-flood protection

`OperatorContextSummary` exposes only:

- meaningful deltas;
- current action;
- errors and blockers;
- tool boundaries;
- evidence and checkpoint refs;
- full-output cursor and artifact handle.

Each list is capped at twenty items and each item at 500 bytes. Full output cannot be inlined; it remains available through cursor/artifact handles.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_wizard.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_wizard -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove all thirteen steps, skipped/repeated-step rejection, identity/auth/entitlement failures, exact model/config preview equality, mutation disabled before approval, automatic watch last, and summary item/text/full-output bounds.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

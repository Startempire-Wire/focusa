# Spec 133 Phase 6.4 — completion evidence and decision proof

Date: 2026-07-23
Bead: `focusa-a6yq6.7.4`
Scope: Spec 133 §20.3–§20.4

## Completion evidence bundle

`CompletionEvidenceBundle` requires the Spec 133 minimum for code-changing sessions:

- exact project root/identity and worktree root/ref;
- starting and ending git status refs, hashes, heads, and dirty state;
- bounded diff summary (maximum 16 KiB), full diff artifact/hash, and files changed;
- required verification classes with command refs, output artifacts/hashes, exit codes;
- commit refs when policy requires them;
- final Workpoint checkpoint;
- unresolved blockers;
- Context Authority refs;
- requested/effective/observed model usage and cost counters;
- resource usage;
- stream manifest ref/hash;
- completion verifier result;
- receipt preview ref/hash.

## Fail-closed completion decision

`evaluate_completion` accepts only an exited run whose session is already `completing`. It refresh-gates ProjectIdentity and Workpoint and evaluates policy-required evidence and acceptance.

Outcomes:

- missing evidence → `blocked`, reason `completion_evidence_missing`;
- unresolved blockers → `blocked`, reason `unresolved_blockers`;
- nonzero process exit, failed required verification, failed completion verifier, failed acceptance, or failed adversarial verifier → `failed`, reason `verification_failed`;
- complete evidence plus receipt preview but no receipt commit → remains `completing`/`incomplete`, reason `receipt_not_committed`;
- only complete evidence, refreshed authority, passing verification/acceptance, no blockers, and a committed receipt → `completed`.

The evaluator produces the existing `SilentSessionCompletionEvaluation` record and never treats an agent final message as evidence or closure truth.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_completion.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_completion -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove successful committed completion, missing-evidence blocking, unresolved-blocker blocking, process/test/completion-verifier failure, acceptance/adversarial failure, receipt-preview-without-commit incompleteness, exact project/worktree matching, required verification classes, and bounded artifact metadata.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.

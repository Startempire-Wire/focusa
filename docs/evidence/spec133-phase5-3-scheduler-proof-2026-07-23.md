# Spec 133 Phase 5.3 — Work Loop-owned scheduler proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.3`
Scope: Spec 133 §18.6

## One scheduler law

`select_silent_session_dispatch` calls the existing canonical provider-neutral Work Loop `evaluate_readiness` function. It does not evaluate dependencies, parent graphs, provider status, or blocked siblings independently and therefore cannot become a second scheduler.

The Silent Session overlay only filters/ranks the canonical ready set by:

- global/user/project/provider quota and resource admission result;
- writer admission/lease result;
- declared session priority (`interactive`, `high`, `normal`, `background`, `low`, `maintenance`);
- normalized Work Item priority;
- queue age and session ID for deterministic ties.

## Deferral and alternate work

Each non-dispatched candidate receives a typed deferral:

- `work_item_not_ready`, preserving the canonical Work Loop blocked reason;
- `resource_admission_denied`, preserving quota/resource denials;
- `writer_admission_denied`, preserving conflict/fencing denials.

A blocked high-priority sibling does not freeze the scheduler: another canonical-ready, admitted candidate is selected. No retry or completion state is invented by the overlay.

## Proof

Focused tests prove:

1. dependency-blocked interactive work defers while alternate ready work dispatches;
2. quota-denied and writer-denied candidates defer while another candidate runs;
3. session priority, Work Item priority, queue age, and session ID produce deterministic ordering.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 405 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

Governed integration, foreground/background multi-session isolation, and the combined Phase 5 gate remain `.6.4`–`.6.6`.

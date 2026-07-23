# Spec 133 Phase 4 gate proof

Date: 2026-07-23
Bead: `focusa-a6yq6.5.7`
Scope: Spec 133 Phase 4 — §§13, 21, and 28

## Implementation commits

- `6618dfd9` — evented POSIX controlled process-tree stop;
- `9259039e` — generation-fenced hard pause and control truth;
- `19905b96` — independent typed retries and orphan reconciliation;
- `b1f19b1d` — checkpointed reboot recovery policy;
- `55bb21bc` — resource admission, usage pressure, and durable backpressure;
- `04dd7957` — complete 42-class failure/recovery envelope.

## Combined mandatory gate

`tests/spec133_phase4_runtime_gate.sh` is wired into the Rust CI job before strict workspace clippy. It fails if any Phase 4 evidence artifact is missing, then executes the combined fault/recovery matrix:

- live POSIX process ownership, hard pause, native abort, TERM-resistant descendant force cleanup, and leak verification;
- exact adoption and signed reconciliation;
- all seven independent retry budgets and bounded backoff;
- reboot orphan classification, checkpoint/policy/operator gates, and fresh generation;
- global/user/project/provider admission, eight budget scopes, enforcement truth, usage warning and hard-limit actions;
- nonblocking durable output overflow under full/disconnected subscribers;
- all 42 canonical failure/recovery classes.

Command:

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 \
CARGO_INCREMENTAL=0 \
bash tests/spec133_phase4_runtime_gate.sh
```

Result:

```text
process_posix: 6 passed
adoption/reconciliation: 3 passed
retry: 2 passed
reboot recovery: 3 passed
resource admission/pressure: 2 passed
durable output overflow: 1 passed
failure taxonomy: 2 passed
PASS: Spec133 Phase 4 supervision, recovery, resource, backpressure, and failure matrix
```

Additional gates already recorded by the six child evidence reports:

- 395 full core tests passed after the final Phase 4 slice;
- 31 full session-runner unit tests and 1 protected-runner E2E passed;
- 11 harness adapter/model-safety tests passed;
- strict all-target clippy passed for every changed crate;
- `cargo fmt --all -- --check`, workflow YAML parsing, shell syntax, and `git diff --check` passed.

## Gate disposition

Phase 4 implementation and local deterministic proof are complete. Official installed daemon, reboot, platform, and release proof remains dependency-ordered in later Spec 133 cross-platform/exhaustive gates and must run through the VPS release pipeline; this local gate does not claim deployment.

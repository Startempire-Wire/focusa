# Spec 133 Phase 6.1 — Focusa authority integration

Date: 2026-07-23
Bead: `focusa-a6yq6.7.1`
Scope: Spec 133 §19.1–§19.5

## Implemented authority chain

A Silent Session now durably captures an exact `OperatorAskBinding` with source ref, exact text, SHA-256, monotonic revision, and capture time. Session validation rejects empty, revision-zero, or digest-mismatched Ask bindings.

`focusa.silent_session_authority.v1` composes one run-scoped envelope across:

- exact operator Ask;
- ProjectIdentity root/ref;
- Continuity ID;
- canonical-advisory Trajectory ref, waypoints, and active gap;
- canonical Workpoint binding as immediate action authority;
- exact next action, active objects, hook refs, blockers, and do-not-drift;
- evented operator steering bound to session, run, generation, project, continuity, and Workpoint.

Direction revisions are unique. A newer scoped operator steering event supersedes the creation Ask and older steering. Stale or cross-scope prompts cannot outrank the current direction.

## Fail-closed behavior

- bootstrap and session Ask bindings must match exactly;
- project root/ref, continuity, trajectory, Workpoint, session/run/generation must agree;
- Trajectory remains explicitly advisory;
- Workpoint remains explicitly action authority;
- `generic_degraded` trajectory blocks canonical bootstrap;
- missing trajectory waypoints or active gap blocks bootstrap;
- bootstrap verification receipts bind Ask ref, digest, and revision;
- changed Ask content invalidates a prior verification receipt.

## Local non-building proof

Per operator policy, this shared machine performs no Cargo, CI, compilation, or test execution. The final implementation passed only non-building checks:

```bash
rustfmt --edition 2024 --check <changed Rust files>
git diff --check
bash -n tests/spec133_phase4_runtime_gate.sh tests/spec133_phase5_isolation_gate.sh
python3 -m py_compile tests/spec98_work_loop_execution_partition_static_test.py
```

Result: passed.

A static constructor audit found all real `SilentSession`, core/runner `AgentBootstrapPacket`, and `TrajectoryBootstrapBinding` literals contain their newly required fields. Same-named API preload types and `impl` blocks were explicitly excluded as unrelated types/non-literals.

## Required server proof

Run these only on the build server:

```bash
cargo test -p focusa-core silent_session_authority -- --nocapture
cargo test -p focusa-core silent_session_bootstrap -- --nocapture
cargo test -p focusa-core
cargo test -p focusa-session-runner mutation_posix -- --nocapture
cargo test -p focusa-session-runner
cargo test -p focusa-api silent_sessions -- --nocapture
cargo clippy -p focusa-core -p focusa-session-runner -p focusa-api --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server proof must confirm the end-to-end composition fixture, Ask tamper rejection, generic trajectory blocking, stale/cross-scope steering rejection, dependent constructors, and strict lint.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains explicitly server-owned; this bead must not be represented as fully proven until those commands pass there.

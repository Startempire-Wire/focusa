# Spec 133 Phase 5 gate proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.6`
Scope: Spec 133 §18 concurrency, worktrees, scheduling, and integration

## Implementation commits

- `8c0135b7` — durable writer admission, lease scopes, CAS, and fencing;
- `dab5ca26` — sanitized collision-safe workspace strategy and real worktree materialization;
- `01b74492` — Silent Session dispatch through the canonical Work Loop scheduler;
- `deb2a338` — evidence-gated governed integration and receipt;
- `30546d91` — adversarial foreground/background multi-session isolation and CI gate.

## Combined release gate

`tests/spec133_phase5_isolation_gate.sh` is wired into strict CI and requires every Phase 5 evidence artifact. It runs:

- writer conflict/renewal/expiry/fencing and SQLite restart/CAS tests;
- all workspace strategies plus real one- and two-worktree git tests;
- dependency-blocked/alternate-ready scheduler tests;
- governed preflight/conflict/dirty-preservation/receipt tests;
- POSIX identity and owner-safe mutation barriers.

Command:

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 \
CARGO_INCREMENTAL=0 \
bash tests/spec133_phase5_isolation_gate.sh
```

Result:

```text
writer admission/fencing/persistence: 4 passed
workspace strategies/real git isolation: 4 passed
canonical scheduler overlay: 3 passed
governed integration: 3 passed
runner identity: 6 passed
owner-safe mutation: 4 passed
PASS: Spec133 Phase 5 writer, worktree, scheduler, integration, and isolation matrix
```

Latest full gates after the isolation implementation:

- 409 core tests passed;
- 31 session-runner unit tests and 1 protected-runner E2E passed;
- strict all-target clippy passed for core and runner;
- formatting, shell syntax, workflow YAML, and diff checks passed.

## Operator-work preservation verdict

The real git matrix proves background sessions do not move the primary HEAD, alter its index, overwrite dirty content, or share a mutable worktree. Explicit shared mode remains approval/path-conflict gated. Integration conflicts become blocked outcomes and destructive cleanup is rejected.

## Gate disposition

Phase 5 is implementation-complete and CI-gated. Official platform ownership and release integration remain later Spec 133/VPS gates; no local release is implied.

# Spec 133 Phase 5.2 — workspace strategy proof

Date: 2026-07-23
Bead: `focusa-a6yq6.6.2`
Scope: Spec 133 §18.2, §18.4, §18.5

## Implemented contract

`focusa.silent_session_workspace_plan.v1` selects and validates all four workspace strategies:

- background mutation defaults to `isolated_worktree`;
- `exclusive_existing` requires an acquired lease and no competing writer;
- `read_only_shared` carries no branch or mutation authority;
- `explicit_shared` requires an approval ref and safe path intents, and emits visible conflict-monitoring warnings.

## Isolated naming and binding

Default names follow the spec:

```text
branch: focusa/silent/<session-short-id>/<sanitized-work-item>
path:   <worktree-root>/<sanitized-project>/<session-short-id>
```

Project/work-item segments are lowercase, bounded, separator-normalized, and traversal-safe. Existing branch/path collisions append a deterministic eight-character SHA-256 suffix. Existing worktree roots are canonicalized before planning so macOS aliases such as `/var` and `/private/var` cannot create identity mismatches.

## Owner-safe materialization

Isolated materialization:

- invokes `git` with direct argv, never a shell-composed command;
- canonicalizes and verifies the source repository;
- rejects existing targets;
- creates and validates a non-symlink, non-shared-writable parent;
- runs `git worktree add -b <branch> <path> <base>`;
- canonicalizes the result and proves exact planned root, `.git` binding, branch ref, strategy, and session workspace ID.

A real temporary repository test initializes and commits a source repository, materializes the planned worktree, verifies the binding, then removes it through git.

## Proof

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo test -q -p focusa-core
```

Result: 402 passed, 0 failed; one unrelated ignored integration test remained ignored.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-spec133-451 CARGO_INCREMENTAL=0 \
  cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Result: passed.

## Remaining boundary

Dependency-aware scheduling, governed integration, multi-session isolation, and the combined Phase 5 gate remain `.6.3`–`.6.6`.

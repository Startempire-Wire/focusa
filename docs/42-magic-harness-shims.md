# docs/42-magic-harness-shims.md — Magic Harness Shims (Desired UX)

## Goal (Desired)

When a user runs a harness CLI directly (e.g. `pi`, `claude`, `codex`), Focusa should **automatically** intercept the session and route it through **Mode A** (`focusa wrap`) without the user needing to remember a prefix.

This is the "magic" UX:
- `pi "..."` transparently becomes `focusa wrap -- pi "..."`
- daemon auto-start happens as needed
- fail-open behavior: if Focusa is unavailable, harness runs normally

This is how Focusa "picks up agent sessions automatically" in a local dev shell.

## Why this is needed

Mode A (CLI wrapping) is the MVP primary integration per:
- `docs/G1-detail-04-proxy-adapter.md`

But without shims/aliases, users will naturally run `pi` directly, and Focusa will see nothing.

## Design

### Approach: PATH shims (recommended)

Install small executable wrappers **named after harnesses** into a directory earlier in `$PATH` (e.g. `~/.local/bin`).

Example shims:
- `~/.local/bin/pi`
- `~/.local/bin/claude`

Each shim:
1. checks if Focusa magic is disabled (`FOCUSA_MAGIC_DISABLE=1`)
2. checks `focusa` exists
3. resolves the real harness binary
4. `exec focusa wrap -- <real_harness> <args...>`

### Fail-open behavior

If Focusa isn’t installed or errors early:
- run harness directly

### Recursion guard

If the shim name shadows the real harness:
- resolve the real harness by searching `$PATH` with the shim directory removed

## Implementation (repo)

Scripts:
- `scripts/magic/focusa-magic.sh` — generic wrapper
- `scripts/magic/install.sh` — installs per-harness symlinks into `~/.local/bin`

Install examples:
```bash
# install shims
./scripts/magic/install.sh pi claude opencode

# then (in a new shell)
pi "hello"        # transparently routed through focusa
claude "hello"    # transparently routed through focusa
opencode           # transparently routed through focusa (best-effort)
```

## Notes / Future

- On macOS, consider writing the shim in Swift for better integration, but shell is fine for MVP.
- If we want system-wide, package shims via installer + update PATH.
- Alternatively: recommend shell aliases, but PATH shims are more reliable across shells and subprocesses.

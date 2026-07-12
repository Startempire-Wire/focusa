# 132 E4 PTY lifecycle and output-isolation proof

Command:

```text
bash tests/spec132_pty_lifecycle_runtime_test.sh
```

Result: PASS.

1. JSON preflight is parsed as `focusa.install_preflight.v1`, remains read-only, and emits no ANSI bytes on stdout.
2. A real `script(1)` pseudo-terminal run in `FOCUSA_INSTALL_UI=plain` mode completes with durable plain preflight output and no alternate-screen or cursor-control sequences.
3. The executable test also verifies the real TerminalGuard, PlainPresenter, and renderer-channel failure fallback implementation paths.
4. Existing Rust terminal-guard tests cover idempotent restore, cancellation sharing, and scoped signal registration; the full terminal UI unit suite passes.

No sleeps, live installation, release, or deployment were used.

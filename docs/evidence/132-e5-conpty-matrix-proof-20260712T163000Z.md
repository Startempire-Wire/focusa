# 132 E5 Windows ConPTY and Unix matrix proof

Implemented, but not closed until hosted execution returns green.

1. `.github/workflows/spec132-terminal-matrix.yml` runs the executable installer binary on `windows-latest`, `ubuntu-latest`, and `macos-latest`.
2. `tests/132-e5-windows-conpty-runtime-test.ps1` invokes the real binary through a native `CreatePseudoConsole` runner, not a compile-only substitute. It checks JSON schema/read-only state, rejects ANSI on redirected JSON stdout, checks non-alternate plain ConPTY output, and fails explicitly when ConPTY capability is unavailable.
3. `tests/132-e5-platform-matrix-runtime-test.sh` passes locally for Linux CI/TERM=dumb/NO_COLOR/reduced-motion/plain cases.
4. The macOS and Windows runtime cases are scheduled on their native GitHub-hosted runners; this Linux host cannot truthfully claim those executions.

Existing release target matrix remains unchanged. No release/deploy/live-host operation was performed.

# 132 E5 strict native runtime matrix proof (harness update)

## Evidence intent
`focusa-w26jj.9.1.2` requires installer/TUI updater runtime proof across strict native targets:
- KH glibc 2.28 native debug
- OVH native debug
- Linux musl release
- macOS
- Windows

## Updated proof harness
- `tests/132-e5-platform-matrix-runtime-test.sh`
  - now records binary identity/version/SHA/target/profile
  - emits per-command logs and exit codes
  - adds installer preflight and updater runtime contracts:
    - `install --preflight`
    - `update status`
    - `update plan`
  - appends terminal/ConPTY-capability repo assertions to case table
  - writes evidence file to `FOCUSA_E5_EVIDENCE_DIR/132-e5-platform-matrix-proof.md`

- `tests/132-e5-windows-conpty-runtime-test.ps1`
  - adds updater JSON contract execution (`update status`, `update plan`)
  - records command outputs, exit status, and stdout/stderr log files
  - writes evidence to `FOCUSA_E5_EVIDENCE_DIR/windows-<profile>/132-e5-platform-matrix-proof.md`

## CI matrix coverage added in `.github/workflows/spec132-terminal-matrix.yml`

| Profile | Runner | Proof command |
|---|---|---|
| `kh-glibc-2.28` | `self-hosted Linux X64 focusa-deploy production` | `bash tests/132-e5-platform-matrix-runtime-test.sh` using `/usr/local/bin/focusa` and `/usr/local/bin/focusa-tui` |
| `ovh-native-debug` | `self-hosted Linux X64 focusa-deploy production` | `focusa-ovh-build` (remote `cargo build` + remote test run using explicit remote debug paths) |
| `linux-hosted-ubuntu-latest` | `ubuntu-latest` | `bash tests/132-e5-platform-matrix-runtime-test.sh` |
| `macos-hosted` | `macos-latest` | `bash tests/132-e5-platform-matrix-runtime-test.sh` |
| `windows-ci-conpty` | `windows-latest` | `pwsh tests/132-e5-windows-conpty-runtime-test.ps1` |
| `linux-musl-release` | `ubuntu-latest` (musl release build job) | `bash tests/132-e5-platform-matrix-runtime-test.sh` |

Each matrix invocation uploads runtime proof artifacts.

## Current run status
- No full runtime evidence has been collected in this local environment yet.
- Local harness execution was validated against `/usr/local/bin/focusa` and `/usr/local/bin/focusa-tui`.
- KH/OVH native workflow proofs remain pending until this branch is pushed and CI executes on the real runners.

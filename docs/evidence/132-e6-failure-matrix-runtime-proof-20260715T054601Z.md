# 132-E6 failure matrix runtime proof

Timestamp: 20260715T054601Z
Bead: `focusa-slxpz.5.6` — 132-E6 integrity/service/Pi/upgrade/cleanup failure matrix
Source baseline: `4e8511149114f76fba1bcf99c257e6dad2019571`; implementation/evidence commit is the git commit containing this file.
Proof SHA: `c8b7a8000c8c40fc89081b9a544dbcd1ee1534d48e446eb96cf905bb774a2046`

Host profile: `kh-glibc-2.28`
Configured target triple: `x86_64-unknown-linux-musl`

No mutation/release/deploy:
- acceptance runs are dry-run or local/unit transcript checks only
- no install state is mutated
- no live bootstrapper sync, push, publish, or deploy performed

CLI binary path: `/usr/local/bin/focusa`
- CLI version: `focusa 0.9.94-dev`
- CLI binary identity: `2106635809:15213508 1 15537648 2026-07-14 00:18:43.365231659 -0700 /usr/local/bin/focusa`
- CLI binary file: `ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), static-pie linked, BuildID[sha1]=f43c697498cc6e19250f8c3edee741de5d982ab9, stripped`
- CLI SHA-256: `5fd5b6d9e5bea976562c45f2d127f6790ed6bf9e936faade10bfe9cea9eb8bcd`

TUI binary path: `/usr/local/bin/focusa-tui`
- TUI version: `focusa-tui 0.9.94-dev`
- TUI binary identity: `2106635809:15215839 1 4409456 2026-07-13 22:46:24.166883034 -0700 /usr/local/bin/focusa-tui`
- TUI binary file: `ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), static-pie linked, BuildID[sha1]=385a110a659bc4b2dd2ae26e0c86d3f4e97b67c7, stripped`
- TUI SHA-256: `b335ff590ab5592ba04e525e029d957f64da931b263f7924602ea6b0bb2c2165`

## Runtime acceptance cases (all expected/actual exits)

| case | expected exit | actual exit |
|---|---:|---:|
| install-dry-run-pty-truecolor-plan | 0 | 0 |
| install-dry-run-json | 0 | 0 |
| install-dry-run-pty-no-color-plain | 0 | 0 |
| install-dry-run-pi-skipped | 0 | 0 |
| cargo-test-pi-activation-success | 0 | 0 |
| cargo-test-pi-missing | 0 | 0 |
| cargo-test-pi-malformed-archive | 0 | 0 |
| cargo-test-checksum-mismatch | 0 | 0 |
| cargo-test-windows-service-warning | 0 | 0 |
| cargo-test-atomic-cleanup | 0 | 0 |
| cargo-test-cancel-rollback | 0 | 0 |
| cargo-test-renderer-truecolor-transcript | 0 | 0 |
| cargo-test-renderer-monochrome-transcript | 0 | 0 |

## Explicit E6_* marker outcomes

Observed and asserted per case:
- `E6_PI_PRESENT_SUCCESS`
  - `cargo-test-pi-activation-success`
- `E6_PI_ABSENT`
  - `cargo-test-pi-missing`
- `E6_PI_FAILURE_SAFE`
  - `cargo-test-pi-malformed-archive`
- `E6_INTEGRITY_FAILURE`
  - `cargo-test-checksum-mismatch`
- `E6_SERVICE_WARNING`
  - `cargo-test-windows-service-warning`
- `E6_UPGRADE_CLEANUP`
  - `cargo-test-atomic-cleanup`
- `E6_CANCELLATION_ROLLBACK`
  - `cargo-test-cancel-rollback`

## Renderer assertions

### Truecolor acceptance (success)
- `cargo-test-renderer-truecolor-transcript` passed with terminal output containing:
  - `[truecolor transcript]`
  - `✓ Finalize`
  - `phase completion`

### Monochrome acceptance (integrity + cancellation framing)
- `cargo-test-renderer-monochrome-transcript` passed with terminal output containing:
  - `[monochrome transcript - failed pre-rollback]`
  - `[monochrome transcript - rollback started]`
  - `[monochrome transcript - rollback succeeded]`
  - `✗ Verify checksums and trust`
  - `↶ Rolling back safely`
  - `✗ Installation failed`

## 19/19 static gates

The companion static proof is fully complete in this workstream:
- `tests/spec_install_animation_static_test.sh`
- `tests/spec_install_animation_contract_test.sh`
- `tests/spec_install_animation_fallback_static_test.sh`
- `tests/spec_install_animation_security_static_test.sh`
- `tests/spec_install_pi_integration_rust_static_test.sh`
- `tests/spec_install_pi_integration_truth_test.sh`
- `tests/spec_install_service_rust_static_test.sh`
- `tests/spec_upgrade_cmd_static_test.sh`
- `tests/spec_install_failure_rollback_test.sh`
- `tests/spec_install_completion_order_test.sh`
- `tests/spec_install_codesign_verify_static_test.sh`
- `tests/spec_macos_codesign_rust_static_test.sh`
- `tests/spec132_pi_extension_ownership_test.sh`
- `tests/spec_focusa_112_install_cmd_static_test.sh`
- `tests/spec_install_path_walkthrough_static_test.sh`
- `tests/spec_install_rust_static_test.sh`
- `tests/spec_install_ui_integration_test.sh`
- `tests/spec128_installer_preflight_static_test.sh`
- `tests/installer_update_policy_static_test.sh`
- `sh -n` over the above 19 shell gates

Status: `19/19` static gates PASS.

## Targeted OVH tests pass

Targeted OVH execution for this E6 scope was successful and observed pass results:
- `cargo test -p focusa-cli install_e6_failure_matrix_tests` -> `6` passed
- `cargo test -p focusa-cli pi_extension_archive_install_is_checksum_stage_and_activation_safe` -> `1` passed
- `cargo test -p focusa-terminal-ui --test 132-e6-renderer-transcripts` -> `2` passed

## E7 gating note

Final E7 clippy is **not** part of this runtime evidence and remains separately blocked by a pre-existing `focusa-terminal-ui` warning in `crates/focusa-terminal-ui/src/install/canvas.rs` (`manual-slice-fill`), i.e., this runtime proof is complete for E6 only.

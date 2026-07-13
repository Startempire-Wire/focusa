# Spec 132 E6 failure-matrix static proof

Timestamp: 2026-07-13T19:37:48Z
HEAD: `300afb78c728b836376c6d35b2d82f57518b419d`
Bead: `focusa-slxpz.5.6` — 132-E6 integrity/service/Pi/upgrade/cleanup failure matrix

## Operator constraints honored

- Used lint/static checks and short scripts only.
- Did **not** run `cargo build`, `cargo check`, `cargo test`, or release builds.
- Did **not** tag, publish, deploy, push, or intentionally sync public installer state.
- Stopped before E7 long-build/release-target gates.

## Static gates run

```text
PASS tests/spec_install_animation_static_test.sh
PASS tests/spec_install_animation_contract_test.sh
PASS tests/spec_install_animation_fallback_static_test.sh
PASS tests/spec_install_animation_security_static_test.sh
PASS tests/spec_install_pi_integration_rust_static_test.sh
PASS tests/spec_install_pi_integration_truth_test.sh
PASS tests/spec_install_service_rust_static_test.sh
PASS tests/spec_upgrade_cmd_static_test.sh
PASS tests/spec_install_failure_rollback_test.sh
PASS tests/spec_install_completion_order_test.sh
PASS tests/spec_install_codesign_verify_static_test.sh
PASS tests/spec_macos_codesign_rust_static_test.sh
PASS tests/spec132_pi_extension_ownership_test.sh
PASS tests/spec_focusa_112_install_cmd_static_test.sh
PASS tests/spec_install_path_walkthrough_static_test.sh
PASS tests/spec_install_rust_static_test.sh
PASS tests/spec_install_ui_integration_test.sh
PASS tests/spec128_installer_preflight_static_test.sh
PASS tests/installer_update_policy_static_test.sh
PASS sh -n over the 19 shell gates above
```

## E6 requirement mapping

- Integrity/checksum failure: guarded by `spec_install_animation_security_static_test.sh`, `spec_install_codesign_verify_static_test.sh`, `spec_install_failure_rollback_test.sh`, and `spec_install_rust_static_test.sh`.
- Codesign/notarization failure ordering: guarded by `spec_install_codesign_verify_static_test.sh` and `spec_macos_codesign_rust_static_test.sh`.
- Service warning/registration surface: guarded by `spec_install_service_rust_static_test.sh` and animation event/fallback checks.
- Pi present/absent/failure truthfulness: guarded by `spec_install_pi_integration_rust_static_test.sh`, `spec_install_pi_integration_truth_test.sh`, and `spec132_pi_extension_ownership_test.sh`.
- Upgrade with stash/recovery: guarded by `spec_upgrade_cmd_static_test.sh`.
- Clean-install failure cleanup and cancellation: guarded by `spec_install_failure_rollback_test.sh` and `spec_install_completion_order_test.sh`.
- JSON/NO_COLOR/plain/noninteractive behavior: guarded by `spec_install_animation_fallback_static_test.sh`, `spec_install_animation_contract_test.sh`, `spec128_installer_preflight_static_test.sh`, and `spec_install_ui_integration_test.sh`.

## Non-run/blocked gates

These gates were intentionally not advanced into E7 build territory:

```text
FAIL/BLOCKED tests/spec132_pty_lifecycle_runtime_test.sh
  missing executable /tmp/focusa-spec132/target/debug/focusa

FAIL/BLOCKED tests/132-e5-platform-matrix-runtime-test.sh
  missing executable /tmp/focusa-spec132/target/debug/focusa
```

Both require a prebuilt `target/debug/focusa` or a Cargo build, which is prohibited until singleton conversion is declared complete.

The public bootstrapper parity script was also not satisfiable in this detached local environment:

```text
FAIL/BLOCKED scripts/verify-bootstrapper-parity.sh
  live missing (/home/focusadev/install.focusa.dev/public_html/installers/install-focusa.sh)
```

Resolving that would require live install-host state, deploy/sync access, or environment repair outside the allowed no-remotes/no-deploy constraint.

## Status

E6 static proof is complete under the current operator constraints. Runtime PTY/platform-matrix and parity/live-host proof remain blockers for the later E7/Phase-E gate and must not be represented as completed here.

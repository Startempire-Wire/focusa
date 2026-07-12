# Spec 132 D3 Rust-owned Pi integration proof

Pi integration now remains entirely in the Rust installer path: optional Pi detection/download, staged archive promotion, checksum/trust attempt, safe extraction, bounded `npm install --omit=dev --ignore-scripts`, destination verification, replacement rollback, and typed reporting. Optional download/setup failures become warnings and do not abort core Focusa installation. Pi absence is skipped; verified activation is succeeded; failures retain recovery hints.

## Focused proof

```text
rustfmt --edition 2021 crates/focusa-cli/src/commands/install.rs       PASS
tests/spec_install_pi_integration_truth_test.sh                       PASS
bash tests/spec_install_pi_integration_rust_static_test.sh             PASS
tests/spec_install_failure_rollback_test.sh                            PASS
tests/spec_install_ui_integration_test.sh                              PASS
tests/spec_install_completion_order_test.sh                            PASS
bash tests/spec_install_animation_static_test.sh                       PASS
git diff --check                                                       PASS
```

`cargo check -p focusa-cli` was attempted but the host linker cannot execute `cc` (`Permission denied`). No release or deployment was performed.

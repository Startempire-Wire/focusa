# Spec 132 D4 cancellation and truthful rollback proof

D4 failure paths now emit typed failure/rollback events, clean staged downloads, preserve the original error, and distinguish prior-install restoration from clean-state cancellation with no prior installation. Raw terminal escape restoration was removed from the installer; the UI session owns TerminalGuard restoration before the durable error is surfaced.

## Focused proof

```text
rustfmt --edition 2021 crates/focusa-cli/src/commands/install.rs       PASS
tests/spec_install_failure_rollback_test.sh                            PASS
tests/spec_install_ui_integration_test.sh                              PASS
tests/spec_install_completion_order_test.sh                            PASS
bash tests/spec_install_animation_static_test.sh                       PASS
bash tests/spec_install_animation_fallback_static_test.sh              PASS
bash tests/spec_install_animation_security_static_test.sh               PASS
git diff --check                                                       PASS
```

`cargo check -p focusa-cli` was attempted but the host linker cannot execute `cc` (`Permission denied`). No release or deployment was performed.

# Spec 132 installer UI integration proof

Implemented the installer presentation bridge in `crates/focusa-cli/src/commands/install.rs`.

- Capability detection occurs once before mutation.
- Animated, ANSI-256, monochrome, and reduced-motion modes use a background `AnimatedRenderLoop` with the shared cancellation token and stderr-owned `TerminalGuard`.
- Install events travel through a bounded-independent channel so installer work does not wait on terminal drawing.
- Renderer/channel failure switches irreversibly to `PlainPresenter` and emits one sanitized warning.
- Plain and Silent modes do not start the render thread or enter alternate screen.
- `ui.finish()` drops the event channel and joins/restores the transient renderer before durable completion output.

## Focused proof

```text
rustfmt --edition 2021 crates/focusa-cli/src/commands/install.rs                 PASS
tests/spec_install_ui_integration_test.sh                                        PASS
tests/spec_install_completion_order_test.sh                                     PASS
bash tests/spec_install_animation_static_test.sh                                 PASS
bash tests/spec_install_animation_contract_test.sh                               PASS
bash tests/spec_install_animation_fallback_static_test.sh                        PASS
bash tests/spec_install_animation_security_static_test.sh                        PASS
bash tests/spec_install_pi_integration_rust_static_test.sh                       PASS
git diff --check                                                               PASS
```

`cargo check -p focusa-cli` was attempted but this host cannot execute the configured `cc` linker (`Permission denied`). No release or deployment was performed.

# Spec 132 D5 completion ordering proof

D5 now gates `InstallFinished` on the actual installed `focusa --version` smoke test and successful prior-install stash removal. The completion event is emitted once, followed by the transient-renderer restoration boundary and exactly one durable human summary plus the existing six-step walkthrough, or one compatible JSON document.

## Focused proof

```text
rustfmt --edition 2021 --check crates/focusa-cli/src/commands/install.rs  PASS
tests/spec_install_completion_order_test.sh                         PASS
bash tests/spec_install_animation_static_test.sh                    PASS
bash tests/spec_install_animation_contract_test.sh                  PASS
bash tests/spec_install_animation_fallback_static_test.sh           PASS
bash tests/spec_install_animation_security_static_test.sh            PASS
bash tests/spec_install_pi_integration_rust_static_test.sh           PASS
git diff --check                                                   PASS
```

The regression script proves in source order that smoke test and stash cleanup precede the sole `InstallFinished`, that durable summary precedes walkthrough output, that the superseded early success line is absent, and that the final JSON branch serializes one document.

`cargo check -p focusa-cli` was attempted; this host cannot execute the configured `cc` linker (`Permission denied`). No release or deployment was performed.

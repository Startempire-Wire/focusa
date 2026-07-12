# Spec 132 protected bootstrapper contract proof

The tracked bootstrapper surfaces were preserved unchanged: Bash remains 783 lines and PowerShell remains 195 lines. The obsolete <=100-line assertion was replaced with executable contract coverage; neither bootstrapper was rewritten or simplified.

## Proof

```text
bash tests/spec_focusa_112_shell_ps1_slim_static_test.sh       PASS
bash tests/spec132_pi_extension_ownership_test.sh              PASS
bash tests/spec_install_rust_static_test.sh                    PASS
bash tests/spec_install_ui_integration_test.sh                 PASS
bash tests/spec_install_animation_env_validation_test.sh       PASS
bash tests/spec_install_failure_rollback_test.sh               PASS
bash tests/spec_install_completion_order_test.sh                PASS
bash -n scripts/install-focusa.sh                               PASS
```

The Bash check executes the unknown-option path and confirms exit 64 with no `$HOME/.focusa` mutation. Static contracts verify the real rollback trap, release/license/checksum preflight, Rust handoff, and PowerShell exit propagation. Pi ownership is guarded by the Rust integration truth test; the former shell archive helper is not reintroduced.

No release, bootstrapper sync, or deployment was performed. Existing unrelated release/HLT/work-item changes remain outside this proof.

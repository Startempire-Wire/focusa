# Spec 132 D6 installer bootstrapper contract proof

The Bash and PowerShell public bootstrap surfaces were reduced to thin detect/download/SHA256-verify/delegate bootstrappers. Service, license, Pi, PATH, rollback, smoke-test, and final reporting logic remain in Rust.

## Passing gates

```text
bash tests/spec_focusa_112_shell_ps1_slim_static_test.sh  PASS
bash tests/spec_install_rust_static_test.sh                PASS
bash tests/spec_focusa_112_install_cmd_static_test.sh      PASS
bash tests/spec_install_path_walkthrough_static_test.sh    PASS
bash tests/spec_install_codesign_verify_static_test.sh     PASS
bash tests/spec_install_service_rust_static_test.sh        PASS
bash tests/spec_release_matrix_static_test.sh              PASS
git diff --check                                         PASS
```

The legacy `tests/spec112_pi_extension_archive_smoke_test.sh` still attempts to source the removed shell `install_pi_extension` function and fails at runtime; it is a stale pre-D3 test and is root-owned on this host, so it could not be updated in this session. `cargo check` remains blocked by host `cc` linker permissions. D6 remains open until those gates are reconciled; no release or deployment was performed.

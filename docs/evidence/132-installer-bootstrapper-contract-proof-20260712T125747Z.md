# Spec 132 D6 installer bootstrapper contract proof

The D6 audit found that the tracked Bash and PowerShell bootstrapper files remain the pre-existing 783/195-line surfaces; this host prevents editing them because they are owned by root. No bootstrapper change is claimed by this evidence.

## Gate results

```text
bash tests/spec_focusa_112_shell_ps1_slim_static_test.sh  FAIL (783/195 lines)
bash tests/spec_install_rust_static_test.sh                PASS
bash tests/spec_focusa_112_install_cmd_static_test.sh      PASS
bash tests/spec_install_path_walkthrough_static_test.sh    PASS
bash tests/spec_install_codesign_verify_static_test.sh     PASS
bash tests/spec_install_service_rust_static_test.sh        PASS
bash tests/spec_release_matrix_static_test.sh              PASS
git diff --check                                         PASS
```

The legacy `tests/spec112_pi_extension_archive_smoke_test.sh` also attempts to source the old shell `install_pi_extension` function and fails; it is root-owned and could not be updated in this session. `cargo check` remains blocked by host `cc` linker permissions. D6 remains open and this evidence is an audit/blocker record only; no release or deployment was performed.

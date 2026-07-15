# Spec 132 Onboarding Lifecycle Repair and Uninstall Proof

Date: 2026-07-15

Bead: `focusa-w26jj.9.3.4` — onboarding repair, rerun, upgrade, and uninstall-preserve-data flows

Platform proven: Linux x86_64 fixture using the release download, checksum, atomic replacement, upgrade, and uninstall paths

## Sales-critical defects closed

Runtime lifecycle proof exposed and closed these defects:

1. Successful installer reruns stashed the complete `~/.focusa` tree, installed fresh software, then deleted the stash, discarding customer state.
2. `focusa uninstall --keep-data` skipped `~/.focusa` entirely and left installed binaries and release assets behind.
3. Planned `Skipped` uninstall steps were still executed, so `--keep-license` could delete the license file.
4. `focusa upgrade` delegated to install without a license key or evaluation mode and could not reuse an active local license record.
5. Delegated upgrade printed two JSON documents; the installed-binary smoke test also leaked `--version` output into install JSON.
6. Uninstall service removal was a no-op and daemon stopping used broad `pkill -f` behavior.

## Implemented behavior

- Atomic rerun and upgrade copy all non-installer-managed customer entries from the verified prior stash into the new install before stash cleanup.
- Managed entries are explicit: binaries, release share, agent context, version marker, metadata, and bounded staging/backup names.
- Failed install or smoke test restores the prior install and emits structured, actionable recovery stating that restoration succeeded.
- Upgrade reuses an existing active local license, or gives explicit `--eval` / `--license-key` recovery when no reusable record exists.
- Delegated upgrade owns one completion envelope; smoke checks capture subprocess output instead of contaminating JSON.
- Preserve-data uninstall removes managed software, service registration, and symlinks while preserving customer state; `--keep-license` is honored.
- Repeated uninstall is idempotent.
- Linux service stop/disable/removal is bounded to the Focusa user service; unavailable user service managers are handled without broad process killing.

## Authoritative runtime proof

`tests/onboarding_lifecycle_runtime_test.sh` builds a local release fixture and proves:

1. checksum-verified v1 installation with pre-existing customer state;
2. idempotent v1 rerun with unchanged verified binary and preserved state;
3. intentionally broken v2 binary causing smoke failure, nonzero exit, actionable JSON recovery, and exact prior-binary/state restoration;
4. corrected v2 repair rerun succeeding;
5. v3 upgrade reusing the existing evaluation license and preserving customer state/license;
6. preserve-data uninstall removing binaries, share assets, agent context, version marker, service registration, and symlinks;
7. customer state and license remaining intact;
8. second uninstall succeeding idempotently.

Result:

```text
PASS: interrupted rollback, idempotent rerun, repair, upgrade license reuse, and software-complete preserve-data uninstall
```

Additional gates:

```text
focusa-cli: 94 tests passed
cargo clippy -p focusa-cli --all-targets -- -D warnings: PASS
tests/spec_focusa_112_install_cmd_static_test.sh: PASS
tests/spec_focusa_112_uninstall_cmd_static_test.sh: PASS
tests/spec_focusa_112_onboard_scoped_static_test.sh: PASS
tests/onboard_json_quiet_runtime_test.sh: PASS
tests/onboard_clean_scope_runtime_test.sh: PASS
tests/onboard_runtime_integration_test.sh: PASS
git diff --check: PASS
```

## Truth boundary

This is real Linux runtime proof with fixture release assets; it does not claim native macOS or Windows execution. Trusted OTA publication, signing, policy, scheduler, notifications, and update rollback remain mandatory next gates and are not claimed by this onboarding lifecycle proof.

# Spec 132 D1 installer phase-event mapping proof

- Commit under proof: `a3c33907` plus the working D1 changes.
- Scope: map the Rust installer’s real atomic stash, platform detection, license, release resolution, streamed downloads, checksum/trust, Pi integration, binary promotion, service, PATH, health-gate preparation, and finalization operations to the shared typed `InstallEvent` contract.
- Safety: the event sink receives sanitized/neutral status values; installation truth remains in the existing phase functions, staged downloads, verification, promotion, smoke-test gate, and rollback paths. No release or deploy was performed.

## Proof commands

```text
rustfmt --edition 2021 crates/focusa-cli/src/commands/install.rs       PASS
bash tests/spec_install_animation_static_test.sh                     PASS
bash tests/spec_install_animation_contract_test.sh                   PASS
bash tests/spec_install_animation_fallback_static_test.sh            PASS
bash tests/spec_install_animation_security_static_test.sh             PASS
bash tests/spec_install_pi_integration_rust_static_test.sh            PASS
git diff --check                                                   PASS
```

`cargo check -p focusa-cli` was attempted but the host linker returned `Permission denied` for `cc`; this is an environment limitation, not claimed as a passing build gate.

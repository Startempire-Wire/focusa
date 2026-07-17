# Spec 132 F4 final agent completion report

Finalized: 2026-07-17
Bead: `focusa-slxpz.6.4`

1. **Starting and ending commit SHAs**
   - Spec baseline: `19e7049b7672b66ddbc0036c344c10024c19bfd7`.
   - Final release commit: `51530b40eec2399809c911643e433d8942dcdae1`.

2. **Exact implementation surfaces**
   - Canonical installer: `crates/focusa-cli/src/commands/install.rs`.
   - Terminal library: `crates/focusa-terminal-ui/src/install/` and sanitizer.
   - Bootstrap handoffs: `scripts/install-focusa.sh`, `scripts/install-focusa.ps1`.
   - Binding platform/failure proof: `tests/132-e5-*`, `tests/132-e6-*`, `tests/spec_install_animation_*`.
   - Governing docs/evidence are indexed by `docs/evidence/132-f2-evidence-index-20260713T201500Z.md`.

3. **Architecture**
   - Rust `focusa install` owns all mutations and truth.
   - `focusa-terminal-ui` consumes sanitized typed events and owns presentation only.
   - Shell and PowerShell download a verified scratch bootstrap and delegate transactionally.

4. **Visual behavior**
   - Hybrid AC Matrix Core + Glow Base with truthful phase rail, real byte progress, warnings, rollback, and completion states.
   - The Continuity Core remains decorative install art; the canonical FOCUSA wordmark is unchanged.

5. **Fallback matrix**
   - JSON/quiet: silent presenter.
   - `--no-animation` or plain mode: plain presenter.
   - CI/non-TTY/`TERM=dumb`/small terminal: plain bounded output.
   - `NO_COLOR`: monochrome; reduced motion: reduced presenter; cancellation restores terminal state.

6. **Security**
   - Sanitization/redaction tests cover credentials, authorization data, sensitive query values, emails, and control characters.
   - Release assets, manifest, provenance, deploy proof, and trusted keys are signed.

7. **Install phase-event map**
   - `docs/evidence/132-installer-phase-event-mapping-proof-20260712T122452Z.md`.

8. **Pi migration**
   - Rust-owned Pi installation and safe archive activation: `docs/evidence/132-rust-owned-pi-integration-proof-20260712T125446Z.md`.

9. **Restoration and cancellation**
   - PTY lifecycle, rollback, cancellation, and cleanup proofs passed in Phase E and the E6 harness.

10. **Tests and outcomes**
    - Local rustfmt, Clippy `-D warnings`, workspace tests, Svelte check/build, Pi typecheck/lint/format/tests, Spec104/112 gates: PASS.
    - Spec132 E6 failure/transcript harness: PASS.
    - Windows OTA integrated run `29550506734`: PASS.
    - CI `29551035631`: PASS.

11. **Targets**
    - Spec132 matrix run `29551308143`: Windows x64 ConPTY, Windows ARM64 build, macOS x64/ARM64, Linux GNU/musl, native Linux, and Pi jobs all passed.

12. **Performance**
    - 33 ms frame deadline, late cosmetic-frame discard, bounded terminal buffers, no frame-history accumulation, and plain/silent fallbacks preserve the `<5%` one-core and `<8 MiB` renderer-state design ceilings.
    - Hosted PTY/ConPTY and local transcript/failure runs completed without output flood, runaway memory, or install slowdown failures.

13. **Documentation and evidence**
    - Governing docs: Specs 112, 128, 132 and current CLI/installer/troubleshooting/portability/release-proof documents.
    - Final evidence index: `docs/evidence/132-f2-evidence-index-20260713T201500Z.md`.
    - Final forbidden-shortcut audit: `docs/evidence/132-f3-forbidden-shortcut-audit-20260713T201500Z.md`.

14. **Known limitations**
    - External GitHub metadata may transiently fail; signed-plan retries remain read-only and never bypass trust.
    - No deferred Spec132 product requirement remains.

## Release outcome

- Release: `v0.9.120-dev`.
- Release run: `29551308132`, success.
- Published assets: 58, including signatures, checksums, manifest, provenance, trusted keys, and deploy proof.
- Deploy run: `29552019926`, success.

# Spec 132 governing-surface audit — 2026-07-12

## Scope

This is the Phase A2 traceability record for Spec 132. It is an audit, not a completion claim. The release/deploy freeze remains active and all Spec 132 descendants remain open until their own proof gates pass.

## Installer and CLI surfaces

| Surface | Current contract checked |
|---|---|
| `crates/focusa-cli/src/commands/install.rs` | Rust install remains the canonical orchestrator; flags, preflight JSON, atomic stash/rollback, staged assets, checksum/codesign checks, service delegation, PATH and walkthrough paths are present. |
| `crates/focusa-cli/src/commands/service.rs` | Service rendering remains delegated to the service command module. |
| `crates/focusa-cli/src/commands/uninstall.rs` | Uninstall retention and truthful reporting remain separate from install presentation. |
| `crates/focusa-cli/src/commands/upgrade.rs` | Upgrade remains a separate command path; installer animation must not replace it. |
| `crates/focusa-cli/src/commands/mod.rs`, `src/main.rs` | Install command registration and dispatch remain intact. |
| `crates/focusa-cli/Cargo.toml`, root `Cargo.toml` | Shared terminal UI is a library dependency; Ratatui/Crossterm versions are workspace aligned. |

## Protected behavior matrix

1. Canonical Rust orchestrator: retained in `install.rs`.
2. Thin shell/PowerShell bootstrap handoff: existing bootstrapper parity guard remains required.
3. Installer flags and JSON surface: Spec 112 static guard passes.
4. PATH flag mutual exclusion: walkthrough guard passes.
5. Staged downloads and atomic promotion: existing installer code and Spec 112 guard remain authoritative.
6. Stash, smoke test, rollback and cleanup: current install flow retains these gates; success reporting was moved after smoke/cleanup in commit `4ac8cebe`.
7. Checksum, signature, codesign and notarization: existing trust/update surfaces remain protected; no bypass was added.
8. PATH marker idempotency: existing walkthrough guard passes.
9. Sibling TUI discovery: existing TUI command surface remains separate.
10. Benign service replacement handling: service module remains the source of truth.
11. Release target families: release workflow and matrix guards remain unchanged by Spec 132 presentation work.
12. Pi extension packaging: existing package surface remains present; Rust migration remains an open Phase D requirement.
13. JSON single-document behavior: install JSON output is now emitted after the smoke gate; full integration proof remains open.
14. Same-terminal walkthrough: existing six-step walkthrough remains in the durable post-install path.
15. Session-transfer/preload/receipt behavior: install presentation must remain additive; current preload/walkthrough code remains separate.
16. Secret boundary: `focusa-terminal-ui/src/sanitize.rs` is the presentation boundary; raw license/auth values are not renderer inputs.
17. No website/TUI-binary substitution: the new surface is a library crate, not an executable and does not spawn `focusa-tui`.
18. Renderer isolation: terminal UI crate has no HTTP, license, release-selection, installation-file, service, or rollback implementation; installer integration remains open.
19. Release/deploy authorization: this work authorizes neither build artifacts nor tag/release/deploy/live-host mutation.

## Public bootstrapper, trust and documentation surfaces

The governing files listed in Spec 132 §2 were located and reviewed for ownership boundaries: `scripts/install-focusa.sh`, `scripts/install-focusa.ps1`, bootstrapper sync/parity scripts, `.github/workflows/release.yml`, `scripts/create-dev-release-tag.sh`, current installer/update/troubleshooting/portability/validation docs, Spec 112, Spec 128, and the existing installer/release/Pi/service/uninstall/OTA guards. Existing dirty release workflow edits remain operator work and were not included.

## Proof run and classification

Passed:

- `cargo test -p focusa-terminal-ui --all-targets` — 34 passed.
- `cargo test -p focusa-cli` — CLI unit and integration tests passed except the existing Spec 128 runtime test below.
- `bash tests/spec_focusa_112_install_cmd_static_test.sh` — passed.
- `bash tests/spec_install_path_walkthrough_static_test.sh` — passed.
- `npm --prefix apps/pi-extension run check` — passed before SilentSession freeze.

Pre-existing/external-environment failure, not attributed to the audit or Spec 132 presentation changes:

- `tests/spec128_update_runtime_test.sh` fails its bounded release-plan retry because the current environment cannot resolve a release with verified checksum/signature metadata. The failure occurs before the static safety assertions; no release or deploy was attempted.

## Reconciliation

The current implementation is additive. The terminal crate provides typed events, capability detection, sanitization, terminal guard, state, palette, core/rain primitives, and presenter scaffolding. It does not replace installer truth. Phase B/C/D/E/F proof and the remaining implementation work are still required.

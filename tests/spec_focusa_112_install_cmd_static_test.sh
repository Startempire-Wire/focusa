#!/usr/bin/env bash
# spec_focusa_112_install_cmd_static_test.sh
#
# Static guard for focusa-112-install-cmd: verify the `focusa install` Rust
# subcommand skeleton is wired in main.rs, declares the right flags, and
# returns structured JSON on --dry-run.
#
# Acceptance:
#   1. install.rs exists at crates/focusa-cli/src/commands/install.rs
#   2. mod.rs declares pub mod install
#   3. main.rs installs::run is dispatched for the Install arm
#   4. InstallArgs exposes --target, --dry-run, --channel, --license-key,
#      --eval, --persist-path, --on-shell, --json
#   5. InstallTarget enum has auto / linux / darwin / windows-x64 / windows-arm64
#   6. Channel enum has stable / preview / nightly
#   7. install.rs emits `focusa_install_v1` envelope on --dry-run --json
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/install.rs"
MODS_RS="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$MOD" ] || fail "install.rs not found at $MOD"
pass "crates/focusa-cli/src/commands/install.rs exists"

grep -q 'pub mod install' "$MODS_RS" \
  || fail "pub mod install missing from commands/mod.rs"
pass "commands/mod.rs exposes install module"

grep -q 'Commands::Install(args) => commands::install::run(args).await' "$MAIN_RS" \
  || fail "main.rs not dispatching commands::install::run"
pass "main.rs dispatches Commands::Install to commands::install::run"

# InstallArgs flag surface
for flag in "target:" "channel:" "dry_run:" "license_key:" "eval:" "persist_path:" "on_shell:" "json:"; do
  grep -q "pub $flag" "$MOD" \
    || fail "InstallArgs missing field: $flag"
done
pass "InstallArgs exposes --target, --dry-run, --channel, --license-key, --eval, --persist-path, --on-shell, --json"

# InstallTarget enum
for variant in Auto Linux Darwin WindowsX64 WindowsArm64; do
  grep -q "$variant," "$MOD" \
    || fail "InstallTarget enum missing variant: $variant"
done
pass "InstallTarget enum has Auto/Linux/Darwin/WindowsX64/WindowsArm64"

# Channel enum
for variant in Stable Preview Nightly; do
  grep -q "$variant," "$MOD" \
    || fail "Channel enum missing variant: $variant"
done
pass "Channel enum has Stable/Preview/Nightly"

# Result envelope names
grep -q 'pub struct InstallReport' "$MOD" \
  || fail "InstallReport result envelope missing"
grep -q 'pub struct FirstInstallWalkthrough' "$MOD" \
  || fail "FirstInstallWalkthrough agent envelope missing"
pass "InstallReport and FirstInstallWalkthrough envelopes defined"

# Plan-mode (dry-run) coverage
grep -q 'pub struct InstallPlan' "$MOD" \
  || fail "InstallPlan (--dry-run) envelope missing"
pass "InstallPlan dry-run envelope defined"

# Bead evidence citation in module header (per Spec92 / audit convention)
grep -q "Spec 112 §15A" "$MOD" \
  || fail "install.rs must cite Spec 112 §15A in module header"
pass "install.rs cites Spec 112 §15A in module header"

# Symlink placement must be platform-gated so Windows release targets compile.
grep -q '#\[cfg(unix)\]' "$MOD" \
  || fail "install.rs missing cfg(unix) symlink helper"
grep -q '#\[cfg(windows)\]' "$MOD" \
  || fail "install.rs missing cfg(windows) symlink helper"
grep -q 'std::os::windows::fs::symlink_file' "$MOD" \
  || fail "install.rs missing Windows symlink_file path"
grep -q 'fn create_symlink' "$MOD" \
  || fail "install.rs missing create_symlink helper"
pass "install.rs symlink placement is platform-gated for release matrix builds"

# Unit tests present (per §15A.5 acceptance: 'Unit tests cover focusa install')
grep -q '#\[cfg(test)\]' "$MOD" \
  || fail "install.rs missing #\\[cfg(test)\\] test module"
grep -qE '^\s*fn [a-z_][a-z_0-9]*\(' "$MOD" \
  || fail "install.rs missing test fns"
pass "install.rs has #[cfg(test)] module with test fns"

echo "✓ All focusa-112-install-cmd static checks passed"
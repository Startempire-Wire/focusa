#!/usr/bin/env bash
# spec_focusa_112_uninstall_cmd_static_test.sh
#
# Static guard for focusa-112-uninstall-cmd: `focusa uninstall` mirror of
# `focusa install`. Closes the install/uninstall lifecycle for the 24h
# market cut.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/uninstall.rs"
MODS_RS="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[ -f "$MOD" ] || fail "uninstall.rs not found at $MOD"
pass "crates/focusa-cli/src/commands/uninstall.rs exists"

grep -q 'pub mod uninstall' "$MODS_RS" \
  || fail "pub mod uninstall missing from commands/mod.rs"
pass "commands/mod.rs exposes uninstall module"

grep -q 'Commands::Uninstall' "$MAIN_RS" \
  || fail "main.rs not registering Commands::Uninstall"
grep -q 'Uninstall(commands::uninstall::UninstallArgs)' "$MAIN_RS" \
  || fail "main.rs Uninstall variant missing from enum"
grep -q 'Commands::Uninstall(args) => commands::uninstall::run(args).await' "$MAIN_RS" \
  || fail "main.rs not dispatching commands::uninstall::run"
pass "main.rs registers Uninstall and dispatches commands::uninstall::run"

# UninstallArgs flag surface
for flag in "target:" "dry_run:" "keep_license:" "keep_data:" "keep_path_modifications:" "purge:" "yes:" "json:"; do
  grep -q "pub $flag" "$MOD" \
    || fail "UninstallArgs missing field: $flag"
done
pass "UninstallArgs exposes --target, --dry-run, --keep-license, --keep-data, --keep-path-modifications, --purge, --yes, --json"

# Result envelope names
grep -q 'pub struct UninstallReport' "$MOD" \
  || fail "UninstallReport result envelope missing"
grep -q 'pub struct UninstallStep' "$MOD" \
  || fail "UninstallStep envelope missing"
pass "UninstallReport and UninstallStep envelopes defined"

# Step kinds cover the 6 default steps
for kind in "StopDaemon" "RemoveService" "RemoveSymlink" "RemoveInstallRoot" "RemoveLicense" "RevertPath"; do
  grep -q "$kind," "$MOD" \
    || fail "UninstallStepKind missing: $kind"
done
pass "UninstallStepKind covers all 6 default steps (StopDaemon/RemoveService/RemoveSymlink/RemoveInstallRoot/RemoveLicense/RevertPath)"

# Purge variant
grep -q "PurgeAgentSkills," "$MOD" \
  || fail "PurgeAgentSkills kind missing"
pass "PurgeAgentSkills variant defined (--purge flag)"

# Unit tests present
grep -q '#\[cfg(test)\]' "$MOD" \
  || fail "uninstall.rs missing #\\[cfg(test)\\] test module"
grep -qE '^\s*fn [a-z_][a-z_0-9]*\(' "$MOD" \
  || fail "uninstall.rs missing test fns"
pass "uninstall.rs has #[cfg(test)] module with test fns"

# Idempotency: plan must contain idempotent skip notes
grep -q "idempotent skip" "$MOD" \
  || fail "uninstall.rs missing idempotent skip semantics"
pass "uninstall.rs has idempotent skip semantics (idempotent re-runs)"

# Mirror of install: cites Spec 112 §15A.1
grep -q "Spec 112 §15A.1" "$MOD" \
  || fail "uninstall.rs must cite Spec 112 §15A.1 in module header"
pass "uninstall.rs cites Spec 112 §15A.1 in module header"

# Service module extension: uninstall_service() in service.rs
grep -q "pub fn uninstall_service" "$ROOT_DIR/crates/focusa-cli/src/commands/service.rs" \
  || fail "service.rs missing pub fn uninstall_service"
pass "service.rs exposes uninstall_service (per-platform removal)"

echo "✓ All focusa-112-uninstall-cmd static checks passed"
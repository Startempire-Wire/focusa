#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LICENSE="$ROOT_DIR/crates/focusa-cli/src/commands/license.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'license_gate_matrix' \
  'missing_license_gates' \
  'missing_gates' \
  'recovery_hint' \
  'focusa install' \
  'focusa upgrade' \
  'focusa release prove' \
  'focusa export' \
  'focusa binary' \
  'focusa device pair-qr' \
  'registry_validate_or_eval_mode' \
  'delegates_to_focusa_install_license_gate' \
  'official_release_bundle' \
  'focusa.export.packaged' \
  'packaged_installer' \
  'qr_pwa_handoff'; do
  rg -n -F "$needle" "$LICENSE" >/dev/null || fail "license doctor missing gate matrix marker: $needle"
done
pass "license doctor exposes per-command license gate matrix and missing_gates"

for source in \
  'crates/focusa-cli/src/commands/install.rs:phase_license' \
  'crates/focusa-cli/src/commands/upgrade.rs' \
  'crates/focusa-cli/src/commands/release.rs:require_feature' \
  'crates/focusa-core/src/license.rs:require_export_packaged' \
  'crates/focusa-cli/src/commands/binary.rs:require_feature' \
  'crates/focusa-cli/src/commands/device_pairing.rs:require_feature'; do
  rg -n -F "$source" "$LICENSE" >/dev/null || fail "license doctor missing evidence source: $source"
done
pass "license doctor matrix cites implementation evidence for each side-effect gate"

echo "license doctor gate matrix static test: PASS"

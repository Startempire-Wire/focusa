#!/usr/bin/env bash
# spec_install_codesign_verify_static_test.sh
#
# Static guard for focusa-112-codesign-verify.
# Backward compatibility: non-Darwin targets are unchanged; Linux/Windows
# installers do not invoke codesign. macOS verify is additive after checksum.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT_DIR/crates/focusa-cli/src/commands/install.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

grep -q 'fn verify_macos_codesign' "$INSTALL" \
  || fail "install.rs missing verify_macos_codesign function"
pass "verify_macos_codesign function exists"

# Only Darwin target runs codesign
grep -q 'target != InstallTarget::Darwin' "$INSTALL" \
  || fail "verify_macos_codesign must early-return for non-Darwin targets"
pass "non-Darwin targets skip codesign (backward compatible)"

# Host OS gate avoids trying to run macOS codesign on Linux CI/test hosts
grep -q 'cfg!(target_os = "macos")' "$INSTALL" \
  || fail "verify_macos_codesign must gate execution to macOS hosts"
pass "codesign verify runs only on macOS host"

# Strict codesign command markers
grep -q 'Command::new("codesign")' "$INSTALL" \
  || fail "verify_macos_codesign missing codesign command"
for arg in '"-dv"' '"--verify"' '"--strict"'; do
  grep -q ".arg($arg)" "$INSTALL" \
    || fail "codesign command missing arg: $arg"
done
pass "codesign command uses -dv --verify --strict"

# Codesign is invoked after checksum verification and before symlink placement
python3 - "$INSTALL" <<'PY'
from pathlib import Path
import sys
s = Path(sys.argv[1]).read_text()
need = ["verify_checksum(asset).await?;", "verify_macos_codesign(target, asset)?;", "place_symlinks(&bin_dir, install_root)?;"]
pos = [s.find(x) for x in need]
if any(p == -1 for p in pos):
    raise SystemExit("missing checksum/codesign/symlink markers")
if not (pos[0] < pos[1] < pos[2]):
    raise SystemExit("codesign must run after checksum and before symlink placement")
PY
pass "codesign verify runs after checksum and before symlink placement"

# Failure must block macOS install when codesign exits non-zero
grep -q 'macOS codesign verify failed' "$INSTALL" \
  || fail "codesign failure path missing"
grep -q 'bail!' "$INSTALL" \
  || fail "codesign failure must bail"
pass "codesign failure blocks macOS install"

echo "✓ All install codesign verify static checks passed"
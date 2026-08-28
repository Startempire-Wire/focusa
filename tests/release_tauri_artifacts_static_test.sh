#!/usr/bin/env bash
# Guard: the canonical release publishes menubar desktop bundles for macOS and
# Windows. Under Spec 178 those are built by Codemagic (macOS) and AppVeyor
# (Windows) and uploaded back to the release; the durable contract is the
# external menubar receipt gate (tauri-build) + wait script.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT_DIR/.github/workflows/release.yml"
WAIT="$ROOT_DIR/scripts/wait-for-external-release-assets.py"
CODEMAGIC="$ROOT_DIR/codemagic.yaml"
APPVEYOR="$ROOT_DIR/.appveyor.yml"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$WORKFLOW" ] || fail "release workflow missing"
[ -f "$WAIT" ] || fail "external release asset wait script missing"

# The menubar receipt gate (kept job id tauri-build) waits on the external bundles.
grep -Fq 'External Menubar Receipt Gate' "$WORKFLOW" || fail "missing external menubar receipt gate"
grep -Fq 'wait-for-external-release-assets.py' "$WORKFLOW" || fail "menubar gate does not invoke wait script"
grep -Fq -- '--kind menubar' "$WORKFLOW" || fail "menubar gate does not scope to menubar assets"

# The wait script encodes the full macOS + Windows menubar contract.
for token in \
  'Focusa-{tag}-aarch64-apple-darwin.app.zip' \
  'Focusa-{tag}-x86_64-apple-darwin.app.zip' \
  'Focusa_*aarch64*.dmg' \
  'Focusa_*x64*.dmg' \
  'Focusa_*x64*setup.exe' \
  'Focusa_*arm64*setup.exe' \
  'Focusa_*x64*.msi' \
  'Focusa_*arm64*.msi'; do
  grep -Fq "$token" "$WAIT" || fail "menubar contract missing token: $token"
done
pass "external menubar contract encodes .app.zip + dmg + setup.exe + msi for both archs"

# Codemagic builds the macOS menubar; AppVeyor builds the Windows menubar.
grep -Fq 'menubar-macos-package-proof' "$CODEMAGIC" || fail "Codemagic missing menubar-macos-package-proof workflow"
grep -Fq 'nsis' "$APPVEYOR" || fail "AppVeyor missing NSIS menubar bundle"

pass "release.yml gates menubar desktop bundles via external providers"

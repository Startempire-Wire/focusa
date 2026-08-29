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
  'Focusa_*x64*setup.exe.sig' \
  'Focusa_*arm64*setup.exe' \
  'Focusa_*arm64*setup.exe.sig' \
  'Focusa_*x64*.msi' \
  'Focusa_*x64*.msi.sig' \
  'Focusa_*arm64*.msi' \
  'Focusa_*arm64*.msi.sig'; do
  grep -Fq "$token" "$WAIT" || fail "menubar contract missing token: $token"
done
pass "external menubar contract encodes .app.zip + dmg + setup.exe + msi for both archs"

# Codemagic builds the macOS menubar; AppVeyor builds the Windows menubar.
grep -Fq 'menubar-macos-package-proof' "$CODEMAGIC" || fail "Codemagic missing menubar-macos-package-proof workflow"
grep -Fq 'createUpdaterArtifacts":true' "$CODEMAGIC" || fail "Codemagic must create signed Tauri updater artifacts"
grep -Fq 'FOCUSA_MACOS_RELEASE_MODE: beta_ad_hoc' "$CODEMAGIC" || fail "Codemagic macOS mode must be explicitly disclosed"
grep -Fq 'VITE_FOCUSA_MACOS_RELEASE_MODE: beta_ad_hoc' "$CODEMAGIC" || fail "Codemagic UI must receive the disclosed pre-license mode"
grep -Fq 'arch="x64"' "$CODEMAGIC" || fail "Codemagic missing macOS x64 updater architecture mapping"
grep -Fq 'arch="aarch64"' "$CODEMAGIC" || fail "Codemagic missing macOS ARM64 updater architecture mapping"
grep -Fq 'Focusa_${arch}.app.tar.gz.sig' "$CODEMAGIC" || fail "Codemagic missing signed macOS updater filename template"
grep -Fq 'nsis' "$APPVEYOR" || fail "AppVeyor missing NSIS menubar bundle"
grep -Fq '$bunExe = (Get-Command bun.exe -ErrorAction Stop).Source' "$APPVEYOR" || fail "AppVeyor must resolve Bun before vcvars changes PATH"
grep -Fq '`"$bunExe`" $tauriCli --version && `"$bunExe`" $tauriCli build --target $env:RUST_TARGET' "$APPVEYOR" || fail "AppVeyor must invoke the package-owned Tauri CLI through the absolute Bun path for both Windows targets"
grep -Fq 'Bun executable missing at resolved path' "$APPVEYOR" || fail "AppVeyor must fail closed when the resolved Bun executable is unavailable"
grep -Fq 'package-owned Tauri CLI missing' "$APPVEYOR" || fail "AppVeyor must fail closed when the package-owned Tauri CLI is unavailable"
grep -Fq '".sig"' "$APPVEYOR" || fail "AppVeyor must retain Windows updater signature receipts"
grep -Fq 'Generate signed Tauri updater metadata from provider receipts' "$WORKFLOW" || fail "release workflow must generate latest.json from provider signatures"

pass "release.yml gates signed menubar updater bundles via external providers"

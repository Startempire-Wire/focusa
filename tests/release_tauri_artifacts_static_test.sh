#!/usr/bin/env bash
# Guard: the canonical release publishes menubar desktop bundles for macOS and
# Windows. Under Spec 178 those are built by Codemagic (macOS) and AppVeyor
# (Windows). Codemagic uploads; the canonical self-hosted workflow pulls exact
# AppVeyor artifacts before the consolidated external completeness gate.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT_DIR/.github/workflows/release.yml"
WAIT="$ROOT_DIR/scripts/wait-for-external-release-assets.py"
INTAKE="$ROOT_DIR/scripts/intake-appveyor-release-artifacts.py"
CODEMAGIC="$ROOT_DIR/codemagic.yaml"
APPVEYOR="$ROOT_DIR/.appveyor.yml"
MENUBAR_CONFIG="$ROOT_DIR/apps/menubar/src-tauri/tauri.conf.json"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$WORKFLOW" ] || fail "release workflow missing"
[ -f "$WAIT" ] || fail "external release asset wait script missing"
[ -x "$INTAKE" ] || fail "AppVeyor intake adapter missing"

# One canonical intake pulls Windows bundles and checks every external surface.
grep -Fq 'Exact external provider artifact intake' "$WORKFLOW" || fail "missing consolidated external intake"
grep -Fq 'intake-appveyor-release-artifacts.py' "$WORKFLOW" || fail "Windows menubar intake is not invoked"
grep -Fq 'wait-for-external-release-assets.py' "$WORKFLOW" || fail "external completeness gate does not invoke wait script"
grep -Fq -- '--kind all' "$WORKFLOW" || fail "external gate does not require all assets"

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
[ "$(jq -r '.identifier' "$MENUBAR_CONFIG")" = "com.focusa.menubar" ] \
  || fail "Tauri config has unexpected canonical menubar identifier"
grep -Fq '[[ "$expected_identifier" == "com.focusa.menubar" ]]' "$CODEMAGIC" \
  || fail "Codemagic does not pin the canonical menubar identifier"
grep -Fq "Print :CFBundleIdentifier' \"\$app/Contents/Info.plist\")\" = \"\$expected_identifier\"" "$CODEMAGIC" \
  || fail "Codemagic Info.plist gate does not use the canonical config identifier"
if grep -Fq 'dev.focusa.menubar' "$CODEMAGIC"; then
  fail "Codemagic retains the obsolete menubar identifier"
fi
grep -Fq 'nsis' "$APPVEYOR" || fail "AppVeyor missing NSIS menubar bundle"
grep -Fq '$npmRoot = (& npm root -g).Trim()' "$APPVEYOR" || fail "AppVeyor must resolve npm global root after installing Bun"
grep -Fq '$bunExe = Join-Path $npmRoot "bun\bin\bun.exe"' "$APPVEYOR" || fail "AppVeyor must resolve the real Bun executable beneath npm global root instead of the PowerShell shim"
! grep -Fq 'Get-Command bun.exe' "$APPVEYOR" || fail "AppVeyor must not resolve nonexistent bun.exe through the PowerShell command shim"
grep -Fq '`"$bunExe`" $tauriCli --version && `"$bunExe`" $tauriCli build --target $env:RUST_TARGET' "$APPVEYOR" || fail "AppVeyor must invoke the package-owned Tauri CLI through the absolute Bun path for both Windows targets"
grep -Fq 'Bun executable missing under npm global root' "$APPVEYOR" || fail "AppVeyor must fail closed when the npm-owned Bun executable is unavailable"
grep -Fq 'package-owned Tauri CLI missing' "$APPVEYOR" || fail "AppVeyor must fail closed when the package-owned Tauri CLI is unavailable"
grep -Fq '".sig"' "$APPVEYOR" || fail "AppVeyor must retain Windows updater signature receipts"
if grep -Eq 'GH_TOKEN|GITHUB_RELEASE_TOKEN|uploads.github.com' "$APPVEYOR"; then
  fail "AppVeyor must not receive GitHub release write authority"
fi
grep -Fq 'Focusa_{version}_{architecture}-setup.exe' "$INTAKE" || fail "intake lacks exact tagged NSIS names"
grep -Fq 'Focusa_{version}_{architecture}_en-US.msi' "$INTAKE" || fail "intake lacks exact tagged MSI names"
grep -Fq 'Generate signed Tauri updater metadata from provider receipts' "$WORKFLOW" || fail "release workflow must generate latest.json from provider signatures"

pass "release.yml gates signed menubar updater bundles via external providers"

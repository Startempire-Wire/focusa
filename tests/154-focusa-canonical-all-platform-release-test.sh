#!/usr/bin/env bash
set -euo pipefail
# Canonical all-platform release guard under Spec 178 (external provider parity).
# Windows + macOS surfaces are built off GitHub (AppVeyor / Codemagic) and the
# durable contract lives in scripts/wait-for-external-release-assets.py, gated
# by the release.yml external receipt jobs. Linux stays on OVH self-hosted.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/release.yml"
WAIT="$ROOT/scripts/wait-for-external-release-assets.py"
APPVEYOR="$ROOT/.appveyor.yml"
CODEMAGIC="$ROOT/codemagic.yaml"
WINDOWS_WORKFLOW="$ROOT/.github/workflows/windows-ota-e2e.yml"
INSTALLER="$ROOT/scripts/install-focusa.ps1"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

# Windows + macOS Rust binaries are contractually required via the receipt gate.
for target in x86_64-pc-windows-msvc aarch64-pc-windows-msvc aarch64-apple-darwin x86_64-apple-darwin; do
  grep -q "${target}" "$WAIT" || fail "${target} must be present in the external Rust binary receipt contract"
done
pass "all four external Rust targets (win x64/arm64 + mac x64/arm64) in receipt contract"

# AppVeyor builds the full Windows surface; Codemagic builds the full macOS surface.
grep -q 'focusa-daemon' "$APPVEYOR" || fail "AppVeyor must build the daemon (full Windows surface)"
grep -q 'aarch64-pc-windows-msvc' "$APPVEYOR" || fail "AppVeyor must build Windows ARM64"
grep -q 'rust-macos-release-binaries' "$CODEMAGIC" || fail "Codemagic must define the rust-macos-release-binaries workflow"
grep -q 'aarch64-apple-darwin' "$CODEMAGIC" || fail "Codemagic must build macOS ARM64"
grep -q 'x86_64-apple-darwin' "$CODEMAGIC" || fail "Codemagic must build macOS x86_64"

# Installer still resolves every Windows target.
for target in x86_64-pc-windows-msvc aarch64-pc-windows-msvc; do
  grep -q "${target}" "$INSTALLER" || fail "Windows installer does not resolve ${target}"
done

# The canonical all-surface asset gate remains release-blocking.
grep -q 'Require every canonical surface on every supported system' "$WORKFLOW" || fail "final release lacks all-surface asset gate"
grep -q 'verify-canonical-release-assets.py' "$WORKFLOW" || fail "canonical asset verifier is not release-blocking"
grep -q 'focusa-installer-.*\.ps1' "$WORKFLOW" || fail "PowerShell installer is not published"
grep -q 'focusa-generated-clients-' "$WORKFLOW" || fail "generated clients are not published"

# Windows OTA workflow preflight remains intact.
grep -q 'Windows native dependency preflight' "$WINDOWS_WORKFLOW" || fail "Windows dependency preflight job missing"
grep -q 'windows_dependency_preflight_native_resolves_path_semver_and_health' "$WINDOWS_WORKFLOW" || fail "Windows PATH/semver/health diagnostic is not executed"
grep -q 'needs: \[windows-dependency-preflight\]' "$WINDOWS_WORKFLOW" || fail "Windows OTA does not wait for dependency preflight"
grep -q 'UIAI_ENGINE_URL.*127.0.0.1:17456' "$WINDOWS_WORKFLOW" || fail "Windows OTA lacks a responsive UIAI health fixture"

echo "canonical_all_platform_release=pass windows=x64,arm64 macos=x64,arm64 surfaces=cli,daemon,tui,desktop"

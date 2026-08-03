#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/release.yml"
INSTALLER="$ROOT/scripts/install-focusa.ps1"

fail() { echo "FAIL: $*" >&2; exit 1; }

for target in x86_64-pc-windows-msvc aarch64-pc-windows-msvc; do
  count="$(grep -c "target: ${target}" "$WORKFLOW")"
  [ "$count" -ge 2 ] || fail "${target} must exist in both desktop and Rust release matrices"
  grep -q "${target}" "$INSTALLER" || fail "Windows installer does not resolve ${target}"
done

grep -q 'os: windows-latest' "$WORKFLOW" || fail "Rust release matrix lacks Windows runner"
grep -q 'platform: windows-latest' "$WORKFLOW" || fail "desktop release matrix lacks Windows runner"
grep -q 'exe: .exe' "$WORKFLOW" || fail "Windows Rust binaries lack .exe packaging"
grep -q 'Require every canonical surface on every supported system' "$WORKFLOW" || fail "final release lacks all-surface asset gate"
grep -q 'verify-canonical-release-assets.py' "$WORKFLOW" || fail "canonical asset verifier is not release-blocking"
grep -q 'focusa-installer-.*\.ps1' "$WORKFLOW" || fail "PowerShell installer is not published"
grep -q 'focusa-generated-clients-' "$WORKFLOW" || fail "generated clients are not published"

echo "canonical_all_platform_release=pass windows=x64,arm64 surfaces=cli,daemon,tui,desktop"

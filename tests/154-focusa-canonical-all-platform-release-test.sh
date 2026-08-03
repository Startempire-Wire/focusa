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
grep -q 'Require canonical cross-platform release assets' "$WORKFLOW" || fail "final release lacks cross-platform asset gate"
grep -q 'Canonical release missing Windows x64 desktop installer' "$WORKFLOW" || fail "x64 desktop installer is not release-blocking"
grep -q 'Canonical release missing Windows ARM64 desktop installer' "$WORKFLOW" || fail "ARM64 desktop installer is not release-blocking"
grep -q 'for bin in focusa focusa-daemon focusa-tui' "$WORKFLOW" || fail "CLI/daemon/TUI Windows assets are not release-blocking"

echo "canonical_all_platform_release=pass windows=x64,arm64 surfaces=cli,daemon,tui,desktop"

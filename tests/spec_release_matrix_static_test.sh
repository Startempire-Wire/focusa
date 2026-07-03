#!/usr/bin/env bash
# spec_release_matrix_static_test.sh
#
# Static guard for release asset matrix beads:
# - focusa-112-windows-arm64-asset
# - focusa-112-musl-asset
#
# Backward compatibility: existing macOS/Linux glibc assets stay in matrix;
# new targets are additive. No target is removed or renamed.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WF="$ROOT_DIR/.github/workflows/release.yml"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# Existing targets retained
for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu; do
  grep -q "target: $target" "$WF" \
    || fail "Existing release matrix target removed: $target"
done
pass "existing macOS/Linux glibc release targets retained"

# Musl target present and uses cross (older glibc compatibility)
grep -q 'target: x86_64-unknown-linux-musl' "$WF" \
  || fail "Missing x86_64-unknown-linux-musl release matrix target"
grep -q 'musl: true' "$WF" \
  || fail "Musl release matrix target must set musl: true"
grep -q 'cross build --release --target' "$WF" \
  || fail "Musl release path must use cross build"
pass "musl/static Linux release asset target present and cross-built"

# Windows ARM64 target present and runs on Windows runner
grep -q 'target: aarch64-pc-windows-msvc' "$WF" \
  || fail "Missing aarch64-pc-windows-msvc release matrix target"
grep -q 'os: windows-latest' "$WF" \
  || fail "Windows ARM64 target must run on windows-latest"
grep -q 'exe: .exe' "$WF" \
  || fail "Windows ARM64 target must declare exe suffix"
pass "Windows ARM64 release asset target present"

# Packaging supports both Unix binaries and Windows .exe assets
grep -q 'EXE="${{ matrix.exe ||' "$WF" \
  || fail "Packaging step missing optional EXE suffix"
grep -q 'release/${bin}${EXE}' "$WF" \
  || fail "Packaging step missing .exe-aware source path"
grep -q 'dist/${bin}-${TAG}-${{ matrix.target }}${EXE}' "$WF" \
  || fail "Packaging step missing .exe-aware destination asset name"
pass "packaging step handles Windows .exe suffix without renaming Unix assets"

# Release notes advertise new assets
grep -q 'Windows ARM64' "$WF" \
  || fail "Release notes missing Windows ARM64 rows"
grep -q 'x86_64-unknown-linux-musl' "$WF" \
  || fail "Release notes missing musl rows"
pass "release notes advertise Windows ARM64 + musl assets"

echo "✓ All release matrix static checks passed"
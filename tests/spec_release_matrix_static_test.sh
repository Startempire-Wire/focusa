#!/usr/bin/env bash
# spec_release_matrix_static_test.sh
#
# Static guard for the canonical release asset matrix under Spec 178
# (temporary CI provider parity — docs/178).
#
# Linux targets build on OVH self-hosted runners inside release.yml.
# macOS + Windows Rust binaries build on Codemagic / AppVeyor and are
# uploaded back to the same release; the durable contract for those
# external surfaces lives in scripts/wait-for-external-release-assets.py.
#
# No target is removed or renamed from the canonical matrix — only the
# builder moved off GitHub-hosted (still billing-locked).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WF="$ROOT_DIR/.github/workflows/release.yml"
WAIT="$ROOT_DIR/scripts/wait-for-external-release-assets.py"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# Linux targets retained in the GitHub (OVH self-hosted) matrix.
for target in \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu \
  x86_64-unknown-linux-musl; do
  grep -q "target: $target" "$WF" \
    || fail "Linux release matrix target removed: $target"
done
pass "Linux gnu/musl/arm64 release targets retained on OVH self-hosted"

# macOS + Windows Rust binaries are now external (Codemagic / AppVeyor).
for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-pc-windows-msvc \
  aarch64-pc-windows-msvc; do
  grep -q "$target" "$WAIT" \
    || fail "external macOS/Windows release target missing from receipt gate: $target"
done
pass "macOS + Windows release targets retained via external receipt gate"

# Musl target present and uses cross (older glibc compatibility).
grep -q 'target: x86_64-unknown-linux-musl' "$WF" \
  || fail "Missing x86_64-unknown-linux-musl release matrix target"
grep -q 'musl: true' "$WF" \
  || fail "Musl release matrix target must set musl: true"
grep -q 'cross build --release --target' "$WF" \
  || fail "Musl release path must use cross build"
pass "musl/static Linux release asset target present and cross-built"

# Packaging stays .exe-aware (Windows binaries still exist, built externally).
grep -q 'EXE="${{ matrix.exe ||' "$WF" \
  || fail "Packaging step missing optional EXE suffix"
grep -q 'release/${bin}${EXE}' "$WF" \
  || fail "Packaging step missing .exe-aware source path"
pass "packaging step handles Windows .exe suffix without renaming Unix assets"

# The external receipt gates are wired into the release DAG.
grep -q 'External Rust Binary Receipt Gate' "$WF" \
  || fail "release.yml missing external Rust binary receipt gate job"
grep -q 'External Menubar Receipt Gate' "$WF" \
  || fail "release.yml missing external menubar receipt gate job"
grep -q 'wait-for-external-release-assets.py' "$WF" \
  || fail "release.yml missing external receipt wait script invocation"
pass "external macOS/Windows receipt gates wired into release DAG"

echo "✓ All release matrix static checks passed"

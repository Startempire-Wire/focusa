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
CI="$ROOT_DIR/.github/workflows/ci.yml"
WAIT="$ROOT_DIR/scripts/wait-for-external-release-assets.py"
CODEMAGIC="$ROOT_DIR/codemagic.yaml"
APPVEYOR="$ROOT_DIR/.appveyor.yml"

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

# Billing-lock recovery must resume an immutable tag from the current
# controller without moving it. The exact tag/SHA pair is verified before any
# draft Release is created; hosted rows stay visible but are disabled until the
# all-at-once restoration switch is explicitly enabled.
grep -q 'workflow_dispatch:' "$WF" \
  || fail "release controller lacks immutable-tag recovery dispatch"
grep -Fq 'RELEASE_TAG: ${{ inputs.release_tag || github.ref_name }}' "$WF" \
  || fail "release recovery does not carry the requested immutable tag"
grep -Fq 'RELEASE_SHA: ${{ inputs.release_sha || github.sha }}' "$WF" \
  || fail "release recovery does not carry the exact candidate SHA"
grep -q 'release_candidate_identity=passed' "$WF" \
  || fail "release recovery does not verify tag/SHA identity"
grep -q 'candidate_gate_substituted workflow=Spec-132 route=spec178 providers=ovh,appveyor,codemagic' "$WF" \
  || fail "Spec 132 billing-locked receipt is not delegated to the Spec 178 providers"
if grep -q 'require_success_with_wait "Spec 132 terminal matrix"' "$WF"; then
  fail "release controller still hard-blocks on GitHub-hosted Spec 132"
fi
grep -Fq "vars.FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX == 'enabled'" "$WF" \
  || fail "GitHub-hosted release matrix lacks an explicit restoration boundary"
pass "immutable-tag recovery bypasses GitHub billing through exact provider receipts"

# Release tags must trigger both Codemagic workflows regardless of the final
# stamped commit's changed paths. External adapters wait for, but never create,
# the canonical gated GitHub Release and fail closed on upload authority/errors.
if grep -q 'changeset:' "$CODEMAGIC"; then
  fail "Codemagic release tag workflows must not be path-filtered"
fi
[ "$(grep -c 'gh release view "\$CM_TAG"' "$CODEMAGIC")" -ge 2 ] \
  || fail "Codemagic adapters must wait for the canonical GitHub Release"
if grep -q 'bundles remain in artifacts\|binaries remain in artifacts' "$CODEMAGIC"; then
  fail "Codemagic must fail closed when GitHub upload authority is unavailable"
fi
grep -Fq "target\\%RUST_TARGET% -> Cargo.lock" "$APPVEYOR" \
  || fail "AppVeyor must keep target-specific Cargo.lock-keyed caches"
grep -q 'missing GitHub release upload credential' "$APPVEYOR" \
  || fail "AppVeyor must fail closed when GitHub upload authority is unavailable"
if grep -q 'Method Post.*repos/\$repo/releases"' "$APPVEYOR"; then
  fail "AppVeyor must never create the canonical GitHub Release"
fi
if grep -q 'upload failed for' "$APPVEYOR"; then
  fail "AppVeyor must not swallow artifact upload failures"
fi
pass "external release adapters are unconditional on tags and fail closed"

# Spec 178 keeps the billing-locked macOS job visible but non-authoritative;
# Codemagic remains fail-closed at release receipt gates.
menubar_block="$(awk '/^  menubar:/{job=1} /^  rust:/{job=0} job{print}' "$CI")"
grep -q 'continue-on-error: true' <<<"$menubar_block" \
  || fail "billing-locked CI Menubar job must remain informational under Spec 178"
pass "billing-locked GitHub macOS CI is non-authoritative without hiding the restoration target"

rust_ci_block="$(awk '/^  rust:/{job=1} /^  menubar:/{job=0} job{print}' "$CI")"
spec_ci_block="$(awk '/^  spec-gates:/{job=1} job{print}' "$CI")"
grep -Fq 'group: focusa-cargo-rust-${{ github.ref }}' <<<"$rust_ci_block" \
  || fail "Rust CI requires its own ref-scoped concurrency group"
grep -Fq 'group: focusa-cargo-spec-${{ github.ref }}' <<<"$spec_ci_block" \
  || fail "Spec CI requires its own ref-scoped concurrency group"
[[ "$(grep -c 'cancel-in-progress: true' <<<"$rust_ci_block")" -eq 1 ]] \
  || fail "Rust CI must cancel only its superseded same-ref instance"
[[ "$(grep -c 'cancel-in-progress: true' <<<"$spec_ci_block")" -eq 1 ]] \
  || fail "Spec CI must cancel only its superseded same-ref instance"
pass "Rust and Spec CI cannot cancel each other and superseded work is bounded"

# External Windows compilation must not cross an unguarded POSIX process API.
BG="crates/focusa-cli/src/commands/bg.rs"
grep -q '#\[cfg(unix)\]' "$BG" \
  || fail "bg detached monitor is missing Unix process-group boundary"
grep -q '#\[cfg(windows)\]' "$BG" \
  || fail "bg detached monitor is missing Windows process boundary"
grep -q 'CREATE_NEW_PROCESS_GROUP' "$BG" \
  || fail "bg detached monitor is missing Windows new-process-group authority"
grep -q 'CREATE_NO_WINDOW' "$BG" \
  || fail "bg detached monitor is missing Windows no-window behavior"
pass "bg detached monitor is platform-correct for Windows release builds"

echo "✓ All release matrix static checks passed"

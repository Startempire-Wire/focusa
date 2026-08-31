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
APPVEYOR_RECOVERY="$ROOT_DIR/config/appveyor-release-recovery.json"
CODEMAGIC_RECOVERY="$ROOT_DIR/config/codemagic-release-recovery.json"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

if awk 'NF && $0 !~ /^[[:space:]]/ && $0 !~ /^#/ && $0 != "workflows:" { print NR ":" $0 }' "$CODEMAGIC" | grep -q .; then
  fail "Codemagic YAML contains unexpected unindented block content"
fi
pass "Codemagic YAML block content remains structurally indented"

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
menubar_receipt_block="$(awk '/^  external-menubar-receipts:/{job=1} /^  rust-release:/{job=0} job{print}' "$WF")"
rust_receipt_block="$(awk '/^  external-rust-binaries:/{job=1} /^  pi-extension-release:/{job=0} job{print}' "$WF")"
grep -Fq 'needs: [create-release, pi-extension-release]' <<<"$menubar_receipt_block" \
  || fail "external menubar receipt waiter can starve the Pi-extension producer"
grep -Fq 'needs: [create-release, rust-release]' <<<"$rust_receipt_block" \
  || fail "external Rust receipt waiter can starve the Linux matrix producers"
for receipt_block in "$menubar_receipt_block" "$rust_receipt_block"; do
  grep -Fq 'timeout-minutes: 150' <<<"$receipt_block" \
    || fail "external receipt job cannot cover the serial AppVeyor matrix"
  grep -Fq -- '--timeout-minutes 145' <<<"$receipt_block" \
    || fail "external receipt polling cannot cover the serial AppVeyor matrix"
done
pass "external macOS/Windows receipt gates are producer-ordered and serial-provider-budgeted"

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
create_block="$(awk '/^  create-release:/{job=1} /^  tauri-build:/{job=0} job{print}' "$WF")"
if grep -q 'actions/upload-artifact' <<<"$create_block"; then
  fail "candidate lock must not depend on billing-locked GitHub artifact storage"
fi
grep -q 'files: release-candidate.json' <<<"$create_block" \
  || fail "candidate lock is not attached durably to the draft Release"
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
[ "$(grep -c 'gh release view "\$release_tag"' "$CODEMAGIC")" -ge 2 ] \
  || fail "Codemagic adapters must wait for the canonical GitHub Release"
grep -q 'FOCUSA_CODEMAGIC_RECOVERY' "$CODEMAGIC" \
  || fail "Codemagic recovery requires an explicit branch-build grant"
[ "$(grep -c 'codemagic_recovery_identity=passed' "$CODEMAGIC")" -eq 2 ] \
  || fail "both Codemagic workflows must prove exact tag/SHA identity"
[ "$(grep -Fc 'https://sh.rustup.rs' "$CODEMAGIC")" -eq 2 ] \
  || fail "both Codemagic workflows must bootstrap Rust on a clean API build"
[ "$(grep -Fc 'rustup default nightly-2026-08-28' "$CODEMAGIC")" -eq 2 ] \
  || fail "both Codemagic workflows must pin the canonical Rust toolchain"
grep -Fq -- '-p focusa-cli -p focusa-api -p focusa-tui' "$CODEMAGIC" \
  || fail "Codemagic Rust workflow must select the canonical daemon package"
[ "$(grep -Fc 'focusa-daemon-${release_tag}-' "$CODEMAGIC")" -eq 2 ] \
  || fail "Codemagic Rust workflow must require both canonical daemon assets"
grep -Fq '[[ "$FOCUSA_RELEASE_TAG" == "v0.9.187" ]]' "$CODEMAGIC" \
  || fail "Codemagic lock normalization is not fixed to exact recovery tag"
grep -Fq '[[ "$FOCUSA_RELEASE_SHA" == "01aae7ea9ab886627d49b68e7aed2349d9ceafc0" ]]' "$CODEMAGIC" \
  || fail "Codemagic lock normalization is not fixed to exact recovery SHA"
grep -Fq 'assert normalized(before) == normalized(after), "Cargo.lock drift exceeds recovery allowlist"' "$CODEMAGIC" \
  || fail "Codemagic recovery does not reject out-of-allowlist lock drift"
grep -Fq 'git diff --exit-code -- Cargo.lock' "$CODEMAGIC" \
  || fail "Codemagic recovery does not restore the immutable candidate lock"
grep -Fq 'cargo metadata --locked --format-version 1' "$CODEMAGIC" \
  || fail "normal Codemagic Rust builds do not prove locked metadata"
[ "$(grep -Fc 'cargo build --locked --release --target "$target"' "$CODEMAGIC")" -eq 1 ] \
  || fail "Codemagic Rust packaging must return to one locked build loop"
if grep -Eq '(^|[^[:alnum:]_-])focusad([^[:alnum:]_-]|$)' "$CODEMAGIC"; then
  fail "Codemagic Rust workflow references nonexistent focusad package or binary"
fi
grep -q 'missing base64 Tauri updater signing key payload' "$CODEMAGIC" \
  || fail "Codemagic signer does not require the encoded key payload"
grep -Fq 'base64.b64decode(os.environ["TAURI_SIGNING_PRIVATE_KEY"], validate=True)' "$CODEMAGIC" \
  || fail "Codemagic does not validate the secure outer-base64 key payload"
[ "$(grep -Fc 'python3 ../../scripts/ci/convert-legacy-tauri-signing-key.py' "$CODEMAGIC")" -eq 1 ] \
  || fail "Codemagic must convert the authenticated legacy signer to the current in-memory envelope exactly once"
[ "$(grep -Fc 'brew install libsodium' "$CODEMAGIC")" -eq 1 ] \
  || fail "Codemagic must install the signer conversion runtime exactly once"
grep -Fq 'assert ctypes.util.find_library("sodium"), "installed libsodium is not discoverable"' "$CODEMAGIC" \
  || fail "Codemagic must prove libsodium discovery before signer conversion"
libsodium_line="$(grep -Fn -m1 'brew install libsodium' "$CODEMAGIC" | cut -d: -f1)"
conversion_line="$(grep -Fn -m1 'python3 ../../scripts/ci/convert-legacy-tauri-signing-key.py' "$CODEMAGIC" | cut -d: -f1)"
[ "$libsodium_line" -lt "$conversion_line" ] \
  || fail "Codemagic must prepare libsodium before signer conversion"
grep -Fq '[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($env:TAURI_SIGNING_PRIVATE_KEY))' "$APPVEYOR" \
  || fail "AppVeyor does not validate the secure outer-base64 key payload"
if grep -q 'focusa-tauri-signing-key\|keyPath\|TAURI_SIGNING_PRIVATE_KEY = \$keyPath' "$CODEMAGIC" "$APPVEYOR"; then
  fail "provider workflow must not persist the decoded Tauri signing key"
fi
grep -Fq 'test -s "${updater}.sig"' "$CODEMAGIC" \
  || fail "Codemagic package step does not fail on a missing generated signature"
grep -Fq 'codesign --force --deep --sign - "$app"' "$CODEMAGIC" \
  || fail "Codemagic does not seal the completed app bundle"
[ "$(grep -Fc 'codesign -dv --verbose=4 "$app" 2>&1 | grep -F '\''Signature=adhoc'\''' "$CODEMAGIC")" -eq 2 ] \
  || fail "Codemagic must consume both complete codesign detail streams while asserting ad-hoc identity"
if grep -Fq "grep -q 'Signature=adhoc'" "$CODEMAGIC"; then
  fail "Codemagic ad-hoc verification can SIGPIPE codesign under pipefail"
fi
grep -Fq 'test -s "$app/Contents/_CodeSignature/CodeResources"' "$CODEMAGIC" \
  || fail "Codemagic does not require the app resource seal"
grep -Fq 'hdiutil create -volname Focusa -srcfolder "$dmg_stage" -ov -format UDZO "$dmg"' "$CODEMAGIC" \
  || fail "Codemagic does not rebuild the DMG from the sealed app"
grep -Fq 'mv "${updater}.sig" "${updater}.sig.tauri-original"' "$CODEMAGIC" \
  || fail "Codemagic can accidentally reuse the pre-seal updater signature"
grep -Fq 'tar -czf "$updater" -C "$(dirname "$app")" "$(basename "$app")"' "$CODEMAGIC" \
  || fail "Codemagic does not rebuild the updater archive from the sealed app"
grep -Fq 'npx tauri signer sign "$updater"' "$CODEMAGIC" \
  || fail "Codemagic does not re-sign the regenerated updater archive"
build_line="$(grep -Fn -m1 'npx tauri build --target "$target"' "$CODEMAGIC" | cut -d: -f1)"
seal_line="$(grep -Fn -m1 'codesign --force --deep --sign - "$app"' "$CODEMAGIC" | cut -d: -f1)"
dmg_line="$(grep -Fn -m1 'hdiutil create -volname Focusa' "$CODEMAGIC" | cut -d: -f1)"
updater_line="$(grep -Fn -m1 'tar -czf "$updater"' "$CODEMAGIC" | cut -d: -f1)"
signer_line="$(grep -Fn -m1 'npx tauri signer sign "$updater"' "$CODEMAGIC" | cut -d: -f1)"
copy_line="$(grep -Fn -m1 'ditto -c -k --keepParent "$app"' "$CODEMAGIC" | cut -d: -f1)"
[ "$build_line" -lt "$seal_line" ] && [ "$seal_line" -lt "$dmg_line" ] \
  && [ "$dmg_line" -lt "$updater_line" ] && [ "$updater_line" -lt "$signer_line" ] \
  && [ "$signer_line" -lt "$copy_line" ] \
  || fail "Codemagic must seal, rebuild, sign, then copy menubar artifacts in that order"
[ "$(grep -Fc 'app.tar.gz.sig' "$CODEMAGIC")" -ge 4 ] \
  || fail "Codemagic package/upload contract does not require both updater signatures"
if grep -q 'bundles remain in artifacts\|binaries remain in artifacts' "$CODEMAGIC"; then
  fail "Codemagic must fail closed when GitHub upload authority is unavailable"
fi
grep -Fq "target\\%RUST_TARGET% -> Cargo.lock" "$APPVEYOR" \
  || fail "AppVeyor must keep target-specific Cargo.lock-keyed caches"
grep -Fq "apps\\menubar\\src-tauri\\target\\%RUST_TARGET% -> apps\\menubar\\src-tauri\\Cargo.lock" "$APPVEYOR" \
  || fail "AppVeyor must keep target-specific Menubar caches"
[ "$(grep -c '^      SURFACE: rust$' "$APPVEYOR")" -eq 2 ] \
  || fail "AppVeyor must isolate Rust work into two architecture jobs"
[ "$(grep -c '^      SURFACE: menubar$' "$APPVEYOR")" -eq 2 ] \
  || fail "AppVeyor must isolate Menubar work into two architecture jobs"
grep -Fq 'cargo build --release --target $env:RUST_TARGET -p focusa-cli -p focusa-api -p focusa-tui' "$APPVEYOR" \
  || fail "AppVeyor Rust jobs must build only canonical release packages"
grep -Fq 'if ($env:SURFACE -eq "menubar" -and ($env:APPVEYOR_REPO_TAG -eq "true" -or $env:FOCUSA_RECOVERY_TAG))' "$APPVEYOR" \
  || fail "AppVeyor Menubar packaging must be surface-isolated and release-gated"
[ "$(grep -Fc 'if ($env:SURFACE -ne "rust")' "$APPVEYOR")" -eq 3 ] \
  || fail "AppVeyor Rust build, binary copy, and tests must be surface-isolated"
grep -q 'missing GitHub release upload credential' "$APPVEYOR" \
  || fail "AppVeyor must fail closed when GitHub upload authority is unavailable"
grep -Fq '[Convert]::FromBase64String($env:TAURI_SIGNING_PRIVATE_KEY)' "$APPVEYOR" \
  || fail "AppVeyor does not decode the secure signing key payload"
grep -Fq '$env:TAURI_SIGNING_PRIVATE_KEY = $null' "$APPVEYOR" \
  || fail "AppVeyor does not clear the signing payload after package work"
grep -q 'appveyor_recovery_identity=passed' "$APPVEYOR" \
  || fail "AppVeyor lacks exact tag/SHA recovery identity proof"
grep -Fq '$recovery.enabled -eq $true -and $env:APPVEYOR_REPO_TAG -ne "true"' "$APPVEYOR" \
  || fail "AppVeyor recovery is not restricted to controller branch builds"
grep -Fq 'appveyor_recovery_ignored_for_tag=true' "$APPVEYOR" \
  || fail "AppVeyor tag builds do not explicitly ignore recovery state"
grep -q 'FOCUSA_RECOVERY_TAG' "$APPVEYOR" \
  || fail "AppVeyor recovery does not carry the immutable release tag"
grep -Fq '$env:CI = "true"' "$APPVEYOR" \
  || fail "AppVeyor does not normalize Tauri CI semantics to lowercase true"
grep -Fq 'appveyor_tauri_ci_normalized=$env:CI' "$APPVEYOR" \
  || fail "AppVeyor lacks lowercase Tauri CI proof"
[ "$(grep -Fc '2>&1"' "$APPVEYOR")" -ge 3 ] \
  || fail "AppVeyor native commands do not redirect normal Cargo stderr inside cmd.exe"
python3 - "$APPVEYOR_RECOVERY" "$CODEMAGIC_RECOVERY" "$WF" "$CODEMAGIC" <<'PY'
import json, re, sys
for recovery_path in sys.argv[1:3]:
    payload = json.load(open(recovery_path, encoding="utf-8"))
    assert isinstance(payload.get("enabled"), bool)
    assert re.fullmatch(r"v\d+\.\d+\.\d+", payload["tag"])
    assert re.fullmatch(r"[0-9a-f]{40}", payload["sha"])
appveyor_recovery = json.load(open(sys.argv[1], encoding="utf-8"))
assert appveyor_recovery == {
    "enabled": True,
    "tag": "v0.9.187",
    "sha": "01aae7ea9ab886627d49b68e7aed2349d9ceafc0",
}, "AppVeyor recovery identity must remain pinned to the immutable v0.9.187 candidate"
lines = open(sys.argv[3], encoding="utf-8").read().splitlines()
uploads = [i for i, line in enumerate(lines) if "uses: softprops/action-gh-release@v2" in line]
assert uploads, "release workflow has no GitHub Release upload actions"
for i in uploads:
    block = "\n".join(lines[i:i + 16])
    assert "tag_name: ${{ env.RELEASE_TAG }}" in block, f"release upload at line {i + 1} infers tag from github.ref"
    assert "draft: true" in block, f"release upload at line {i + 1} can publish before receipt gates"
text = "\n".join(lines)
tauri_uploads = [i for i, line in enumerate(lines) if "uses: tauri-apps/tauri-action@v0" in line]
assert tauri_uploads, "release workflow has no Tauri upload action"
for i in tauri_uploads:
    block = "\n".join(lines[i:i + 24])
    assert "releaseDraft: true" in block, f"Tauri upload at line {i + 1} can publish before receipt gates"
assert text.count("--draft=false") == 1, "release workflow must have exactly one final publisher"
publish = text.index("--draft=false")
checksums = text.index("  checksums:")
assert publish > checksums, "release publisher must remain downstream of receipt-gated checksums"
cm_text = open(sys.argv[4], encoding="utf-8").read()
assert cm_text.count("printf 'PATH=%s\\n' \"$PATH\" >> \"$CM_ENV\"") == 2, "Codemagic Rust bootstrap PATH does not persist across steps"
cm_lines = cm_text.splitlines()
script_count = 0
for index, line in enumerate(cm_lines):
    if line.strip() != "script: |":
        continue
    script_count += 1
    cursor = index + 1
    while cursor < len(cm_lines) and not cm_lines[cursor].strip():
        cursor += 1
    assert cm_lines[cursor].strip() == "set -euo pipefail", f"Codemagic script at line {index + 1} lacks strict shell mode"
assert script_count >= 10, "Codemagic release workflow unexpectedly lost script gates"
PY
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

python3 tests/codemagic_recovery_lock_normalization_test.py

echo "✓ All release matrix static checks passed"

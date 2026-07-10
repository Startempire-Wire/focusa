#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/crates/focusa-core/src/update.rs"
LIB="$ROOT/crates/focusa-core/src/lib.rs"
SPEC="$ROOT/docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f "$SRC" ]] || fail "missing focusa-core update primitives module"
rg -q 'pub mod update;' "$LIB" || fail "focusa-core does not export update module"
rg -q 'RELEASE_MANIFEST_SCHEMA_V1.*focusa\.release_manifest\.v1' "$SRC" || fail "missing release manifest schema constant"
rg -q 'struct ReleaseManifest' "$SRC" || fail "missing ReleaseManifest"
rg -q 'struct ReleaseAsset' "$SRC" || fail "missing ReleaseAsset"
rg -q 'struct AssetSignature' "$SRC" || fail "missing AssetSignature"
rg -q 'struct ReleaseTrust' "$SRC" || fail "missing ReleaseTrust"
rg -q 'struct ReleaseProvenance' "$SRC" || fail "missing ReleaseProvenance"
rg -q 'struct ReleaseCompatibility' "$SRC" || fail "missing ReleaseCompatibility"
rg -q 'struct ReleaseEligibilityOptions' "$SRC" || fail "missing ReleaseEligibilityOptions"
rg -q 'fn evaluate_release_manifest' "$SRC" || fail "missing release eligibility evaluator"
rg -q 'release_yanked|release_revoked|trust_root_missing|untrusted_signing_key|asset_signature_missing|unsupported_platform|channel_mismatch' "$SRC" || fail "missing required eligibility/trust failure codes"
rg -q 'auto_apply_allowed: false' "$SRC" || fail "release primitive must not allow auto-apply before policy/lock/rollback gates"
rg -q 'Release manifest|focusa\.release_manifest\.v1|manifest schema' "$SPEC" || fail "Spec128 does not describe release manifest schema"
rg -q 'Release eligibility|Cryptographic trust|signing trust' "$SPEC" || fail "Spec128 missing trust/eligibility sections"

pass "Spec128 release manifest/signing/eligibility primitives present"

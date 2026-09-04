#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POLICY="$ROOT_DIR/docs/current/ARCHITECTURE_AUTHORITY_POLICY.md"
AGENTS="$ROOT_DIR/AGENTS.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f "$POLICY" ]] || fail "missing architecture authority policy"

grep -Fq 'Verious Smith III is the sole current and final canonical human architecture authority' "$POLICY" \
  || fail "Verious Smith III root authority invariant missing"
grep -Fq 'wirebot_identity_sha256 = SHA-256(canonical_json(identity_manifest))' "$POLICY" \
  || fail "Wirebot identity hash contract missing"
grep -Fq 'subject_public_key_fingerprint' "$POLICY" \
  || fail "Wirebot public-key binding missing"
grep -Fq 'may_delegate: false' "$POLICY" \
  || fail "Wirebot non-transitive delegation default missing"
grep -Fq 'Verious Smith III-rooted signed delegation' "$POLICY" \
  || fail "Wirebot signed delegation root missing"
grep -Fq 'advisory_external' "$POLICY" \
  || fail "external provenance advisory posture missing"
pass "constitutional architecture authority contract present"

# Known customer/customer-agent identifiers must never re-enter current product docs.
# The strings are split here so this guard itself does not become a searchable docs occurrence.
for forbidden in "Bar""ry" "Spo""ck" "Kre""voy" "4ir""inc"; do
  if grep -RIni --include='*.md' --include='*.txt' --exclude='architecture_authority_policy_static_test.sh' \
      "$forbidden" "$ROOT_DIR/docs" "$ROOT_DIR/README.md" "$AGENTS" 2>/dev/null; then
    fail "customer-specific identifier found in current product documentation: $forbidden"
  fi
done
pass "known customer-specific identifiers absent from current product docs"

echo "architecture authority policy static test: PASS"

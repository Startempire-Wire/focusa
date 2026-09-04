#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
POLICY="$ROOT_DIR/docs/current/ARCHITECTURE_AUTHORITY_POLICY.md"
DOC_AGENTS="$ROOT_DIR/docs/AGENTS.md"
ROOT_AGENTS="$ROOT_DIR/AGENTS.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f "$POLICY" ]] || fail "missing architecture authority policy"
[[ -f "$DOC_AGENTS" ]] || fail "missing docs authority agent contract"

grep -Fq 'Verious Smith III is the sole current and final canonical human architecture authority' "$POLICY" \
  || fail "Verious Smith III root authority invariant missing"
grep -Fq 'every GitHub repository and organization owned, administered, or canonically controlled by Verious Smith III' "$POLICY" \
  || fail "GitHub estate scope invariant missing"
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
grep -Fq 'Architecture authority hard stop' "$DOC_AGENTS" \
  || fail "docs/AGENTS.md does not enforce authority policy"
pass "constitutional architecture authority contract present"

# Known customer/customer-agent identifiers must never re-enter current product docs.
# Split literals keep this guard from itself becoming a searchable product-doc occurrence.
for forbidden in "Bar""ry" "Spo""ck" "Kre""voy" "4ir""inc"; do
  if grep -RIni --include='*.md' --include='*.txt' \
      "$forbidden" "$ROOT_DIR/docs" "$ROOT_DIR/README.md" "$ROOT_AGENTS" 2>/dev/null; then
    fail "customer-specific identifier found in current product documentation: $forbidden"
  fi
done
pass "known customer-specific identifiers absent from current product docs"

echo "architecture authority policy static test: PASS"

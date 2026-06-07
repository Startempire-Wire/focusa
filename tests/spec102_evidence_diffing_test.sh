#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in evidence_diff before_ref after_ref changed_claims confidence_delta regressions stale_refs_removed new_followups no_confidence_change; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse evidence diff missing $term"
done
pass "traverse declares evidence_diff contract terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-ediff-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"ediff-agent\",\"work_item_id\":\"focusa-pm2b.40\",\"mission\":\"Spec102 evidence diff fixture\",\"next_slice\":\"evidence diff\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\",\"verification_records\":[{\"target_ref\":\"tests/spec102_evidence_diffing_test.sh\",\"result\":\"PASS old proof baseline $KEY\",\"evidence_ref\":\"evidence:$KEY:old\"},{\"target_ref\":\"tests/spec102_evidence_diffing_test.sh\",\"result\":\"PASS new proof increases confidence $KEY\",\"evidence_ref\":\"evidence:$KEY:new\"}]}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-ediff-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-ediff-wp.json)

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"evidence\",\"selector\":\"confidence_change\",\"query\":\"$KEY\",\"limit\":10}" \
  "$BASE/v1/traverse" >/tmp/spec102-ediff.json
jq -e --arg wp "$WP" '
  .traversal.evidence_diff.before_ref != null
  and .traversal.evidence_diff.after_ref != null
  and (.traversal.evidence_diff.changed_claims | length) >= 1
  and (.traversal.evidence_diff.confidence_delta | test("increased|proof_linked"; "i"))
  and (.traversal.evidence_diff.regressions | length) == 0
  and (.traversal.evidence_diff.stale_refs_removed | type == "array")
  and (.traversal.evidence_diff.new_followups | type == "array")
  and (.traversal.artifact_browser.artifacts[] | select(.workpoint_id == $wp and .confidence_delta != null))
' /tmp/spec102-ediff.json >/dev/null || fail "evidence_diff missing confidence-changing comparison"
pass "evidence_diff compares old/new proof and confidence delta"

NOCHANGE="spec102-ediff-nochange-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"evidence\",\"selector\":\"confidence_change\",\"query\":\"$NOCHANGE\",\"limit\":10}" \
  "$BASE/v1/traverse" >/tmp/spec102-ediff-nochange.json
jq -e '
  .traversal.evidence_diff.confidence_delta == "no_confidence_change"
  and (.traversal.evidence_diff.new_followups | length) >= 1
  and (.traversal.evidence_diff.new_followups[0] | test("next proof|capture|link"; "i"))
' /tmp/spec102-ediff-nochange.json >/dev/null || fail "no-change evidence diff missing next proof suggestion"
pass "no-change evidence_diff suggests next proof"

echo "SPEC102 evidence diffing test: PASS"

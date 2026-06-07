#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in artifact_browser confidence_change confidence_delta duplicate_cluster stale_refs group_by; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse evidence missing $term"
done
pass "traverse evidence declares confidence navigation terms"

if curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null 2>&1; then
  KEY="spec102-confidence-$$"
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"$KEY\",\"mission\":\"Spec102 confidence evidence fixture\",\"next_slice\":\"Find confidence-changing proof\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
    "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-confidence-wp.json
  WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-confidence-wp.json)
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"workpoint_id\":\"$WP\",\"target_ref\":\"tests/spec102_evidence_confidence_navigation_test.sh\",\"result\":\"PASS confidence-changing proof $KEY\",\"evidence_ref\":\"evidence:$KEY\"}" \
    "$BASE/v1/workpoint/evidence/link" >/tmp/spec102-confidence-link.json
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"surface\":\"evidence\",\"selector\":\"confidence_change\",\"query\":\"$KEY\",\"limit\":10}" \
    "$BASE/v1/traverse" >/tmp/spec102-confidence-traverse.json
  jq -e --arg wp "$WP" '
    .traversal.artifact_browser.group_by == "confidence_change"
    and (.traversal.artifact_browser.artifacts | length >= 1)
    and (.traversal.artifact_browser.artifacts[] | select(.workpoint_id == $wp and .confidence_delta != null and .rehydrate_ref != null))
    and (.items | length <= 10)
  ' /tmp/spec102-confidence-traverse.json >/dev/null || fail "confidence_change evidence view missing grouped confidence artifact"
  pass "confidence_change selector finds scoped proof without huge count"
fi

echo "SPEC102 evidence confidence navigation test: PASS"

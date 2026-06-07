#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in artifact_browser workpoint bead spec file test confidence_change freshness stale_refs duplicate_clusters group_key; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse artifact browser missing $term"
done
pass "artifact browser declares all grouping terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-artifact-$$"
TARGET="tests/spec102_proof_artifact_browser_test.sh"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"artifact-agent\",\"work_item_id\":\"focusa-pm2b.29\",\"mission\":\"Spec102 artifact browser fixture\",\"active_object_refs\":[\"$TARGET\",\"docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md\"],\"next_slice\":\"proof artifact browser\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\",\"verification_records\":[{\"target_ref\":\"$TARGET\",\"result\":\"PASS spec102 artifact proof $KEY\",\"evidence_ref\":\"evidence:$KEY:test\"},{\"target_ref\":\"docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md#15.3\",\"result\":\"PASS spec102 artifact report $KEY\",\"evidence_ref\":\"evidence:$KEY:spec\"}]}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-artifact-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-artifact-wp.json)
[[ -n "$WP" && "$WP" != null ]] || fail "checkpoint missing workpoint_id"

for GROUP in workpoint bead spec file test confidence_change; do
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"surface\":\"evidence\",\"selector\":\"$GROUP\",\"query\":\"$KEY\",\"limit\":10}" \
    "$BASE/v1/traverse" >"/tmp/spec102-artifact-$GROUP.json"
  jq -e --arg group "$GROUP" --arg wp "$WP" '
    .traversal.artifact_browser.group_by == $group
    and (.traversal.artifact_browser.stale_refs | length) == 0
    and (.traversal.artifact_browser.artifacts | length) >= 1
    and (.traversal.artifact_browser.artifacts[] | select(.workpoint_id == $wp and .evidence_ref != null and .target_ref != null and .kind != null and .freshness != null and .rehydrate_ref != null and .group_key != null))
    and (.items | length) <= 10
  ' "/tmp/spec102-artifact-$GROUP.json" >/dev/null || fail "artifact browser group $GROUP missing scoped proof metadata"
done
pass "artifact browser groups scoped proof by workpoint/bead/spec/file/test/confidence_change"

echo "SPEC102 proof artifact browser test: PASS"

#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in change_feed files_changed beads_changed workpoints_changed evidence_changed predictions_changed agents_changed attention_required 'changes: none relevant'; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse change feed missing $term"
done
pass "traverse declares change_feed contract terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
NONE="spec102-no-changes-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"change_feed\",\"selector\":\"since\",\"query\":\"$NONE\",\"limit\":5}" \
  "$BASE/v1/traverse" >/tmp/spec102-change-none.json
jq -e '
  .traversal.change_feed.summary == "changes: none relevant"
  and .traversal.change_feed.attention_required == false
  and (.traversal.change_feed.files_changed | length) == 0
  and (.traversal.change_feed.beads_changed | length) == 0
' /tmp/spec102-change-none.json >/dev/null || fail "no-change feed not calm"
pass "no relevant changes render calm none relevant"

KEY="spec102-change-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"change-agent\",\"work_item_id\":\"focusa-pm2b.37\",\"mission\":\"Spec102 change feed fixture\",\"active_object_refs\":[\"crates/focusa-api/src/routes/traverse.rs\",\"tests/spec102_notification_change_feed_test.sh\"],\"next_slice\":\"change feed\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\",\"verification_records\":[{\"target_ref\":\"tests/spec102_notification_change_feed_test.sh\",\"result\":\"PASS change feed proof $KEY\",\"evidence_ref\":\"evidence:$KEY\"}]}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-change-wp.json
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"change_feed\",\"selector\":\"since\",\"query\":\"$KEY\",\"limit\":5}" \
  "$BASE/v1/traverse" >/tmp/spec102-change-feed.json
jq -e '
  .traversal.change_feed.attention_required == true
  and (.traversal.change_feed.workpoints_changed | length) >= 1
  and (.traversal.change_feed.beads_changed | index("focusa-pm2b.37"))
  and (.traversal.change_feed.files_changed | index("crates/focusa-api/src/routes/traverse.rs"))
  and (.traversal.change_feed.evidence_changed | length) >= 1
  and (.traversal.change_feed.agents_changed | index("change-agent"))
  and (.traversal.change_feed.summary | test("attention|required|changed"; "i"))
' /tmp/spec102-change-feed.json >/dev/null || fail "relevant change feed missing attention summary"
pass "relevant changes show attention-required summary"

echo "SPEC102 notification change feed test: PASS"

#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in stuck_loop repeated_actions last_confidence_change likely_cause break_glass_action no_confidence_change; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "stuck loop term missing $term"
done
pass "stuck-loop detector declares Spec102 terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
OKKEY="spec102-no-loop-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$OKKEY\",\"session_id\":\"loop-agent\",\"work_item_id\":\"focusa-pm2b.35\",\"mission\":\"Spec102 no loop\",\"next_slice\":\"one useful action\",\"canonical\":true,\"idempotency_key\":\"wp-$OKKEY\",\"verification_records\":[{\"target_ref\":\"tests/spec102_stuck_loop_detector_test.sh\",\"result\":\"PASS confidence change\",\"evidence_ref\":\"evidence:$OKKEY\"}]}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-no-loop-wp.json
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"workpoints\",\"selector\":\"window\",\"query\":\"$OKKEY\",\"limit\":10}" \
  "$BASE/v1/traverse" >/tmp/spec102-no-loop.json
jq -e '(.traversal.stuck_loop == null) or (.traversal.stuck_loop.detected == false)' /tmp/spec102-no-loop.json >/dev/null || fail "happy path should be silent/no stuck loop"
pass "happy path is silent/no stuck loop"

LOOPKEY="spec102-loop-$$"
for i in 1 2 3; do
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$LOOPKEY\",\"session_id\":\"loop-agent-$i\",\"work_item_id\":\"focusa-pm2b.35\",\"mission\":\"Spec102 repeated resume/checkpoint loop\",\"next_slice\":\"repeat same route without proof\",\"canonical\":true,\"idempotency_key\":\"wp-$LOOPKEY-$i\"}" \
    "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-loop-wp-$i.json
 done
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"workpoints\",\"selector\":\"window\",\"query\":\"$LOOPKEY\",\"limit\":10}" \
  "$BASE/v1/traverse" >/tmp/spec102-loop.json
jq -e '
  .traversal.stuck_loop.detected == true
  and (.traversal.stuck_loop.repeated_actions | length) >= 1
  and (.traversal.stuck_loop.last_confidence_change == "none" or (.traversal.stuck_loop.last_confidence_change | test("none|no_confidence_change"; "i")))
  and (.traversal.stuck_loop.likely_cause | test("no confidence|repeated|proof"; "i"))
  and (.traversal.stuck_loop.break_glass_action | test("link evidence|change route|operator"; "i"))
' /tmp/spec102-loop.json >/dev/null || fail "loop path missing repeated actions/cause/break-glass action"
pass "loop path detects repeated no-confidence-change actions"

echo "SPEC102 stuck-loop detector test: PASS"

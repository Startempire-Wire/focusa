#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in ownership_board active_agents collision_risk safe_next_action lease_status ownership:; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse ownership board missing $term"
done
pass "traverse declares ownership board contract terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-own-$$"
# Single active scoped Workpoint should be calm and bounded.
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"agent-a\",\"work_item_id\":\"focusa-pm2b.27\",\"mission\":\"Spec102 ownership board single\",\"active_object_refs\":[\"crates/focusa-api/src/routes/traverse.rs\"],\"next_slice\":\"ownership board\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY-a\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-own-a.json
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"ownership\",\"selector\":\"board\",\"query\":\"$KEY\",\"limit\":5}" \
  "$BASE/v1/traverse" >/tmp/spec102-own-single.json
jq -e '
  .traversal.ownership_board.status == "ownership: clear"
  and .traversal.ownership_board.collision_risk == "none"
  and (.traversal.ownership_board.active_agents | length) == 1
  and (.traversal.ownership_board.active_agents[0].lease_status == "active")
  and (.traversal.ownership_board.safe_next_action | test("continue"; "i"))
  and (.items | length) <= 5
' /tmp/spec102-own-single.json >/dev/null || fail "single-agent ownership board not calm/clear"
pass "single-agent ownership board is calm and clear"

# Two active workpoints touching same file should expose collision metadata.
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"agent-b\",\"work_item_id\":\"focusa-pm2b.27\",\"mission\":\"Spec102 ownership board collision\",\"active_object_refs\":[\"crates/focusa-api/src/routes/traverse.rs\"],\"next_slice\":\"ownership board collision\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY-b\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-own-b.json
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"ownership\",\"selector\":\"board\",\"query\":\"$KEY\",\"limit\":5}" \
  "$BASE/v1/traverse" >/tmp/spec102-own-collision.json
jq -e '
  .traversal.ownership_board.collision_risk == "high"
  and (.traversal.ownership_board.active_agents | length) >= 2
  and (.traversal.ownership_board.collision_files | index("crates/focusa-api/src/routes/traverse.rs"))
  and (.traversal.ownership_board.safe_next_action | test("coordinate|pause|handoff|collision"; "i"))
  and (.traversal.ownership_board.active_agents[] | select(.agent_id == "agent-a" or .agent_id == "agent-b") | .lease_status == "active")
' /tmp/spec102-own-collision.json >/dev/null || fail "collision ownership board missing owners/files/safe action"
pass "collision ownership board identifies owners, touched files, lease status, safe action"

echo "SPEC102 multi-agent ownership board test: PASS"

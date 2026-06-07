#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in route_recommendation recommended_tool expected_output alternatives avoid confidence full lineage full ontology; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "route recommender term missing $term"
done
pass "route recommender declares Spec102 terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-route-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"route-agent\",\"work_item_id\":\"focusa-pm2b.34\",\"mission\":\"Spec102 route recommender\",\"next_slice\":\"route recommendation\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-route-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-route-wp.json)

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-route-resume.json
jq -e '
  .route_recommendation.recommended_tool == "focusa_trajectory_view"
  and (.route_recommendation.why | test("bounded|next"; "i"))
  and (.route_recommendation.expected_output | test("goal|gap|state"; "i"))
  and .route_recommendation.confidence == "high"
  and (.route_recommendation.alternatives | index("focusa_traverse"))
  and (.route_recommendation.avoid | index("full lineage tree"))
  and (.route_recommendation.avoid | index("full ontology graph"))
  and (.resume_packet_v2.route_recommendation.recommended_tool == "focusa_trajectory_view")
' /tmp/spec102-route-resume.json >/dev/null || fail "Workpoint resume route recommendation missing bounded next route"
pass "Workpoint resume recommends one bounded next route"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"evidence","selector":"workpoint","query":"spec102-route","limit":5}' \
  "$BASE/v1/traverse" >/tmp/spec102-route-traverse.json
jq -e '
  .route_recommendation.recommended_tool == "focusa_traverse"
  and (.route_recommendation.why | test("bounded"; "i"))
  and (.route_recommendation.expected_output | test("slice|items|evidence|artifact"; "i"))
  and (.route_recommendation.avoid | index("full lineage tree"))
  and (.route_recommendation.avoid | index("full ontology graph"))
  and (.traversal.route_recommendation.recommended_tool == "focusa_traverse")
' /tmp/spec102-route-traverse.json >/dev/null || fail "traverse route recommendation missing bounded-route guidance"
pass "traverse recommends bounded route and discourages broad/cold routes"

echo "SPEC102 route recommender test: PASS"

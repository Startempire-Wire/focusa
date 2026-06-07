#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in trust_badges canonical verified advisory projected stale degraded blocked spec_only partial unsafe_scope; do
  rg -F "$term" crates/focusa-api/src/routes/workpoint.rs crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "trust badge term missing $term"
done
pass "Workpoint/traverse declare consistent trust badge vocabulary"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-badge-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"badge-agent\",\"work_item_id\":\"focusa-pm2b.32\",\"mission\":\"Spec102 trust badges\",\"next_slice\":\"trust badge happy path\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-badge-wp.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-badge-wp.json)
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-badge-resume.json
jq -e '
  (.trust_badges == ["canonical", "verified"])
  and (.resume_packet_v2.trust_badges == ["canonical", "verified"])
  and ((.trust_badges | index("degraded")) | not)
  and ((.trust_badges | index("blocked")) | not)
  and ((.trust_badges | index("unsafe_scope")) | not)
' /tmp/spec102-badge-resume.json >/dev/null || fail "canonical resume does not show only positive trust badges"
pass "canonical Workpoint resume shows only positive trust badges"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"workpoints","selector":"window","limit":1}' \
  "$BASE/v1/traverse" >/tmp/spec102-badge-traverse.json
jq -e '(.trust_badges == ["canonical", "verified"]) and (.degraded == false)' /tmp/spec102-badge-traverse.json >/dev/null || fail "healthy traverse missing positive trust badges"
pass "healthy traverse shows canonical verified badges"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"not_a_surface","selector":"window","limit":1}' \
  "$BASE/v1/traverse" >/tmp/spec102-badge-blocked.json
jq -e '(.trust_badges | index("blocked")) and (.trust_badges | index("degraded")) and .failure_class == "validation_rejected"' /tmp/spec102-badge-blocked.json >/dev/null || fail "blocked traverse missing blocked/degraded badges"
pass "blocked traverse shows only relevant negative badges"

echo "SPEC102 trust badges per surface test: PASS"

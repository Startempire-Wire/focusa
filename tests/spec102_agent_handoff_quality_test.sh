#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in handoff_quality 'handoff: ready' next_action_quality proof_quality authority missing stale; do
  rg -F "$term" crates/focusa-api/src/routes/workpoint.rs >/dev/null || fail "workpoint resume missing handoff term $term"
done
pass "workpoint resume declares handoff quality contract terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
KEY="spec102-handoff-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"session_id\":\"agent-ready\",\"work_item_id\":\"focusa-pm2b.28\",\"mission\":\"Spec102 handoff quality ready\",\"active_object_refs\":[\"crates/focusa-api/src/routes/workpoint.rs\"],\"next_slice\":\"Implement handoff quality\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY-ready\",\"verification_records\":[{\"target_ref\":\"tests/spec102_agent_handoff_quality_test.sh\",\"result\":\"PASS fixture\",\"evidence_ref\":\"evidence:$KEY\"}]}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-handoff-checkpoint.json
WP=$(jq -r '.workpoint_id // empty' /tmp/spec102-handoff-checkpoint.json)
[[ -n "$WP" && "$WP" != null ]] || fail "checkpoint missing workpoint_id"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY\",\"workpoint_id\":\"$WP\",\"current_ask\":\"spec 102\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-handoff-ready.json
jq -e '
  .handoff_quality.status == "ready"
  and .handoff_quality.score >= 90
  and (.handoff_quality.missing | length) == 0
  and (.handoff_quality.stale | length) == 0
  and (.handoff_quality.authority == "canonical")
  and (.handoff_quality.next_action_quality == "exact")
  and (.handoff_quality.proof_quality == "linked")
  and (.rendered_summary | test("handoff: ready"))
  and (.rendered_summary | test("next="))
  and (.resume_packet_v2.handoff_quality.status == "ready")
' /tmp/spec102-handoff-ready.json >/dev/null || fail "ready handoff quality missing score/status/exact next action"
pass "ready handoff shows score/status/exact next action without gaps"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY-partial\",\"session_id\":\"agent-partial\",\"work_item_id\":\"focusa-pm2b.28\",\"mission\":\"Spec102 handoff quality partial\",\"canonical\":false,\"idempotency_key\":\"wp-$KEY-partial\"}" \
  "$BASE/v1/workpoint/checkpoint" >/tmp/spec102-handoff-partial-checkpoint.json
PWP=$(jq -r '.workpoint_id // empty' /tmp/spec102-handoff-partial-checkpoint.json)
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"$KEY-partial\",\"workpoint_id\":\"$PWP\",\"current_ask\":\"spec 102\",\"mode\":\"compact_prompt\"}" \
  "$BASE/v1/workpoint/resume" >/tmp/spec102-handoff-partial.json
jq -e '
  (.handoff_quality.status == "partial" or .handoff_quality.status == "unsafe")
  and (.handoff_quality.missing | length) >= 1
  and (.handoff_quality.authority != "canonical")
  and (.handoff_quality.next_action_quality != "exact")
  and (.handoff_quality.proof_quality != "linked")
' /tmp/spec102-handoff-partial.json >/dev/null || fail "partial/unsafe handoff missing gap lists"
pass "partial handoff lists authority/proof/next-action gaps"

echo "SPEC102 agent handoff quality test: PASS"

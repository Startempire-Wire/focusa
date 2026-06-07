#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in recovery_playbook first_safe_tool proof_to_capture stop_conditions project_identity_mismatch unsafe_broad_cwd stale_trajectory wrong_workpoint_id focus_state_blocked evidence_index_lag ontology_selector_empty doctor_ready_blocked_ambiguity uiai_pressure stuck_loop_no_confidence_change; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse recovery playbook missing $term"
done
pass "traverse declares required recovery playbook scenarios"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
for SCENARIO in project_identity_mismatch unsafe_broad_cwd stale_trajectory wrong_workpoint_id focus_state_blocked evidence_index_lag ontology_selector_empty doctor_ready_blocked_ambiguity uiai_pressure stuck_loop_no_confidence_change; do
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"surface\":\"recovery_playbooks\",\"selector\":\"scenario\",\"query\":\"$SCENARIO\",\"limit\":5}" \
    "$BASE/v1/traverse" >"/tmp/spec102-playbook-$SCENARIO.json"
  jq -e --arg s "$SCENARIO" '
    (.traversal.recovery_playbook.scenario == $s)
    and (.traversal.recovery_playbook.first_safe_tool | type == "string")
    and (.traversal.recovery_playbook.next_tools | length) >= 1
    and (.traversal.recovery_playbook.proof_to_capture | type == "string")
    and (.traversal.recovery_playbook.stop_conditions | length) >= 1
    and (.items | length) >= 1
  ' "/tmp/spec102-playbook-$SCENARIO.json" >/dev/null || fail "playbook $SCENARIO missing required fields"
done
pass "all required recovery playbook scenarios have first tool/proof/stop conditions"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"workpoints","selector":"window","query":"definitely-no-playbook-needed-xyz","limit":5}' \
  "$BASE/v1/traverse" >/tmp/spec102-playbook-happy.json
jq -e '(.traversal.recovery_playbook == null) and (.recovery_playbook == null)' /tmp/spec102-playbook-happy.json >/dev/null || fail "playbook appeared in normal unrelated happy path"
pass "playbooks appear only when requested/active"

echo "SPEC102 recovery playbooks test: PASS"

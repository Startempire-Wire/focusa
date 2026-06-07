#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="spec102-trajectory-workpoint-reconcile-$(date +%s)-$$"
TMP_DIR="${TMPDIR:-/tmp}/spec102-trajectory-workpoint-reconcile"
mkdir -p "${TMP_DIR}"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
urlencode(){ python3 -c 'import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=""))' "$1"; }

ROOT_Q="$(urlencode "$PROJECT_ROOT")"
CONT_Q="$(urlencode "$CONTINUITY_ID")"

jq -n --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" \
  '{project_root:$root, continuity_id:$cont, mission:"Spec102 trajectory/workpoint reconciliation", current_action:"spec102_reconciliation_test", next_slice:"Verify trajectory/workpoint reconciliation", canonical:true}' \
  >"${TMP_DIR}/checkpoint-body.json"

curl -fsS --max-time 15 -X POST "${BASE}/v1/workpoint/checkpoint" \
  -H 'content-type: application/json' \
  --data-binary @"${TMP_DIR}/checkpoint-body.json" \
  >"${TMP_DIR}/checkpoint.json"

workpoint_id="$(jq -r '.workpoint_id // empty' "${TMP_DIR}/checkpoint.json")"
[[ -n "$workpoint_id" && "$workpoint_id" != "null" ]] || fail "checkpoint missing workpoint_id"

curl -fsS --max-time 15 -X POST "${BASE}/v1/workpoint/resume" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"mode\":\"compact_prompt\"}" \
  >"${TMP_DIR}/workpoint-resume.json"

jq -e --arg wp "$workpoint_id" '.status == "completed" and .canonical == true and .workpoint_id == $wp' "${TMP_DIR}/workpoint-resume.json" >/dev/null \
  || fail "Workpoint resume should be canonical before reconciliation check"
pass "canonical Workpoint exists for reconciliation test"

curl -fsS --max-time 15 "${BASE}/v1/trajectory/view?project_root=${ROOT_Q}&continuity_id=${CONT_Q}&mode=summary" \
  >"${TMP_DIR}/trajectory-view.json"

jq -e --arg wp "$workpoint_id" '
  .trajectory_workpoint_reconciliation.workpoint_status == "canonical"
  and .trajectory_workpoint_reconciliation.workpoint_id == $wp
  and (.trajectory_workpoint_reconciliation.trajectory_status | type == "string")
  and (.trajectory_workpoint_reconciliation.authority_for_next_action | type == "string")
  and (.trajectory_workpoint_reconciliation.next_repair_tool | type == "string")
  and .intelligence_view.trajectory_workpoint_reconciliation.workpoint_id == $wp
' "${TMP_DIR}/trajectory-view.json" >/dev/null \
  || fail "trajectory view must include explicit Workpoint/Trajectory reconciliation card when canonical Workpoint exists"
pass "trajectory view exposes reconciliation card for canonical Workpoint + provisional trajectory"

jq -n --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" \
  '{project_root:$root, continuity_id:$cont, long_term_goal:"Spec102 reconciliation happy path", desired_end_state:"Trajectory and Workpoint are aligned for next action", current_state:"Canonical Workpoint exists", mid_level_goal:"Run reconciliation check", short_term_goal:"Verify calm aligned card", waypoints:["checkpoint", "view"], goal_source:"operator", operator_confirmed:true}' \
  >"${TMP_DIR}/define-goal-body.json"

curl -fsS --max-time 15 -X POST "${BASE}/v1/trajectory/define-goal" \
  -H 'content-type: application/json' \
  --data-binary @"${TMP_DIR}/define-goal-body.json" \
  >"${TMP_DIR}/define-goal.json"

curl -fsS --max-time 15 "${BASE}/v1/trajectory/view?project_root=${ROOT_Q}&continuity_id=${CONT_Q}&mode=summary" \
  >"${TMP_DIR}/trajectory-view-aligned.json"

jq -e --arg wp "$workpoint_id" '
  .trajectory_workpoint_reconciliation.workpoint_status == "canonical"
  and .trajectory_workpoint_reconciliation.workpoint_id == $wp
  and .trajectory_workpoint_reconciliation.resolution == "aligned"
  and (.trajectory_workpoint_reconciliation.conflicts | length == 0)
' "${TMP_DIR}/trajectory-view-aligned.json" >/dev/null \
  || fail "aligned happy path should render calm reconciliation without conflict/scar text"
pass "aligned trajectory/workpoint happy path is calm"

echo "SPEC102 trajectory/workpoint reconciliation runtime test: PASS"

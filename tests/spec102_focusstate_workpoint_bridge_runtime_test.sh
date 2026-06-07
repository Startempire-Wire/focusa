#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="spec102-focusstate-workpoint-bridge-$(date +%s)-$$"
TMP_DIR="${TMPDIR:-/tmp}/spec102-focusstate-workpoint-bridge"
mkdir -p "$TMP_DIR"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

jq -n --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" \
  '{project_root:$root, continuity_id:$cont, mission:"Spec102 FocusState Workpoint bridge", current_action:"spec102_focusstate_bridge_test", next_slice:"Verify Focus State bridge", canonical:true}' \
  >"$TMP_DIR/checkpoint-body.json"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/checkpoint" \
  -H 'content-type: application/json' --data-binary @"$TMP_DIR/checkpoint-body.json" \
  >"$TMP_DIR/checkpoint.json"

workpoint_id="$(jq -r '.workpoint_id // empty' "$TMP_DIR/checkpoint.json")"
[[ -n "$workpoint_id" && "$workpoint_id" != "null" ]] || fail "checkpoint missing workpoint_id"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/resume" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"mode\":\"compact_prompt\"}" \
  >"$TMP_DIR/workpoint-resume.json"

jq -e --arg wp "$workpoint_id" '.status == "completed" and .canonical == true and .workpoint_id == $wp' "$TMP_DIR/workpoint-resume.json" >/dev/null \
  || fail "Workpoint resume should be canonical before Focus State bridge check"
pass "canonical Workpoint exists for Focus State bridge test"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/update" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"delta\":{\"notes\":[\"Spec102 bridge probe\"]}}" \
  >"$TMP_DIR/focus-update-no-frame.json"

jq -e --arg wp "$workpoint_id" '
  .status == "no_active_frame"
  and .canonical == false
  and .focus_state_workpoint_bridge.workpoint_status == "canonical"
  and .focus_state_workpoint_bridge.workpoint_id == $wp
  and .focus_state_workpoint_bridge.focus_state_status == "missing_project_bound_frame"
  and .focus_state_workpoint_bridge.authority_for_next_action == "workpoint"
  and (.focus_state_workpoint_bridge.next_repair_tool | type == "string")
' "$TMP_DIR/focus-update-no-frame.json" >/dev/null \
  || fail "Focus State write block must reconcile with canonical Workpoint and name repair route"
pass "blocked Focus State write exposes Workpoint bridge"

curl -fsS --max-time 15 -X POST "$BASE/v1/session/start" \
  -H 'content-type: application/json' \
  -d "{\"adapter_id\":\"spec102\",\"workspace_id\":\"$PROJECT_ROOT\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}" \
  >"$TMP_DIR/session-start.json"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/push" \
  -H 'content-type: application/json' \
  -d "{\"title\":\"Spec102 FocusState bridge\",\"goal\":\"Verify happy path\",\"beads_issue_id\":\"focusa-pm2b.4\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}" \
  >"$TMP_DIR/focus-push.json"

frame_id="$(jq -r '.frame_id // empty' "$TMP_DIR/focus-push.json")"
[[ -n "$frame_id" && "$frame_id" != "null" ]] || fail "focus push missing frame_id"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/update" \
  -H 'content-type: application/json' \
  -d "{\"frame_id\":\"$frame_id\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"delta\":{\"notes\":[\"Spec102 bridge happy path\"]}}" \
  >"$TMP_DIR/focus-update-happy.json"

jq -e '
  .status == "accepted"
  and (.focus_state_workpoint_bridge // null) == null
  and (.safe_recovery // null) == null
' "$TMP_DIR/focus-update-happy.json" >/dev/null \
  || fail "happy Focus State write should remain clean without bridge/scar fields"
pass "happy Focus State write has no bridge/scar fields"

echo "SPEC102 FocusState/Workpoint bridge runtime test: PASS"

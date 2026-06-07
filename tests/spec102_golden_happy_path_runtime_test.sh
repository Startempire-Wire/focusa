#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="spec102-golden-happy-path-$(date +%s)-$$"
TMP_DIR="${TMPDIR:-/tmp}/spec102-golden-happy-path"
mkdir -p "$TMP_DIR"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
urlencode(){ python3 -c 'import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=""))' "$1"; }

ROOT_Q="$(urlencode "$PROJECT_ROOT")"
CONT_Q="$(urlencode "$CONTINUITY_ID")"

curl -fsS --max-time 15 "$BASE/v1/project/identity?project_root=$ROOT_Q" > "$TMP_DIR/project-identity.json"
jq -e --arg root "$PROJECT_ROOT" '(.project_identity.status == "verified" or .project_identity.project_summary.project.status == "verified") and .project_identity.project_root == $root' "$TMP_DIR/project-identity.json" >/dev/null \
  || fail "project identity must verify exact Focusa root"
pass "project identity verifies exact root"

jq -n --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" \
  '{project_root:$root, continuity_id:$cont, mission:"Spec102 golden real-life happy path", current_action:"spec102_golden_happy_path", next_slice:"Complete clean golden flow", canonical:true, active_object_refs:["docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md"], verification_records:[{target_ref:"Spec102Golden", result:"golden flow initialized", evidence_ref:"spec102-golden-init"}]}' \
  >"$TMP_DIR/checkpoint-body.json"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/checkpoint" \
  -H 'content-type: application/json' --data-binary @"$TMP_DIR/checkpoint-body.json" \
  >"$TMP_DIR/checkpoint.json"
workpoint_id="$(jq -r '.workpoint_id // empty' "$TMP_DIR/checkpoint.json")"
[[ -n "$workpoint_id" && "$workpoint_id" != "null" ]] || fail "checkpoint missing workpoint_id"
pass "Workpoint checkpoint created"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/resume" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"mode\":\"compact_prompt\"}" \
  >"$TMP_DIR/resume-clean.json"

jq -e --arg wp "$workpoint_id" '
  .status == "completed"
  and .canonical == true
  and .workpoint_id == $wp
  and (.schema_version == "focusa.workpoint_resume_packet.v2")
  and ((.fallback_used // false) == false)
  and (.requested_workpoint_id // null) == null
  and (.rendered_summary | test("WORKPOINT"))
' "$TMP_DIR/resume-clean.json" >/dev/null \
  || fail "clean Workpoint resume must be canonical v2 with no fallback/requested-id scar"
pass "clean Workpoint resume is canonical and scar-free"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/evidence/link" \
  -H 'content-type: application/json' \
  -d "{\"workpoint_id\":\"$workpoint_id\",\"target_ref\":\"Spec102Golden\",\"result\":\"golden flow proof linked\",\"evidence_ref\":\"spec102-golden-evidence-$CONTINUITY_ID\"}" \
  >"$TMP_DIR/evidence-link.json"

jq -e --arg wp "$workpoint_id" '(.status == "accepted" or .status == "pending") and .canonical == true and .workpoint_id == $wp' "$TMP_DIR/evidence-link.json" >/dev/null \
  || fail "evidence link should attach or pending-link to canonical Workpoint"
pass "evidence linked or pending-linked to Workpoint"

curl -fsS --max-time 15 -X POST "$BASE/v1/trajectory/define-goal" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"long_term_goal\":\"Spec102 golden clean agent UX\",\"desired_end_state\":\"Fresh tester completes Focusa golden flow without noticing repaired issues\",\"current_state\":\"Canonical Workpoint and evidence exist\",\"mid_level_goal\":\"Run golden happy-path regression\",\"short_term_goal\":\"Verify clean resume, evidence, trajectory, focus bridge, and drift flow\",\"waypoints\":[\"verify project\",\"checkpoint\",\"resume\",\"link evidence\",\"trajectory view\",\"focus state\",\"drift check\"],\"goal_source\":\"operator\",\"operator_confirmed\":true,\"required_evidence_refs\":[\"spec102-golden-evidence-$CONTINUITY_ID\"]}" \
  >"$TMP_DIR/define-goal.json"

curl -fsS --max-time 15 "$BASE/v1/trajectory/view?project_root=$ROOT_Q&continuity_id=$CONT_Q&mode=summary" \
  >"$TMP_DIR/trajectory-view.json"

jq -e --arg wp "$workpoint_id" '
  .trajectory_workpoint_reconciliation.workpoint_status == "canonical"
  and .trajectory_workpoint_reconciliation.workpoint_id == $wp
  and .trajectory_workpoint_reconciliation.resolution == "aligned"
  and (.trajectory_workpoint_reconciliation.conflicts | length == 0)
  and .intelligence_view.trajectory_workpoint_reconciliation.workpoint_id == $wp
' "$TMP_DIR/trajectory-view.json" >/dev/null \
  || fail "trajectory view should show calm aligned Workpoint reconciliation"
pass "trajectory and Workpoint reconciliation aligned"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/update" \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"delta\":{\"notes\":[\"Spec102 golden bridge probe\"]}}" \
  >"$TMP_DIR/focus-update-bridge.json"

jq -e --arg wp "$workpoint_id" '
  .status == "no_active_frame"
  and .focus_state_workpoint_bridge.workpoint_status == "canonical"
  and .focus_state_workpoint_bridge.workpoint_id == $wp
  and .focus_state_workpoint_bridge.authority_for_next_action == "workpoint"
' "$TMP_DIR/focus-update-bridge.json" >/dev/null \
  || fail "Focus State blocked path should bridge to canonical Workpoint"
pass "Focus State blocked path bridges to Workpoint"

curl -fsS --max-time 15 -X POST "$BASE/v1/session/start" \
  -H 'content-type: application/json' \
  -d "{\"adapter_id\":\"spec102\",\"workspace_id\":\"$PROJECT_ROOT\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}" \
  >"$TMP_DIR/session-start.json"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/push" \
  -H 'content-type: application/json' \
  -d "{\"title\":\"Spec102 golden Focus State\",\"goal\":\"Clean Focus State happy path\",\"beads_issue_id\":\"focusa-pm2b.25\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}" \
  >"$TMP_DIR/focus-push.json"
frame_id="$(jq -r '.frame_id // empty' "$TMP_DIR/focus-push.json")"
[[ -n "$frame_id" && "$frame_id" != "null" ]] || fail "focus push missing frame_id"

curl -fsS --max-time 15 -X POST "$BASE/v1/focus/update" \
  -H 'content-type: application/json' \
  -d "{\"frame_id\":\"$frame_id\",\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\",\"delta\":{\"notes\":[\"Spec102 golden happy path\"]}}" \
  >"$TMP_DIR/focus-update-happy.json"

jq -e '.status == "accepted" and (.focus_state_workpoint_bridge // null) == null and (.safe_recovery // null) == null' "$TMP_DIR/focus-update-happy.json" >/dev/null \
  || fail "Focus State happy path should be accepted without bridge/scar fields"
pass "Focus State happy path is clean"

curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/drift-check" \
  -H 'content-type: application/json' \
  -d "{\"workpoint_id\":\"$workpoint_id\",\"latest_action\":\"spec102_golden_happy_path docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md\",\"expected_action_type\":\"spec102_golden_happy_path\",\"emit\":false}" \
  >"$TMP_DIR/drift-check.json"

jq -e '(.status == "completed" or .status == "no_drift") and (.drift_detected == false or .decision.drift_detected == false)' "$TMP_DIR/drift-check.json" >/dev/null \
  || fail "golden flow drift check should remain calm/no drift"
pass "drift check is calm"

# No-residual UX spot checks: normal happy-path artifacts should not mention previous fixes or issue history.
for file in resume-clean.json trajectory-view.json focus-update-happy.json drift-check.json; do
  if rg -i 'previous issue|previously fixed|scar|split-brain|silent fallback bug|repair history' "$TMP_DIR/$file" >/dev/null; then
    fail "residual repair/scar language leaked into $file"
  fi
done
pass "no residual repair/scar language in happy-path outputs"

echo "SPEC102 golden real-life happy-path runtime test: PASS artifacts=$TMP_DIR"

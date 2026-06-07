#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in requested_project_root persisted_project_root verified_project_root matched_axes mismatched_axes authority_decision safe_next_action ProjectIdentityMismatchSemantics; do
  rg -F "$term" crates/focusa-api/src/routes/project.rs >/dev/null || fail "project mismatch semantics missing $term"
done
pass "project route declares mismatch comparison terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }

curl -fsS --max-time 15 -H 'Content-Type: application/json' -d "{\"project_root\":\"$ROOT_DIR\",\"persisted_project_root\":\"$ROOT_DIR\",\"project_id\":\"focusa\"}" "$BASE/v1/project/verify" >/tmp/spec102-project-match.json
jq -e '
  .status == "completed"
  and .verification.verified == true
  and .project_identity.status == "verified"
  and (.project_identity.project_root == "'$ROOT_DIR'")
  and (.mismatch_semantics == null or .mismatch_semantics.mismatched_axes == [])
  and (.failure_class == null)
' /tmp/spec102-project-match.json >/dev/null || fail "matching project path not calm/verified"
pass "matching project happy path stays calm"

curl -fsS --max-time 15 -H 'Content-Type: application/json' -d "{\"project_root\":\"$ROOT_DIR\",\"persisted_project_root\":\"/home/wpuiai/uiai-engine\",\"project_id\":\"focusa\"}" "$BASE/v1/project/verify" >/tmp/spec102-project-mismatch.json
jq -e '
  .status == "mismatch"
  and .verification.verified == false
  and .details.tool_result_v1.failure_class == "scope_mismatch"
  and .mismatch_semantics.requested_project_root == "'$ROOT_DIR'"
  and .mismatch_semantics.persisted_project_root == "/home/wpuiai/uiai-engine"
  and .mismatch_semantics.verified_project_root == "'$ROOT_DIR'"
  and (.mismatch_semantics.matched_axes | index("requested_project_root==verified_project_root"))
  and (.mismatch_semantics.mismatched_axes | index("persisted_project_root!=requested_project_root"))
  and .mismatch_semantics.authority_decision == "operator_confirmation_required"
  and (.mismatch_semantics.safe_next_action | test("focusa_project_verify|focusa_project_identity|focusa_workpoint_checkpoint"))
  and (.next_tools | index("focusa_project_verify"))
  and (.next_tools | index("focusa_workpoint_checkpoint"))
' /tmp/spec102-project-mismatch.json >/dev/null || fail "mismatch path lacks axes/operator/safe next semantics"
pass "mismatch path explains axes and safe next action"

echo "SPEC102 project identity mismatch semantics test: PASS"

#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61}"
export CONTINUITY_ID

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
post_json(){
  local path="$1" payload="$2" out status
  out=$(mktemp)
  status=$(curl -sS -o "$out" -w '%{http_code}' -H 'content-type: application/json' -X POST "$BASE_URL$path" --data "$payload")
  cat "$out"
  rm -f "$out"
  echo "HTTP_STATUS=$status" >&2
}

curl -fsS --max-time 5 "$BASE_URL/v1/health" >/dev/null || fail "daemon health unreachable"
pass "daemon health reachable"

missing_continuity=$(post_json '/v1/context-cognition/curate/eval' "$(jq -nc --arg root "$PROJECT_ROOT" '{project_root:$root,target:"scope",candidates:[],expected_selected_paths:[]}')" 2>/tmp/ctx_eval_status)
grep -q 'HTTP_STATUS=422' /tmp/ctx_eval_status || fail "missing continuity eval did not reject with 422"
jq -e '.failure_class == "continuity_id_missing"' <<<"$missing_continuity" >/dev/null || fail "missing continuity failure_class mismatch"
pass "curate eval rejects missing continuity_id"

bad_scope=$(post_json '/v1/context-cognition/curate/eval' "$(jq -nc --arg root '/home/wirebot/.cargo' --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,target:"scope",candidates:[],expected_selected_paths:[]}')" 2>/tmp/ctx_eval_status)
grep -q 'HTTP_STATUS=422' /tmp/ctx_eval_status || fail "bad scope eval did not reject with 422"
jq -e '.failure_class == "scope_mismatch"' <<<"$bad_scope" >/dev/null || fail "bad scope failure_class mismatch"
pass "curate eval rejects unsafe/cross-project scope"

promoted_eval=$(post_json '/v1/context-cognition/curate/eval' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,case_id:"spec106-exact-selection",target:"auth route Workpoint evidence",token_budget:30,candidates:[{kind:"file",path:"auth.ts",body:"auth route Workpoint evidence",tokens:30},{kind:"doc",path:"noise.md",body:"unrelated marketing prose",tokens:30}],expected_selected_paths:["auth.ts"],score_threshold:0.9,baseline_f1:0.1}')" 2>/tmp/ctx_eval_status)
grep -q 'HTTP_STATUS=200' /tmp/ctx_eval_status || fail "promoted eval did not return 200"
jq -e '.promoted == true and .f1 == 1 and .selected_paths == ["auth.ts"] and .scope_status == "matched" and .continuity_id == env.CONTINUITY_ID' <<<"$promoted_eval" >/dev/null || fail "promoted eval result mismatch"
run_id=$(jq -r '.run_id' <<<"$promoted_eval")
pass "curate eval promotes exact selected critical file"

over_broad_eval=$(post_json '/v1/context-cognition/curate/eval' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,case_id:"spec106-overbroad-selection",target:"auth route Workpoint evidence",token_budget:120,candidates:[{kind:"file",path:"auth.ts",body:"auth route Workpoint evidence",tokens:30},{kind:"doc",path:"noise.md",body:"unrelated marketing prose",tokens:30}],expected_selected_paths:["auth.ts"],score_threshold:0.9,baseline_f1:0.1}')" 2>/tmp/ctx_eval_status)
grep -q 'HTTP_STATUS=200' /tmp/ctx_eval_status || fail "over-broad eval did not return 200"
jq -e '.promoted == false and .precision == 0.5 and .recall == 1 and (.selected_paths | index("noise.md"))' <<<"$over_broad_eval" >/dev/null || fail "over-broad eval result mismatch"
pass "curate eval catches over-broad context selection"

under_eval=$(post_json '/v1/context-cognition/curate/eval' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,case_id:"spec106-under-selected-critical",target:"critical route",token_budget:20,candidates:[{kind:"file",path:"critical.ts",body:"critical route",tokens:80}],expected_selected_paths:["critical.ts"],score_threshold:0.5,baseline_f1:0.0}')" 2>/tmp/ctx_eval_status)
grep -q 'HTTP_STATUS=200' /tmp/ctx_eval_status || fail "under-selected eval did not return 200"
jq -e '.promoted == false and .recall == 0 and .f1 == 0' <<<"$under_eval" >/dev/null || fail "under-selected eval result mismatch"
pass "curate eval catches under-selected critical file"

invalid_score=$(post_json '/v1/context-cognition/curate/optimize' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,prompt_artifact_ref:"artifact://bad",eval_score:1.2,baseline_score:0.0,score_threshold:0.5}')" 2>/tmp/ctx_opt_status)
grep -q 'HTTP_STATUS=422' /tmp/ctx_opt_status || fail "invalid optimizer score did not reject with 422"
jq -e '.failure_class == "score_out_of_range" and .field == "eval_score"' <<<"$invalid_score" >/dev/null || fail "invalid optimizer score failure mismatch"
pass "curate optimize rejects invalid score"

promote_opt=$(post_json '/v1/context-cognition/curate/optimize' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" --arg run "$run_id" '{project_root:$root,continuity_id:$cid,module_name:"curator",prompt_artifact_ref:"artifact://candidate-promote",eval_score:0.9,baseline_score:0.5,score_threshold:0.8,eval_run_id:$run}')" 2>/tmp/ctx_opt_status)
grep -q 'HTTP_STATUS=200' /tmp/ctx_opt_status || fail "optimizer promote did not return 200"
jq -e '.decision == "promote" and .promoted == true and .continuity_id == env.CONTINUITY_ID' <<<"$promote_opt" >/dev/null || fail "optimizer promote result mismatch"
pass "curate optimize promotes only eval-backed improvement above threshold"

rollback_opt=$(post_json '/v1/context-cognition/curate/optimize' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,module_name:"curator",prompt_artifact_ref:"artifact://candidate-rollback",eval_score:0.4,baseline_score:0.5,score_threshold:0.8,rollback:true}')" 2>/tmp/ctx_opt_status)
grep -q 'HTTP_STATUS=200' /tmp/ctx_opt_status || fail "optimizer rollback did not return 200"
jq -e '.decision == "rollback" and .promoted == false and (.rollback_ref != null)' <<<"$rollback_opt" >/dev/null || fail "optimizer rollback result mismatch"
pass "curate optimize rolls back weak/explicit rollback candidates"

echo "context cognition eval optimizer hardening live test: PASS"

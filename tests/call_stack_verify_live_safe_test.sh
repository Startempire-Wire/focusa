#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-/home/wirebot/focusa}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61}"
FOCUSA_CLI="${FOCUSA_CLI:-$PWD/target/release/focusa}"

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

design_payload=$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,mission:"Verify call stack verifier",entry_surface:"http_route",entry_name:"/v1/call-stack/verify",attach_to_workpoint:false,attach_to_stg:false}')
design=$(post_json '/v1/call-stack/design' "$design_payload" 2>/tmp/call_stack_design_status)
grep -q 'HTTP_STATUS=200' /tmp/call_stack_design_status || fail "call stack design did not return 200"
design_id=$(jq -r '.design_id' <<<"$design")
[ -n "$design_id" ] && [ "$design_id" != "null" ] || fail "design_id missing"
export design_id
pass "call stack design created for verifier route"

verify=$(post_json '/v1/call-stack/verify' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" --arg id "$design_id" '{project_root:$root,continuity_id:$cid,design_id:$id}')" 2>/tmp/call_stack_verify_status)
grep -q 'HTTP_STATUS=200' /tmp/call_stack_verify_status || fail "call stack verify did not return 200"
jq -e '.status == "completed" and .canonical == false and .advisory == true and .design_id == env.design_id and (.checks | length) >= 7 and (.drift_status == "needs_review" or .drift_status == "aligned")' <<<"$verify" >/dev/null || fail "verify response shape mismatch"
jq -e '.checks[] | select(.id == "entry_surface_exists" and .status == "pass")' <<<"$verify" >/dev/null || fail "entry surface existence check did not pass"
jq -e '.checks[] | select(.id == "output_envelope" and .status == "pass")' <<<"$verify" >/dev/null || fail "output envelope check did not pass"
pass "call stack verify checks actual route surface and tool_result_v1 envelope"

not_found=$(post_json '/v1/call-stack/verify' "$(jq -nc --arg root "$PROJECT_ROOT" --arg cid "$CONTINUITY_ID" '{project_root:$root,continuity_id:$cid,design_id:"missing-design-id"}')" 2>/tmp/call_stack_verify_status)
grep -q 'HTTP_STATUS=404' /tmp/call_stack_verify_status || fail "missing design did not return 404"
jq -e '.failure_class == "call_stack_design_not_found"' <<<"$not_found" >/dev/null || fail "missing design failure_class mismatch"
pass "call stack verify reports missing design distinctly"

[ -x "$FOCUSA_CLI" ] || fail "focusa CLI binary not executable at $FOCUSA_CLI"
cli_design=$("$FOCUSA_CLI" --json call-stack design --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --mission "CLI parity call stack" --entry-surface http_route --entry-name /v1/call-stack/list)
cli_design_id=$(jq -r '.design_id' <<<"$cli_design")
[ -n "$cli_design_id" ] && [ "$cli_design_id" != "null" ] || fail "CLI design_id missing"
"$FOCUSA_CLI" call-stack list --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --limit 5 | rg 'call-stack list: count=' >/dev/null || fail "CLI list output mismatch"
"$FOCUSA_CLI" call-stack show --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --design-id "$cli_design_id" | rg 'call-stack show:' >/dev/null || fail "CLI show output mismatch"
"$FOCUSA_CLI" call-stack verify --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --design-id "$cli_design_id" | rg 'call-stack verify:' >/dev/null || fail "CLI verify output mismatch"
pass "CLI call-stack design/list/show/verify parity works"

echo "call stack verify live-safe test: PASS"

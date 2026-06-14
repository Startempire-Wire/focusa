#!/usr/bin/env bash
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
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

bad_scope=$(post_json '/v1/device/pair/start' '{"device_name":"bad-scope","scopes":["read","admin"]}' 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=422' /tmp/device_pair_status || fail "bad scope did not reject with 422"
jq -e '.failure_class == "scope_not_allowed"' <<<"$bad_scope" >/dev/null || fail "bad scope failure_class mismatch"
pass "pair-start rejects unknown scopes"

bad_url=$(post_json '/v1/device/pair/start' '{"device_name":"bad-url","daemon_base_url":"http://evil.example.com"}' 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=422' /tmp/device_pair_status || fail "bad URL did not reject with 422"
jq -e '.failure_class == "pairing_url_invalid" and .field == "daemon_base_url"' <<<"$bad_url" >/dev/null || fail "bad URL failure_class mismatch"
pass "pair-start rejects non-https non-local pairing URL"

start=$(post_json '/v1/device/pair/start' '{"device_name":"<Mac Script>alert</Mac>","platform":"Mac OS!!","daemon_base_url":"http://127.0.0.1:8787","scopes":["read","read","write"]}' 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=200' /tmp/device_pair_status || fail "pair-start did not return 200"
jq -e '.expires_in_secs == 300 and .scopes == ["read","write"] and (.device_name | test("^[A-Za-z0-9_. -]+$")) and (.platform | test("^[a-z0-9_. -]+$"))' <<<"$start" >/dev/null || fail "pair-start sanitized/normalized result mismatch"
code=$(jq -r '.code' <<<"$start")
device_id=$(jq -r '.device_id' <<<"$start")
pass "pair-start sanitizes labels, normalizes scopes, and preserves TTL"

unsafe_host=$(post_json '/v1/device/pair/complete' "$(jq -nc --arg code "$code" '{code:$code,host:"/home/wirebot/.cargo"}')" 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=422' /tmp/device_pair_status || fail "unsafe host did not reject with 422"
jq -e '.failure_class == "scope_mismatch" and .field == "host"' <<<"$unsafe_host" >/dev/null || fail "unsafe host failure_class mismatch"
pass "pair-complete rejects unsafe agent runtime host"

complete=$(post_json '/v1/device/pair/complete' "$(jq -nc --arg code "$code" '{code:$code,host:"operator-vps",operator_id:"verious<script>",completed_by:"vps-cli"}')" 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=200' /tmp/device_pair_status || fail "pair-complete did not return 200"
token=$(jq -r '.token' <<<"$complete")
[[ "$token" =~ ^[A-Za-z0-9_-]{43}$ ]] || fail "token is not 32-byte base64url-no-pad shape"
jq -e '.token_ttl_secs == 2592000 and .host == "operator-vps" and (._completion.operator_id | test("^[A-Za-z0-9_. -]+$"))' <<<"$complete" >/dev/null || fail "pair-complete token/label result mismatch"
pass "pair-complete mints 32-byte base64url token and sanitizes audit labels"

reuse=$(post_json '/v1/device/pair/complete' "$(jq -nc --arg code "$code" '{code:$code,host:"operator-vps"}')" 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=409' /tmp/device_pair_status || fail "code reuse did not return 409"
jq -e '.failure_class == "pair_code_already_used"' <<<"$reuse" >/dev/null || fail "code reuse failure_class mismatch"
pass "pair-complete enforces single-use code"

status=$(curl -fsS "$BASE_URL/v1/device/pair/status?code=$code")
jq -e --arg token "$token" '.status == "completed" and .token == $token and .expired == false' <<<"$status" >/dev/null || fail "pair-status completed token mismatch"
pass "pair-status returns completed token to joining device poller"

list=$(curl -fsS "$BASE_URL/v1/device/pair/list?host=operator-vps&limit=20")
jq -e --arg id "$device_id" '.devices[] | select(.device_id == $id and .revoked == false)' <<<"$list" >/dev/null || fail "pair-list missing paired record"
pass "pair-list exposes append-only paired device record"

revoke=$(post_json '/v1/device/pair/revoke' "$(jq -nc --arg id "$device_id" '{device_id:$id,host:"operator-vps",reason:"live-safe-test"}')" 2>/tmp/device_pair_status)
grep -q 'HTTP_STATUS=200' /tmp/device_pair_status || fail "pair-revoke did not return 200"
jq -e '.ledger_appended == true and .status == "completed"' <<<"$revoke" >/dev/null || fail "pair-revoke append result mismatch"
revoked_list=$(curl -fsS "$BASE_URL/v1/device/pair/list?host=operator-vps&limit=50")
jq -e --arg id "$device_id" '.devices[] | select(.device_id == $id and .revoked == true)' <<<"$revoked_list" >/dev/null || fail "pair-list missing revoked append record"
pass "pair-revoke appends revoked audit record"

echo "device pairing endpoint hardening live-safe test: PASS"

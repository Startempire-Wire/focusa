#!/usr/bin/env bash
# Release-grade pairing proof: CLI fallback code flow must complete without QR/PWA license.
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FOCUSA_BIN="${FOCUSA_BIN:-focusa}"
OUT="${FOCUSA_PAIRING_PROOF_OUT:-/tmp/focusa-device-pairing-cli-codeflow.jsonl}"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
post(){ curl -fsS -H 'content-type: application/json' -X POST "$BASE_URL$1" --data "$2"; }
get(){ curl -fsS "$BASE_URL$1"; }

: > "$OUT"
curl -fsS --max-time 5 "$BASE_URL/v1/health" >/dev/null || fail "daemon health unreachable at $BASE_URL"
"$FOCUSA_BIN" --version | tee /tmp/focusa-device-pairing-cli-version.txt >/dev/null || fail "focusa CLI unavailable: $FOCUSA_BIN"

start=$(post /v1/device/pair/start '{"device_name":"klmc-cli-codeflow","platform":"macos","daemon_base_url":"'"$BASE_URL"'","scopes":["read","write"]}')
echo "$start" | jq -c '{step:"start",status,code,device_id,pair_url}' | tee -a "$OUT" >/dev/null
code=$(jq -r '.code // empty' <<<"$start")
device_id=$(jq -r '.device_id // empty' <<<"$start")
[ -n "$code" ] && [ "$code" != null ] || fail "pair-start missing code"
[ -n "$device_id" ] && [ "$device_id" != null ] || fail "pair-start missing device_id"

complete=$(FOCUSA_API_URL="$BASE_URL" "$FOCUSA_BIN" device pair-complete "$code" --host klmc-vps --operator-id release-test --completed-by cli-codeflow 2>&1)
echo "$complete" | grep -q "device pair complete completed" \
  || fail "CLI pair-complete did not report completion: $complete"
echo "{\"step\":\"complete_cli\",\"reported_completed\":true}" | tee -a "$OUT" >/dev/null

status=$(get "/v1/device/pair/status?code=$code")
echo "$status" | jq -c '{step:"status_by_code",status,device_id,token_present:(.token|type=="string" and length>20),host}' | tee -a "$OUT" >/dev/null
jq -e '.status == "completed" and (.token|type == "string" and length > 20) and .device_id == "'"$device_id"'"' <<<"$status" >/dev/null \
  || fail "status by code did not return completed token"

list=$(get '/v1/device/pair/list?host=klmc-vps&limit=20')
echo "$list" | jq -c --arg did "$device_id" '{step:"list_before_revoke",has_device:any(.devices[]?; .device_id==$did and .revoked==false),count:(.devices|length)}' | tee -a "$OUT" >/dev/null
jq -e --arg did "$device_id" 'any(.devices[]?; .device_id==$did and .revoked==false)' <<<"$list" >/dev/null \
  || fail "paired device not listed active"

revoke=$(post /v1/device/pair/revoke "{\"device_id\":\"$device_id\",\"host\":\"klmc-vps\",\"reason\":\"klmc-cli-codeflow\"}")
echo "$revoke" | jq -c '{step:"revoke",status,device_id,ledger_appended}' | tee -a "$OUT" >/dev/null
jq -e '.status == "completed" and .ledger_appended == true' <<<"$revoke" >/dev/null \
  || fail "revoke did not append ledger entry"

list2=$(get '/v1/device/pair/list?host=klmc-vps&limit=20')
echo "$list2" | jq -c --arg did "$device_id" '{step:"list_after_revoke",has_revoked:any(.devices[]?; .device_id==$did and .revoked==true),count:(.devices|length)}' | tee -a "$OUT" >/dev/null
jq -e --arg did "$device_id" 'any(.devices[]?; .device_id==$did and .revoked==true)' <<<"$list2" >/dev/null \
  || fail "revoked device not listed"

pass "CLI pairing code flow completed, tokenized, listed, and revoked"
echo "Evidence: $OUT"

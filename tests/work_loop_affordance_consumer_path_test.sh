#!/bin/bash
# Runtime contract: transport-session affordance is consumed by continuation/runtime control surfaces.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-work-loop-continuation-test}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
scoped_curl(){
  command curl \
    -H "x-scope-project-root: ${PROJECT_ROOT}" \
    -H "x-scope-continuity-id: ${CONTINUITY_ID}" \
    "$@"
}
http_json(){ scoped_curl -sS "$@"; }

wait_for_jq(){
  local url="$1"
  local expr="$2"
  local tries="${3:-40}"
  local delay="${4:-0.15}"
  for _ in $(seq 1 "$tries"); do
    if scoped_curl -sS "$url" | jq -e "$expr" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay"
  done
  return 1
}

WORK_ITEM_ID="spec79-affordance-consumer"
WRITER_ID="spec79-affordance-consumer"
CHECKPOINT_RESP="$(http_json -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"${PROJECT_ROOT}\",\"continuity_id\":\"${CONTINUITY_ID}\",\"work_item_id\":\"${WORK_ITEM_ID}\",\"mission\":\"verify Workloop affordance consumption\",\"current_action\":\"affordance_consumer_contract\",\"next_slice\":\"verify transport affordance transitions\",\"canonical\":true}")"
WORKPOINT_ID="$(echo "$CHECKPOINT_RESP" | jq -r '.workpoint_id // empty')"
for _ in $(seq 1 40); do
  RESUME_RESP="$(http_json -X POST "${BASE_URL}/v1/workpoint/resume" \
    -H 'Content-Type: application/json' \
    -d "{\"project_root\":\"${PROJECT_ROOT}\",\"continuity_id\":\"${CONTINUITY_ID}\",\"mode\":\"compact_prompt\"}")"
  echo "$RESUME_RESP" | jq -e --arg id "$WORKPOINT_ID" '.canonical == true and .workpoint_id == $id' >/dev/null 2>&1 && break
  sleep 0.1
done
ENABLE_RESP="$(http_json -X POST "${BASE_URL}/v1/work-loop/enable" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" \
  -H 'x-focusa-approval: approved' \
  -d "{\"preset\":\"balanced\",\"root_work_item_id\":\"${WORK_ITEM_ID}\"}")"
FENCING_TOKEN="$(echo "$ENABLE_RESP" | jq -r '.fencing_token // empty')"
FENCING_HEADERS=()
if echo "$FENCING_TOKEN" | grep -Eq '^[1-9][0-9]*$'; then
  FENCING_HEADERS=(-H "x-focusa-fencing-token: ${FENCING_TOKEN}")
else
  log_fail "work-loop writer lease unavailable: $ENABLE_RESP"
fi
STATUS0="$(http_json "${BASE_URL}/v1/work-loop/status")"
ORIG_ADAPTER="$(echo "$STATUS0" | jq -r '.transport.adapter // "pi-rpc"')"
ORIG_SESSION_ID="$(echo "$STATUS0" | jq -r '.transport.daemon_supervised_session.session_id // "spec79-affordance-restore"')"

ABORT_RESP="$(http_json -X POST "${BASE_URL}/v1/work-loop/session/abort" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" \
  "${FENCING_HEADERS[@]}" \
  -d '{"reason":"spec79 affordance consumer-path contract"}')"
if echo "$ABORT_RESP" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "transport session abort accepted"
else
  log_fail "transport session abort rejected: $ABORT_RESP"
fi

if wait_for_jq "${BASE_URL}/v1/work-loop/status" '.execution_environment.facts | any(.fact_key=="transport_session_attached" and .fact_value==false)'; then
  log_pass "execution-environment fact reflects detached transport session"
else
  log_fail "transport_session_attached fact did not transition to false"
fi

if wait_for_jq "${BASE_URL}/v1/work-loop/status" '.execution_environment.affordances | any(.id=="affordance_safe_local_code_edit" and .affordance_kind=="safe_local_edit_available" and .status=="blocked")'; then
  log_pass "safe_local_edit affordance transitions to blocked when transport detaches"
else
  log_fail "safe_local_edit affordance did not transition to blocked"
fi

HEARTBEAT_RESP="$(http_json -X POST "${BASE_URL}/v1/work-loop/heartbeat" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" \
  "${FENCING_HEADERS[@]}" \
  -d '{}')"
if echo "$HEARTBEAT_RESP" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "heartbeat dispatch control remains callable while affordance is blocked"
else
  log_fail "heartbeat dispatch control failed under blocked affordance: $HEARTBEAT_RESP"
fi

ATTACH_RESP="$(http_json -X POST "${BASE_URL}/v1/work-loop/session/attach" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" \
  "${FENCING_HEADERS[@]}" \
  -d "{\"adapter\":\"${ORIG_ADAPTER}\",\"session_id\":\"${ORIG_SESSION_ID}\"}")"
if echo "$ATTACH_RESP" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "transport session attach accepted"
else
  log_fail "transport session attach rejected: $ATTACH_RESP"
fi

if wait_for_jq "${BASE_URL}/v1/work-loop/status" '.execution_environment.facts | any(.fact_key=="transport_session_attached" and .fact_value==true)'; then
  log_pass "execution-environment fact reflects re-attached transport session"
else
  log_fail "transport_session_attached fact did not return to true"
fi

if wait_for_jq "${BASE_URL}/v1/work-loop/status" '.execution_environment.affordances | any(.id=="affordance_safe_local_code_edit" and .status=="available")'; then
  log_pass "safe_local_edit affordance returns to available after re-attach"
else
  log_fail "safe_local_edit affordance did not return to available"
fi

echo "=== WORK-LOOP AFFORDANCE CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

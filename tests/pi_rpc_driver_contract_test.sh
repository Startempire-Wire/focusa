#!/bin/bash
# SPEC-79 daemon-owned Pi RPC driver contract
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
curl() {
  command curl \
    -H "x-scope-project-root: ${ROOT_DIR}" \
    -H "x-scope-continuity-id: work-loop-pi-driver-test" \
    "$@"
}
http_json(){ curl -sS "$@"; }
REPO_ROOT="$ROOT_DIR"
WORK_LOOP_ROUTE_FILE="${REPO_ROOT}/crates/focusa-api/src/routes/work_loop.rs"
source "$REPO_ROOT/tests/fixtures/admitted-project-scope.sh"
focusa_test_scope_create "$BASE_URL" work-loop-pi-driver-test
trap focusa_test_scope_cleanup EXIT
ROOT_DIR="$FOCUSA_FIXTURE_ROOT"
if rg -n '/v1/work-loop/driver/start|/v1/work-loop/driver/prompt|/v1/work-loop/driver/abort|/v1/work-loop/driver/stop' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Pi RPC driver routes are registered"
else
  log_fail "Pi RPC driver routes missing"
fi

if rg -n 'never push, deploy, merge, or release' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'must not be reported as blockers' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Pi turn packets exclude prohibited delivery from blocker outcomes"
else
  log_fail "Pi can treat prohibited delivery actions as acceptance blockers"
fi

if rg -Fq 'if kind == "agent_settled"' "$WORK_LOOP_ROUTE_FILE" \
  && ! rg -q 'if .*kind == "agent_end"' "$WORK_LOOP_ROUTE_FILE"; then
  log_pass "Pi outcome governance waits for the outer agent run to settle"
else
  log_fail "Pi outcome governance is not bound to final agent_settled"
fi

if rg -n '"--no-extensions"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"--no-skills"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"--no-prompt-templates"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'FOCUSA_PI_VITAL_INFO_PROMPT_MODE' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Supervised Pi is orchestration-isolated and non-interactive"
else
  log_fail "Supervised Pi can load competing orchestration or block on UI"
fi

if rg -Fq 'process_group(0)' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq 'kill_on_drop(true)' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq '.filter(|pid| *pid > 1)' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq 'args(["-TERM", "--", &pgid])' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq 'args(["-KILL", "--", &pgid])' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq '/bin/kill -TERM -- "-$2"' "$WORK_LOOP_ROUTE_FILE" \
  && rg -Fq '/bin/kill -KILL -- "-$2"' "$WORK_LOOP_ROUTE_FILE"; then
  log_pass "Pi RPC driver and watchdog signal only the supervised group"
else
  log_fail "Unsafe Pi RPC process-group selector; refusing to start the live driver test"
  exit 1
fi
http_json -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"${ROOT_DIR}\",\"continuity_id\":\"work-loop-pi-driver-test\",\"work_item_id\":\"spec79-pi-driver\",\"mission\":\"verify Pi RPC driver contract\",\"action_intent\":{\"action_type\":\"spec79_pi_rpc_driver\",\"lifecycle_stage\":\"verify_outcome\",\"status\":\"ready\"},\"next_slice\":\"verify scoped driver lifecycle\",\"canonical\":true}" >/dev/null
WRITER_ID="spec79-pi-driver"
ENABLE_RESP=$(http_json -X POST "${BASE_URL}/v1/work-loop/enable" -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" -H 'x-focusa-approval: approved' \
  -d '{"preset":"balanced","root_work_item_id":"spec79-pi-driver"}')
FENCING_TOKEN=$(echo "$ENABLE_RESP" | jq -r '.fencing_token // empty')
FENCING_HEADERS=()
if echo "$FENCING_TOKEN" | grep -Eq '^[1-9][0-9]*$'; then
  FENCING_HEADERS=(-H "x-focusa-fencing-token: ${FENCING_TOKEN}")
elif ! echo "$ENABLE_RESP" | jq -e '.ok == true and .writer_id == "spec79-pi-driver"' >/dev/null 2>&1; then
  log_fail "Pi RPC work-loop writer enable rejected: $ENABLE_RESP"
fi
START_PAYLOAD=$(jq -n \
  --arg cwd "${ROOT_DIR}" \
  --arg idempotency_key "spec79-pi-driver-work-loop-pi-driver-test" \
  '{cwd: $cwd, idempotency_key: $idempotency_key}')
START=$(http_json -X POST "${BASE_URL}/v1/work-loop/driver/start" -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" "${FENCING_HEADERS[@]}" \
  -H 'x-focusa-approval: approved' -d "${START_PAYLOAD}")
if echo "$START" | jq -e '.status == "accepted" and .adapter == "pi-rpc"' >/dev/null 2>&1; then
  log_pass "Pi RPC driver accepted the compatible runtime"
  sleep 1
  STATUS=$(http_json "${BASE_URL}/v1/work-loop")
  if echo "$STATUS" | jq -e '.transport.daemon_supervised_session.adapter == "pi-rpc"' >/dev/null 2>&1; then
    log_pass "Daemon-supervised Pi session visible in work-loop status"
  else
    log_fail "Daemon-supervised Pi session not visible: $STATUS"
  fi

  STOP=$(http_json -X POST "${BASE_URL}/v1/work-loop/driver/stop" -H 'Content-Type: application/json' \
    -H "x-focusa-writer-id: ${WRITER_ID}" "${FENCING_HEADERS[@]}" -H 'x-focusa-approval: approved')
  if echo "$STOP" | jq -e '.status == "accepted" or .status == "stopped" or .tool_result.status == "accepted"' >/dev/null 2>&1; then
    log_pass "Pi RPC driver stop accepted"
  else
    log_fail "Pi RPC driver stop not accepted: $STOP"
  fi
else
  log_fail "Pi RPC driver start not accepted: $START"
fi

LOOP_STOP=$(http_json -X POST "${BASE_URL}/v1/work-loop/stop" -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${WRITER_ID}" "${FENCING_HEADERS[@]}" \
  -H 'x-focusa-approval: approved' -d '{}')
if echo "$LOOP_STOP" | jq -e '.status == "accepted" or .state == "stopped" or .ok == true' >/dev/null 2>&1; then
  log_pass "Pi RPC work-loop writer stopped cleanly"
else
  log_fail "Pi RPC work-loop writer stop rejected: $LOOP_STOP"
fi

echo "=== PI RPC DRIVER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

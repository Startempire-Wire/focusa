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
    -H "x-scope-continuity-id: work-loop-continuation-test" \
    "$@"
}
http_json(){ curl -sS "$@"; }
WORK_LOOP_ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
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

if rg -n 'if kind == "agent_end"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && ! rg -n 'if kind == "turn_end" \|\| kind == "agent_end"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Pi outcome governance waits for the complete agent tool loop"
else
  log_fail "Pi outcome governance can stop at an intermediate tool turn"
fi

if rg -n '"--no-extensions"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"--no-skills"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"--no-prompt-templates"' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'FOCUSA_PI_VITAL_INFO_PROMPT_MODE' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Supervised Pi is orchestration-isolated and non-interactive"
else
  log_fail "Supervised Pi can load competing orchestration or block on UI"
fi

if rg -n 'process_group\(0\)' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'kill_on_drop\(true\)' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'terminate_pi_rpc_child' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n 'args\(\["-TERM", &pgid\]\)' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Pi RPC driver uses process-group cleanup to prevent orphaned children"
else
  log_fail "Pi RPC driver process-group cleanup guard missing"
fi
http_json -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"${ROOT_DIR}\",\"continuity_id\":\"work-loop-continuation-test\",\"mission\":\"verify Pi RPC driver contract\",\"current_action\":\"spec79_pi_rpc_driver\",\"next_slice\":\"verify scoped driver lifecycle\",\"canonical\":true}" >/dev/null
WRITER_ID=$(http_json "${BASE_URL}/v1/work-loop" | jq -r '.active_writer // "spec79-pi-driver"')
START_PAYLOAD=$(jq -n \
  --arg cwd "${ROOT_DIR}" \
  --arg idempotency_key "spec79-pi-driver-work-loop-continuation-test" \
  '{cwd: $cwd, idempotency_key: $idempotency_key}')
START=$(http_json -X POST "${BASE_URL}/v1/work-loop/driver/start" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${WRITER_ID}" -d "${START_PAYLOAD}")
DRIVER_UNAVAILABLE=0
if echo "$START" | jq -e '(.status == "accepted" and .adapter == "pi-rpc") or ((.error // "") | test("already active"))' >/dev/null 2>&1; then
  log_pass "Pi RPC driver start accepted or already active"
elif echo "$START" | jq -e '(.error // "") | test("failed to spawn pi rpc")' >/dev/null 2>&1; then
  DRIVER_UNAVAILABLE=1
  log_pass "Pi RPC driver route reports unavailable runtime dependency explicitly"
else
  log_fail "Pi RPC driver start not accepted: $START"
fi

if [ "$DRIVER_UNAVAILABLE" -eq 0 ]; then
  sleep 1
  STATUS=$(http_json "${BASE_URL}/v1/work-loop")
  if echo "$STATUS" | jq -e '.transport.daemon_supervised_session.adapter == "pi-rpc"' >/dev/null 2>&1; then
    log_pass "Daemon-supervised Pi session visible in work-loop status"
  else
    log_fail "Daemon-supervised Pi session not visible: $STATUS"
  fi

  STOP=$(http_json -X POST "${BASE_URL}/v1/work-loop/driver/stop" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${WRITER_ID}")
  if echo "$STOP" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
    log_pass "Pi RPC driver stop accepted"
  else
    log_fail "Pi RPC driver stop not accepted: $STOP"
  fi
fi

echo "=== PI RPC DRIVER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

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
http_json(){ curl -sS "$@"; }
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP_ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
if rg -n '/v1/work-loop/driver/start|/v1/work-loop/driver/prompt|/v1/work-loop/driver/abort|/v1/work-loop/driver/stop' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "Pi RPC driver routes are registered"
else
  log_fail "Pi RPC driver routes missing"
fi
WRITER_ID=$(http_json "${BASE_URL}/v1/work-loop" | jq -r '.active_writer // "spec79-pi-driver"')
START=$(http_json -X POST "${BASE_URL}/v1/work-loop/driver/start" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${WRITER_ID}" -d '{"cwd":"/home/wirebot/focusa"}')
if echo "$START" | jq -e '(.status == "accepted" and .adapter == "pi-rpc") or ((.error // "") | test("already active"))' >/dev/null 2>&1; then
  log_pass "Pi RPC driver start accepted or already active"
else
  log_fail "Pi RPC driver start not accepted: $START"
fi
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

echo "=== PI RPC DRIVER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

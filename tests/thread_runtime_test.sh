#!/bin/bash
# Thread runtime regression test
# Verifies thread create/list/get consistency and proposal runtime basics.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

THREAD_NAME="audit-thread-$(date +%s%N)"
THREAD_INTENT="verify thread runtime"

log_info "Create thread"
resp=$(curl -sS -X POST "${BASE_URL}/v1/threads" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"${THREAD_NAME}\",\"primary_intent\":\"${THREAD_INTENT}\"}")
thread_id=$(echo "$resp" | jq -r '.thread.id // .thread_id // empty')
if [ -n "$thread_id" ] && [ "$thread_id" != "null" ]; then
  log_pass "Thread create returned id ${thread_id}"
else
  log_fail "Thread create did not return id :: $resp"
fi

if [ -n "$thread_id" ] && [ "$thread_id" != "null" ]; then
  log_info "Thread list consistency"
  found=0
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    if curl -sS "${BASE_URL}/v1/threads" | jq -e --arg id "$thread_id" '.threads | any(.id == $id)' >/dev/null 2>&1; then
      found=1
      break
    fi
    sleep 0.1
  done
  if [ "$found" = "1" ]; then
    log_pass "Created thread visible in thread list"
  else
    log_fail "Created thread never appeared in thread list"
  fi

  log_info "Thread get consistency"
  code=$(curl -sS -o /tmp/focusa-thread-runtime-body.json -w "%{http_code}" "${BASE_URL}/v1/threads/${thread_id}")
  if [ "$code" = "200" ] && jq -e --arg id "$thread_id" '.thread.id == $id' /tmp/focusa-thread-runtime-body.json >/dev/null 2>&1; then
    log_pass "Created thread retrievable by id"
  else
    log_fail "Created thread not retrievable by id :: $(cat /tmp/focusa-thread-runtime-body.json 2>/dev/null || true)"
  fi
fi

log_info "Proposal submit/list/resolve basics"
submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d '{"kind":"focus_change","source":"thread-runtime-test","payload":{},"deadline_ms":5000}')
if echo "$submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "Proposal submit accepted"
else
  log_fail "Proposal submit failed :: $submit"
fi

if curl -sS "${BASE_URL}/v1/proposals" | jq -e '.pending >= 1' >/dev/null 2>&1; then
  log_pass "Pending proposal visible"
else
  log_fail "Pending proposal not visible"
fi

resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d '{}')
if echo "$resolve" | jq -e '.status != null' >/dev/null 2>&1; then
  log_pass "Proposal resolution returned structured status"
else
  log_fail "Proposal resolution failed :: $resolve"
fi

echo ""
echo "=== THREAD RUNTIME RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Thread/proposal runtime checks passed${NC}"
  exit 0
else
  echo -e "${RED}Thread/proposal runtime checks failed${NC}"
  exit 1
fi

#!/bin/bash
# SPEC-54/54a: Visible Output Boundary — strict CI gate

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

http_code() {
  curl -sS -o /tmp/focusa-channel-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-channel-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-channel-body.json)"
  fi
}

echo "=== SPEC-54/54a: Visible Output Boundary (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Health + seed internal event"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true and (.version | type == "string")' "Visible health channel available"
else
  log_fail "Health failed"
  exit 1
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"steering","summary":"operator correction"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed steering/internal signal accepted"
else
  log_fail "Seed steering signal failed"
fi

log_info "Channel separation"
headers=$(curl -sSI "${BASE_URL}/v1/events/stream" -H "Accept: text/event-stream" || true)
if echo "$headers" | grep -q '200'; then
  log_pass "Internal SSE stream reachable"
else
  log_fail "Internal SSE stream unreachable"
fi
if echo "$headers" | tr '[:upper:]' '[:lower:]' | grep -q 'content-type: text/event-stream'; then
  log_pass "Internal channel content-type is text/event-stream"
else
  log_fail "Internal channel content-type incorrect :: $headers"
fi

code=$(http_code "${BASE_URL}/v1/events/recent")
if [ "$code" = "200" ]; then
  json_assert '.events | length > 0' "Internal event channel populated"
else
  log_fail "Events recent failed"
fi

code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  if jq -e 'tostring | test("MemoryDecayTick|IntuitionSignal|CandidateSurfaced|Intuition"; "i") | not' /tmp/focusa-channel-body.json >/dev/null 2>&1; then
    log_pass "Visible channel does not echo internal event names"
  else
    log_fail "Visible channel leaked internal event markers :: $(cat /tmp/focusa-channel-body.json)"
  fi
else
  log_fail "Health fetch failed during anti-echo check"
fi

log_info "Internal prominence / budget"
code=$(http_code -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" \
  -d '{"turn_id":"channel-test","raw_user_input":"test","format":"string","budget":500}')
if [ "$code" = "200" ]; then
  json_assert '.context_stats != null and ((.context_stats.total_tokens // .context_stats.estimated_tokens // 0) <= 500)' "Prompt assembly reports bounded context stats"
else
  log_fail "Prompt assemble failed with HTTP ${code}"
fi

log_info "Priority order / steering observability"
code=$(http_code "${BASE_URL}/v1/focus-gate/candidates")
if [ "$code" = "200" ]; then
  json_assert '.candidates != null' "Steering candidates observable"
else
  log_fail "Gate candidates failed"
fi

code=$(http_code "${BASE_URL}/v1/status")
if [ "$code" = "200" ]; then
  json_assert '.active_frame != null' "Operator/active frame context exposed in status"
else
  log_fail "Status failed"
fi

code=$(http_code "${BASE_URL}/v1/focus/stack")
if [ "$code" = "200" ]; then
  json_assert '.stack != null' "Mission context exposed in focus stack"
else
  log_fail "Focus stack failed"
fi

code=$(http_code "${BASE_URL}/v1/memory/semantic")
if [ "$code" = "200" ]; then
  json_assert '.semantic != null' "Working set visible via semantic memory"
else
  log_fail "Semantic memory failed"
fi

code=$(http_code "${BASE_URL}/v1/memory/procedural")
if [ "$code" = "200" ]; then
  json_assert '.procedural != null' "Constraints visible via procedural memory"
else
  log_fail "Procedural memory failed"
fi

echo ""
echo "=== SPEC-54/54a VISIBLE OUTPUT BOUNDARY RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict visible output boundary checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict visible output boundary checks failed${NC}"
  exit 1
fi

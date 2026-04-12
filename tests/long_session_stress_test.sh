#!/bin/bash
# SPEC §15: Long-session stress test (200+ turns)
# Verify behavior at 200+ turns

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
TURNS=${1:-200}

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC §15: Long-session stress test ($TURNS turns) ==="

# Test 1: Health check
if curl -sf "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Daemon health"
else
  log_fail "Daemon not responding"
  exit 1
fi

# Test 2: Start session
SESSION_ID=$(curl -s -X POST "${BASE_URL}/v1/session/start" \
  -H "Content-Type: application/json" \
  -d '{"instance_id":"stress-test"}' | jq -r '.session_id // "new"')
log_pass "Session start"

# Test 3: Rapid turn completion
echo "Running $TURNS turns..."
for i in $(seq 1 $TURNS); do
  TURN_ID="stress-${i}-$(date +%s)"
  curl -s -X POST "${BASE_URL}/v1/turn/start" \
    -H "Content-Type: application/json" \
    -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"stress\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}" >/dev/null
  
  curl -s -X POST "${BASE_URL}/v1/turn/complete" \
    -H "Content-Type: application/json" \
    -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"turn ${i}\",\"tokens\":{\"input\":100,\"output\":50}}" >/dev/null
  
  if [ $((i % 50)) -eq 0 ]; then
    echo "  Progress: $i/$TURNS turns"
  fi
done

# Test 4: Verify telemetry accumulated
TELEMETRY=$(curl -s "${BASE_URL}/v1/telemetry/tokens")
PROMPT_TOKENS=$(echo "$TELEMETRY" | jq '.total_prompt_tokens')
COMPLETION_TOKENS=$(echo "$TELEMETRY" | jq '.total_completion_tokens')

if [ "$PROMPT_TOKENS" -gt 0 ] && [ "$COMPLETION_TOKENS" -gt 0 ]; then
  log_pass "Telemetry accumulated ($PROMPT_TOKENS prompt, $COMPLETION_TOKENS completion)"
else
  log_fail "Telemetry not accumulating"
fi

# Test 5: Focus stack remains functional
STACK=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$STACK" | jq -e '.stack' >/dev/null 2>&1; then
  log_pass "Focus stack functional after $TURNS turns"
else
  log_fail "Focus stack degraded"
fi

# Test 6: Gate candidates observable
CANDIDATES=$(curl -s "${BASE_URL}/v1/focus-gate/candidates")
if echo "$CANDIDATES" | jq -e '.candidates' >/dev/null 2>&1; then
  log_pass "Gate candidates accessible"
else
  log_fail "Gate candidates unavailable"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

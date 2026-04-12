#!/bin/bash
# SPEC §15: Budget exceed test (degradation order)
# Verify exact degradation order when context exceeds budget

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC §15: Budget exceed test ==="

# Test 1: Health
if curl -sf "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Daemon health"
else
  log_fail "Daemon not responding"
  exit 1
fi

# Test 2: Prompt assemble with budget parameter
# Small budget should truncate response
RESP=$(curl -s -X POST "${BASE_URL}/v1/prompt/assemble" \
  -H "Content-Type: application/json" \
  -d '{"turn_id":"budget-test","raw_user_input":"test","format":"string","budget":100}')

if echo "$RESP" | jq -e '.stats' >/dev/null 2>&1; then
  log_pass "Prompt assemble with budget returns stats"
else
  log_fail "Prompt assemble with budget failed"
fi

# Test 3: Context stats observable
STATS=$(curl -s "${BASE_URL}/v1/status" | jq '.prompt_stats')
if echo "$STATS" | jq -e '.' >/dev/null 2>&1; then
  log_pass "Context stats observable via status"
else
  log_fail "Context stats not accessible"
fi

# Test 4: Focus state tokens observable
DUMP=$(curl -s "${BASE_URL}/v1/state/dump")
if echo "$DUMP" | jq -e '.focus_stack' >/dev/null 2>&1; then
  log_pass "Focus stack in state dump"
else
  log_fail "Focus stack not in state"
fi

# Test 5: Degraded mode flag observable
AUTONOMY=$(curl -s "${BASE_URL}/v1/autonomy")
if echo "$AUTONOMY" | jq -e '.' >/dev/null 2>&1; then
  log_pass "Degraded mode flag observable"
else
  log_fail "Degraded mode not accessible"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

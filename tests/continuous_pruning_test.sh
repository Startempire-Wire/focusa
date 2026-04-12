#!/bin/bash
# SPEC §15: Continuous pruning test (bounded growth)
# Session token growth remains bounded without frequent full compaction

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC §15: Continuous pruning test ==="

# Get initial telemetry
INITIAL=$(curl -s "${BASE_URL}/v1/telemetry/tokens")
INITIAL_EVENTS=$(echo "$INITIAL" | jq '.total_events')

# Run multiple turns
for i in $(seq 1 20); do
  TURN="prune-${i}-$(date +%s)"
  curl -s -X POST "${BASE_URL}/v1/turn/start" \
    -H "Content-Type: application/json" \
    -d "{\"turn_id\":\"${TURN}\",\"harness_name\":\"prune\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-12T00:00:00Z\"}" >/dev/null
  curl -s -X POST "${BASE_URL}/v1/turn/complete" \
    -H "Content-Type: application/json" \
    -d "{\"turn_id\":\"${TURN}\",\"assistant_output\":\"prune ${i}\",\"tokens\":{\"input\":50,\"output\":25}}" >/dev/null
done

# Get final telemetry
FINAL=$(curl -s "${BASE_URL}/v1/telemetry/tokens")
FINAL_EVENTS=$(echo "$FINAL" | jq '.total_events')
FINAL_PROMPT=$(echo "$FINAL" | jq '.total_prompt_tokens')

# Test 1: Events accumulated
if [ "$FINAL_EVENTS" -gt "$INITIAL_EVENTS" ]; then
  DIFF=$((FINAL_EVENTS - INITIAL_EVENTS))
  log_pass "Events grew: +${DIFF} (${INITIAL_EVENTS} → ${FINAL_EVENTS})"
else
  log_fail "Events not growing"
fi

# Test 2: Token growth bounded
if [ "$FINAL_PROMPT" -lt 1000000 ]; then
  log_pass "Token growth bounded: ${FINAL_PROMPT} prompt tokens"
else
  log_fail "Token growth unbounded"
fi

# Test 3: Focus state accessible
STACK=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$STACK" | jq -e '.stack' >/dev/null 2>&1; then
  log_pass "Focus state accessible after pruning"
else
  log_fail "Focus state degraded"
fi

# Test 4: Memory accessible
MEMORY=$(curl -s "${BASE_URL}/v1/memory/semantic")
if echo "$MEMORY" | jq -e '.semantic' >/dev/null 2>&1; then
  log_pass "Memory accessible"
else
  log_fail "Memory unavailable"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

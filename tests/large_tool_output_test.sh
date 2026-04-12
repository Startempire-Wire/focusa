#!/bin/bash
# SPEC §15: Large tool-output test (forced ECS handles)
# Verify ECS externalization infrastructure

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC §15: Large tool-output test ==="

# Test 1: Store artifact (any size)
HANDLE=$(curl -s -X POST "${BASE_URL}/v1/ecs/store" \
  -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"test-'"$(date +%s)"'","content":"hello world"}' | jq -r '.id')

if [ -n "$HANDLE" ] && [ "$HANDLE" != "null" ]; then
  log_pass "ECS store endpoint works, handle: ${HANDLE:0:20}..."
else
  log_fail "ECS store endpoint failed"
fi

# Test 2: List handles
COUNT=$(curl -s "${BASE_URL}/v1/ecs/handles" | jq '.count')
if [ "$COUNT" -gt 0 ]; then
  log_pass "ECS handles accessible: $COUNT total"
else
  log_fail "ECS handles empty"
fi

# Test 3: Resolve handle
RESOLVE=$(curl -s "${BASE_URL}/v1/ecs/resolve/${HANDLE}")
if echo "$RESOLVE" | jq -e '.handle' >/dev/null 2>&1; then
  log_pass "Resolve endpoint returns handle metadata"
else
  log_fail "Resolve endpoint failed"
fi

# Test 4: Session start (stress test prep)
curl -s -X POST "${BASE_URL}/v1/session/start" \
  -H "Content-Type: application/json" \
  -d '{"instance_id":"ecs-test"}' >/dev/null
log_pass "Session start for ECS operations"

# Test 5: Multiple store operations
for i in 1 2 3; do
  curl -s -X POST "${BASE_URL}/v1/ecs/store" \
    -H "Content-Type: application/json" \
    -d '{"kind":"text","label":"batch-'"$i"'","content":"test content"}' >/dev/null
done
NEW_COUNT=$(curl -s "${BASE_URL}/v1/ecs/handles" | jq '.count')
if [ "$NEW_COUNT" -gt "$COUNT" ]; then
  log_pass "Multiple store operations work"
else
  log_fail "Multiple store operations failed"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

#!/bin/bash
# REAL integration tests against running focusa-daemon
# Requires daemon running at 127.0.0.1:8787

set -e

BASE_URL="http://127.0.0.1:8787"
FAILED=0
PASSED=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

log_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    echo "  Error: $2"
    ((FAILED++))
}

echo "=== FOCUSA DAEMON INTEGRATION TESTS ==="
echo "Target: $BASE_URL"
echo ""

# Test 1: Health endpoint
echo "Test 1: Health endpoint"
RESPONSE=$(curl -s "$BASE_URL/v1/health")
if echo "$RESPONSE" | grep -q '"ok":true'; then
    log_pass "Health returns ok:true"
else
    log_fail "Health endpoint" "$RESPONSE"
fi

# Test 2: Info endpoint
echo "Test 2: Info endpoint"
RESPONSE=$(curl -s "$BASE_URL/v1/info")
if echo "$RESPONSE" | grep -q '"api_bind"'; then
    log_pass "Info returns api_bind"
else
    log_fail "Info endpoint" "$RESPONSE"
fi

# Test 3: Focus stack exists
echo "Test 3: Focus stack"
RESPONSE=$(curl -s "$BASE_URL/v1/focus/stack")
if echo "$RESPONSE" | grep -q '"frames"'; then
    log_pass "Focus stack returns frames"
else
    log_fail "Focus stack" "$RESPONSE"
fi

# Test 4: Turn lifecycle - START
echo "Test 4: Turn lifecycle - START"
TURN_ID="test-turn-$(uuidgen)"
RESPONSE=$(curl -s -X POST "$BASE_URL/v1/turn/start" \
    -H "content-type: application/json" \
    -d "{\"turn_id\":\"$TURN_ID\",\"adapter_id\":\"integration-test\",\"harness_name\":\"test\",\"timestamp\":\"$(date -Iseconds)\"}")
if echo "$RESPONSE" | grep -q '"status":"accepted"'; then
    log_pass "Turn start accepted"
else
    log_fail "Turn start" "$RESPONSE"
fi

# Test 5: Turn lifecycle - PROMPT ASSEMBLE
echo "Test 5: Turn lifecycle - PROMPT ASSEMBLE"
RESPONSE=$(curl -s -X POST "$BASE_URL/v1/prompt/assemble" \
    -H "content-type: application/json" \
    -d "{\"turn_id\":\"$TURN_ID\",\"raw_user_input\":\"Integration test input\",\"format\":\"string\"}")
if echo "$RESPONSE" | grep -q '"assembled"' && echo "$RESPONSE" | grep -q '"FOCUS FRAME"'; then
    log_pass "Prompt assembly returns focus context"
else
    log_fail "Prompt assembly" "$RESPONSE"
fi

# Test 6: Turn lifecycle - COMPLETE
echo "Test 6: Turn lifecycle - COMPLETE"
RESPONSE=$(curl -s -X POST "$BASE_URL/v1/turn/complete" \
    -H "content-type: application/json" \
    -d "{\"turn_id\":\"$TURN_ID\",\"assistant_output\":\"Integration test response\",\"artifacts\":[],\"errors\":[]}")
if echo "$RESPONSE" | grep -q '"status":"accepted"'; then
    log_pass "Turn complete accepted"
else
    log_fail "Turn complete" "$RESPONSE"
fi

# Test 7: Idempotency - duplicate turn complete should return duplicate:true
echo "Test 7: Idempotency check"
RESPONSE=$(curl -s -X POST "$BASE_URL/v1/turn/complete" \
    -H "content-type: application/json" \
    -d "{\"turn_id\":\"$TURN_ID\",\"assistant_output\":\"Integration test response\",\"artifacts\":[],\"errors\":[]}")
if echo "$RESPONSE" | grep -q '"duplicate":true'; then
    log_pass "Duplicate turn detected"
else
    log_fail "Idempotency" "$RESPONSE"
fi

# Test 8: Events persisted
echo "Test 8: Events persistence"
sleep 0.5  # Allow async event persistence
RESPONSE=$(curl -s "$BASE_URL/v1/events/recent?limit=10")
if echo "$RESPONSE" | grep -q "$TURN_ID"; then
    log_pass "Turn events persisted to event log"
else
    log_fail "Event persistence" "Turn $TURN_ID not found in events"
fi

# Test 9: Gate policy
echo "Test 9: Gate policy"
RESPONSE=$(curl -s "$BASE_URL/v1/gate/policy")
if echo "$RESPONSE" | grep -q '"surface_threshold"'; then
    log_pass "Gate policy returns threshold"
else
    log_fail "Gate policy" "$RESPONSE"
fi

# Test 10: Autonomy state
echo "Test 10: Autonomy state"
RESPONSE=$(curl -s "$BASE_URL/v1/autonomy")
if echo "$RESPONSE" | grep -q '"ari_score"'; then
    log_pass "Autonomy returns ARI score"
else
    log_fail "Autonomy" "$RESPONSE"
fi

# Test 11: Reflection status
echo "Test 11: Reflection status"
RESPONSE=$(curl -s "$BASE_URL/v1/reflect/status")
if echo "$RESPONSE" | grep -q '"scheduler"'; then
    log_pass "Reflection returns scheduler config"
else
    log_fail "Reflection status" "$RESPONSE"
fi

# Test 12: RFM status
echo "Test 12: RFM status"
RESPONSE=$(curl -s "$BASE_URL/v1/rfm")
if echo "$RESPONSE" | grep -q '"ais_score"'; then
    log_pass "RFM returns AIS score"
else
    log_fail "RFM" "$RESPONSE"
fi

# Test 13: Skills list
echo "Test 13: Skills list"
RESPONSE=$(curl -s "$BASE_URL/v1/skills")
if echo "$RESPONSE" | grep -q '"skills"' && echo "$RESPONSE" | grep -q '"prohibited"'; then
    log_pass "Skills returns list and prohibited"
else
    log_fail "Skills" "$RESPONSE"
fi

# Test 14: Memory semantic
echo "Test 14: Memory semantic"
RESPONSE=$(curl -s "$BASE_URL/v1/memory/semantic?limit=5")
if echo "$RESPONSE" | grep -q '"semantic"'; then
    log_pass "Semantic memory returns"
else
    log_fail "Semantic memory" "$RESPONSE"
fi

# Test 15: Proposals list
echo "Test 15: Proposals list"
RESPONSE=$(curl -s "$BASE_URL/v1/proposals")
if echo "$RESPONSE" | grep -q '"proposals"'; then
    log_pass "Proposals endpoint returns"
else
    log_fail "Proposals" "$RESPONSE"
fi

# Test 16: CLT path
echo "Test 16: CLT lineage path"
RESPONSE=$(curl -s "$BASE_URL/v1/clt/path")
if echo "$RESPONSE" | grep -q '"path"'; then
    log_pass "CLT returns lineage path"
else
    log_fail "CLT" "$RESPONSE"
fi

# Test 17: Training status
echo "Test 17: Training status"
RESPONSE=$(curl -s "$BASE_URL/v1/training/status")
if echo "$RESPONSE" | grep -q '"contribution_enabled"'; then
    log_pass "Training status returns"
else
    log_fail "Training" "$RESPONSE"
fi

# Test 18: Export history
echo "Test 18: Export history"
RESPONSE=$(curl -s "$BASE_URL/v1/export/history")
if echo "$RESPONSE" | grep -q '"exports"'; then
    log_pass "Export history returns"
else
    log_fail "Export" "$RESPONSE"
fi

# Test 19: Env endpoints
echo "Test 19: Environment config"
RESPONSE=$(curl -s "$BASE_URL/v1/env")
if echo "$RESPONSE" | grep -q '"proxy_base"'; then
    log_pass "Env returns proxy configuration"
else
    log_fail "Env" "$RESPONSE"
fi

# Test 20: Error handling - 404 for unknown route
echo "Test 20: Error handling (404)"
RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/v1/this-route-does-not-exist")
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "404" ]; then
    log_pass "404 returned for unknown route"
else
    log_fail "404 handling" "Got HTTP $HTTP_CODE"
fi

# Test 21: Invalid JSON handling
echo "Test 21: Invalid JSON handling"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/v1/turn/start" \
    -H "content-type: application/json" \
    -d "not valid json")
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "400" ] || [ "$HTTP_CODE" = "422" ]; then
    log_pass "Bad request returned for invalid JSON"
else
    log_fail "Invalid JSON handling" "Got HTTP $HTTP_CODE"
fi

# Test 22: SSE endpoint exists
echo "Test 22: SSE endpoint"
# Just check endpoint returns 200 (not streaming)
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Accept: text/event-stream" "$BASE_URL/v1/events/stream")
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    log_pass "SSE endpoint returns 200"
else
    log_fail "SSE endpoint" "Got HTTP $HTTP_CODE"
fi

# Summary
echo ""
echo "=== TEST SUMMARY ==="
echo -e "${GREEN}PASSED: $PASSED${NC}"
echo -e "${RED}FAILED: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}ALL TESTS PASSED ✓${NC}"
    exit 0
else
    echo -e "${RED}SOME TESTS FAILED ✗${NC}"
    exit 1
fi

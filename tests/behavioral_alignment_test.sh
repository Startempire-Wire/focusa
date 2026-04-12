#!/bin/bash
# SPEC 53: Behavioral alignment test
# Verify Focusa state materially shapes behavior

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC 53: Behavioral alignment test ==="

# Test 1: Focus stack accessible (constraint/decision consultation)
echo "1. Focus stack (constraints + decisions):"
FOCUS=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$FOCUS" | jq -e '.stack' >/dev/null 2>&1; then
    log_pass "Focus stack accessible"
else
    log_fail "Focus stack not accessible"
fi

# Test 2: Focus state update (decision distillation)
echo "2. Focus state update (decision distillation):"
UPDATE=$(curl -s -X POST "${BASE_URL}/v1/focus/update" \
    -H "Content-Type: application/json" \
    -d '{"frame_id":"019d7f34-8cc2-7603-a2ea-46d283ad198e","turn_id":"test-turn","delta":{"decisions":["test decision from SPEC 53"]}}')
if echo "$UPDATE" | grep -qE '"status"|"accepted"'; then
    log_pass "Decision distillation works"
else
    log_fail "Decision distillation failed"
fi

# Test 3: Focus push (working set)
echo "3. Focus push (working set creation):"
PUSH=$(curl -s -X POST "${BASE_URL}/v1/focus/push" \
    -H "Content-Type: application/json" \
    -d '{"title":"SPEC 53 test frame","source":"test"}')
if echo "$PUSH" | grep -qE '"status"'; then
    log_pass "Working set creation works"
else
    log_fail "Working set creation failed"
fi

# Test 4: State dump (full consultation)
echo "4. State dump (constraint consultation):"
DUMP=$(curl -s "${BASE_URL}/v1/state/dump")
if echo "$DUMP" | jq -e '.focus_stack' >/dev/null 2>&1; then
    log_pass "State dump accessible"
else
    log_fail "State dump not accessible"
fi

# Test 5: Trace steering detection
echo "5. Steering detection trace:"
STEERING=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d '{"event_type":"steering_detected","turn_id":"steering-test","steering_detected":true}')
if echo "$STEERING" | grep -q '"status":"recorded"'; then
    log_pass "Steering detection traceable"
else
    log_fail "Steering detection not traceable"
fi

# Test 6: Trace subject hijack prevention
echo "6. Subject hijack prevention trace:"
HIJACK=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d '{"event_type":"subject_hijack_prevented","turn_id":"hijack-test","subject_hijack_prevented":true}')
if echo "$HIJACK" | grep -q '"status":"recorded"'; then
    log_pass "Subject hijack prevention traceable"
else
    log_fail "Subject hijack prevention not traceable"
fi

# Test 7: Blocker via focus update
echo "7. Blocker emission via focus update:"
BLOCKER=$(curl -s -X POST "${BASE_URL}/v1/focus/update" \
    -H "Content-Type: application/json" \
    -d '{"frame_id":"019d7f34-8cc2-7603-a2ea-46d283ad198e","turn_id":"test-turn","delta":{"failures":["test blocker"]}}')
if echo "$BLOCKER" | grep -qE '"status"|"accepted"'; then
    log_pass "Blocker emission works"
else
    log_fail "Blocker emission failed"
fi

# Test 8: Trace stats
echo "8. Trace stats:"
STATS=$(curl -s "${BASE_URL}/v1/telemetry/trace/stats")
if echo "$STATS" | jq -e '.by_event_type' >/dev/null 2>&1; then
    log_pass "Trace stats accessible"
else
    log_fail "Trace stats not accessible"
fi

# Test 9: Operator subject tracking
echo "9. Operator subject tracking:"
SUBJECT=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d '{"event_type":"operator_subject","turn_id":"subject-test","operator_subject":"test request"}')
if echo "$SUBJECT" | grep -q '"status":"recorded"'; then
    log_pass "Operator subject trackable"
else
    log_fail "Operator subject not trackable"
fi

# Test 10: Focus slice size
echo "10. Focus slice size tracking:"
SLICE=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d '{"event_type":"focus_slice_size","turn_id":"slice-test","focus_slice_size":500}')
if echo "$SLICE" | grep -q '"status":"recorded"'; then
    log_pass "Focus slice size trackable"
else
    log_fail "Focus slice size not trackable"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

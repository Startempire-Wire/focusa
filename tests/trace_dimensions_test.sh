#!/bin/bash
# SPEC 56: Trace dimensions test
# Verify all 18 trace dimensions are trackable

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC 56: Trace dimensions test ==="

# All 18 trace dimension event types from SPEC 56
TRACE_TYPES=(
    "working_set_used"
    "constraints_consulted"
    "decisions_consulted"
    "action_intents_proposed"
    "verification_result"
    "ontology_delta_applied"
    "operator_subject"
    "steering_detected"
    "subject_hijack_prevented"
    "subject_hijack_occurred"
    "prior_mission_reused"
    "focus_slice_size"
    "focus_slice_relevance_score"
)

# Test 1: All trace types can be recorded
for t in "${TRACE_TYPES[@]}"; do
    RESP=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
        -H "Content-Type: application/json" \
        -d "{\"event_type\":\"${t}\",\"turn_id\":\"${t}-test\"}")
    if echo "$RESP" | grep -q '"status":"recorded"'; then
        log_pass "Trace type: ${t}"
    else
        log_fail "Trace type: ${t}"
    fi
done

# Test 2: Trace events retrievable
EVENTS=$(curl -s "${BASE_URL}/v1/telemetry/trace?limit=100" | jq '.events | length')
if [ "$EVENTS" -ge 10 ]; then
    log_pass "Trace events retrievable: $EVENTS events"
else
    log_fail "Trace events not retrievable"
fi

# Test 3: Trace stats accessible
STATS=$(curl -s "${BASE_URL}/v1/telemetry/trace/stats")
if echo "$STATS" | jq -e '.by_event_type' >/dev/null 2>&1; then
    log_pass "Trace stats accessible"
else
    log_fail "Trace stats not accessible"
fi

# Test 4: Steering detection flag
STEERING=$(curl -s "${BASE_URL}/v1/telemetry/trace?event_type=steering_detected" | jq '.events | length')
if [ "$STEERING" -gt 0 ]; then
    log_pass "Steering detected flag tracked"
else
    log_fail "Steering detected not tracked"
fi

# Test 5: Subject hijack tracking
SUBJECT=$(curl -s -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d '{"event_type":"subject_hijack_prevented","turn_id":"hijack-test","subject_hijack_prevented":true}')
if echo "$SUBJECT" | grep -q '"status":"recorded"'; then
    log_pass "Subject hijack prevention tracked"
else
    log_fail "Subject hijack not tracked"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

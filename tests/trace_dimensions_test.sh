#!/bin/bash
# SPEC 56: Trace dimensions test
# Verify all 18 trace dimensions are trackable and retrievable

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC 56: Trace dimensions test ==="

# All 18 trace dimension event types from SPEC 56
TRACE_TYPES=(
    "mission_frame_context"
    "working_set_used"
    "constraints_consulted"
    "decisions_consulted"
    "action_intents_proposed"
    "tools_invoked"
    "verification_result"
    "ontology_delta_applied"
    "blockers_failures_emitted"
    "final_state_transition"
    "operator_subject"
    "active_subject_after_routing"
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

# Test 2: Trace stats/accessibility convergence
STATS='{}'
TRACE_TOTAL=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
    STATS=$(curl -s "${BASE_URL}/v1/telemetry/trace/stats")
    TRACE_TOTAL=$(echo "$STATS" | jq '(.by_event_type // {}) | to_entries | map(.value) | add // 0')
    if [ "$TRACE_TOTAL" -ge 10 ]; then
        break
    fi
    sleep 0.2
done
if echo "$STATS" | jq -e '.by_event_type' >/dev/null 2>&1; then
    log_pass "Trace stats accessible"
else
    log_fail "Trace stats not accessible"
fi

# Test 3: Trace events retrievable after stats converge
EVENTS=$(curl -s "${BASE_URL}/v1/telemetry/trace?limit=100" | jq '.events | length')
if [ "$EVENTS" -ge 18 ]; then
    log_pass "Trace events retrievable: $EVENTS events"
else
    log_fail "Trace events not retrievable"
fi

# Test 4: event_type filter works for steering_detected
STEERING=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
    STEERING=$(curl -s "${BASE_URL}/v1/telemetry/trace?event_type=steering_detected" | jq '.events | length')
    if [ "$STEERING" -gt 0 ]; then
        break
    fi
    sleep 0.2
done
if [ "$STEERING" -gt 0 ]; then
    log_pass "Steering detected filter tracked"
else
    log_fail "Steering detected filter not tracked"
fi

# Test 5: tool-usage route emits tools_invoked trace event
TOOL_USAGE=$(curl -s -X POST "${BASE_URL}/v1/telemetry/tool-usage" \
    -H "Content-Type: application/json" \
    -d '{"turn_id":"tool-trace-test","tools":["read","bash"]}')
if echo "$TOOL_USAGE" | jq -e '.status == "accepted" and .recorded == 2' >/dev/null 2>&1; then
    log_pass "Tool usage batch accepted"
else
    log_fail "Tool usage batch rejected"
fi
TOOLS_INVOKED=$(curl -s "${BASE_URL}/v1/telemetry/trace?event_type=tools_invoked" | jq '.events | length')
if [ "$TOOLS_INVOKED" -gt 0 ]; then
    log_pass "tools_invoked trace emitted"
else
    log_fail "tools_invoked trace missing"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

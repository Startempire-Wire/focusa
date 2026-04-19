#!/bin/bash
# Runtime contract: scope-failure taxonomy/events must be recordable and queryable via telemetry surfaces.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

TURN_ID="scope-taxonomy-$(date +%s%N)"

record_trace() {
  local kind="$1"
  curl -sS -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d "{\"event_type\":\"scope_failure_recorded\",\"turn_id\":\"${TURN_ID}\",\"failure_kind\":\"${kind}\",\"reason\":\"runtime contract check\"}" >/tmp/scope-trace-post.json
}

record_trace "scope_contamination"
record_trace "adjacent_thread_leakage"
record_trace "answer_broadening"

TRACE_JSON="$(curl -sS "${BASE_URL}/v1/telemetry/trace?limit=200")"

if echo "$TRACE_JSON" | jq -e --arg tid "$TURN_ID" '.events | any(.event_type=="scope_failure_recorded" and ((.payload.turn_id // .turn_id // "") == $tid) and ((.payload.failure_kind // "") == "scope_contamination"))' >/dev/null 2>&1; then
  log_pass "scope_contamination failure event is retrievable from telemetry trace"
else
  log_fail "scope_contamination failure event missing from telemetry trace"
fi

if echo "$TRACE_JSON" | jq -e --arg tid "$TURN_ID" '.events | any(.event_type=="scope_failure_recorded" and ((.payload.turn_id // .turn_id // "") == $tid) and ((.payload.failure_kind // "") == "adjacent_thread_leakage"))' >/dev/null 2>&1; then
  log_pass "adjacent_thread_leakage failure event is retrievable from telemetry trace"
else
  log_fail "adjacent_thread_leakage failure event missing from telemetry trace"
fi

if echo "$TRACE_JSON" | jq -e --arg tid "$TURN_ID" '.events | any(.event_type=="scope_failure_recorded" and ((.payload.turn_id // .turn_id // "") == $tid) and ((.payload.failure_kind // "") == "answer_broadening"))' >/dev/null 2>&1; then
  log_pass "answer_broadening failure event is retrievable from telemetry trace"
else
  log_fail "answer_broadening failure event missing from telemetry trace"
fi

STATS_JSON="$(curl -sS "${BASE_URL}/v1/telemetry/trace/stats")"
if echo "$STATS_JSON" | jq -e '.by_event_type | has("scope_failure_recorded")' >/dev/null 2>&1; then
  log_pass "trace stats include scope_failure_recorded aggregate"
else
  log_fail "trace stats missing scope_failure_recorded aggregate"
fi

echo "=== Scope-failure taxonomy/events contract ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

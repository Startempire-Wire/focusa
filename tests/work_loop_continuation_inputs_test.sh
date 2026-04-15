#!/bin/bash
# SPEC-79 §11 guardrail: continuation inputs must be canonicalized in daemon work-loop state.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
http_json() { curl -sS "$@"; }

ACTIVE_WRITER=$(http_json "${BASE_URL}/v1/work-loop" | jq -r '.active_writer // "spec79-context-test"')
CTX_RESP=$(http_json -X POST "${BASE_URL}/v1/work-loop/context" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${ACTIVE_WRITER}" \
  -d '{"current_ask":"Why did the loop pause?","ask_kind":"question","scope_kind":"fresh_question","carryover_policy":"suppress_by_default","excluded_context_reason":"fresh_scope","excluded_context_labels":["MISSION","OLD_WORKING_SET"],"source_turn_id":"spec79-turn-ctx","operator_steering_detected":true}')
if echo "$CTX_RESP" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "work-loop context update accepted"
else
  log_fail "work-loop context update rejected: $CTX_RESP"
fi

STATUS=$(http_json "${BASE_URL}/v1/work-loop")
if echo "$STATUS" | jq -e '.decision_context.current_ask == "Why did the loop pause?" and .decision_context.ask_kind == "question" and .decision_context.scope_kind == "fresh_question" and .decision_context.carryover_policy == "suppress_by_default" and .decision_context.excluded_context_reason == "fresh_scope" and (.decision_context.excluded_context_labels | index("MISSION")) != null' >/dev/null 2>&1; then
  log_pass "decision context is canonicalized in work-loop status"
else
  log_fail "decision context missing from work-loop status: $STATUS"
fi

if echo "$STATUS" | jq -e '.continuation_inputs.current_ask == "Why did the loop pause?" and .continuation_inputs.ask_kind == "question" and .continuation_inputs.scope_kind == "fresh_question" and .continuation_inputs.pending_proposals_requiring_resolution != null and .continuation_inputs.autonomy_level != null and .continuation_inputs.next_work_risk_class != null' >/dev/null 2>&1; then
  log_pass "spec79 continuation inputs exposed in status"
else
  log_fail "spec79 continuation inputs incomplete in status: $STATUS"
fi

if echo "$STATUS" | jq -e '.last_continue_reason == "operator steering detected"' >/dev/null 2>&1; then
  log_pass "operator steering is consumed by daemon continuation state"
else
  log_fail "operator steering did not reach daemon continuation state: $STATUS"
fi

echo "=== WORK-LOOP CONTINUATION INPUT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  echo -e "${RED}Work-loop continuation input test failed${NC}"
  exit 1
fi

echo -e "${GREEN}Work-loop continuation inputs verified${NC}"

#!/bin/bash
# Runtime contract: query-scope/reset semantics persist from context input to status projection.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

WRITER_ID="$(curl -sS "${BASE_URL}/v1/work-loop/status" | jq -r '.active_writer')"
SOURCE_TURN_ID="pi-turn-$(date +%s%N)"
ASK_TEXT="scope-boundary-runtime-check"

curl -sS -X POST "${BASE_URL}/v1/work-loop/context" \
  -H "Content-Type: application/json" \
  -H "x-focusa-writer-id: ${WRITER_ID}" \
  -d "{\"current_ask\":\"${ASK_TEXT}\",\"ask_kind\":\"question\",\"scope_kind\":\"fresh_question\",\"carryover_policy\":\"suppress_by_default\",\"excluded_context_reason\":\"correction_reset\",\"excluded_context_labels\":[\"legacy\",\"unrelated\"],\"source_turn_id\":\"${SOURCE_TURN_ID}\"}" >/dev/null

STATUS_JSON=""
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  STATUS_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/status")"
  if echo "$STATUS_JSON" | jq -e '.decision_context.current_ask == $ask and .decision_context.source_turn_id == $turn' --arg ask "$ASK_TEXT" --arg turn "$SOURCE_TURN_ID" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

if echo "$STATUS_JSON" | jq -e '.decision_context.current_ask == $ask and .decision_context.ask_kind == "question" and .decision_context.scope_kind == "fresh_question" and .decision_context.carryover_policy == "suppress_by_default"' --arg ask "$ASK_TEXT" >/dev/null 2>&1; then
  log_pass "status projects question/fresh scope boundary semantics"
else
  log_fail "status missing projected question/fresh scope boundary semantics"
fi

if echo "$STATUS_JSON" | jq -e '.decision_context.source_turn_id == $turn' --arg turn "$SOURCE_TURN_ID" >/dev/null 2>&1; then
  log_pass "status preserves source_turn_id from context input"
else
  log_fail "status did not preserve source_turn_id"
fi

if echo "$STATUS_JSON" | jq -e '.decision_context.excluded_context_reason == "correction_reset" and (.decision_context.excluded_context_labels | index("legacy") and index("unrelated"))' >/dev/null 2>&1; then
  log_pass "status preserves reset reason and excluded context labels"
else
  log_fail "status missing reset reason or excluded context labels"
fi

echo "=== QueryScope boundary contract results ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

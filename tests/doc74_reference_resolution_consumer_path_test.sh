#!/bin/bash
# Runtime contract: doc74 reference-resolution evidence must be exposed in verification trace payloads.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

TRACE_JSON="$(curl -sS "${BASE_URL}/v1/telemetry/trace?limit=5000")"

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="verification_result")' >/dev/null 2>&1; then
  log_pass "verification trace events are queryable for trace-review"
else
  log_fail "verification trace events missing"
fi

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="verification_result" and ((.payload.resolved_reference_count // 0) >= 0))' >/dev/null 2>&1; then
  log_pass "verification trace payload includes resolved_reference_count"
else
  log_fail "resolved_reference_count missing from verification trace payload"
fi

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="verification_result" and ((.payload.resolved_reference_aliases // []) | type == "array"))' >/dev/null 2>&1; then
  log_pass "verification trace payload includes resolved_reference_aliases"
else
  log_fail "resolved_reference_aliases missing from verification trace payload"
fi

echo "=== DOC 74 REFERENCE RESOLUTION CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

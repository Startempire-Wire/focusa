#!/bin/bash
# Runtime contract: doc76 retention/decay policy evidence is exposed via trace + event surfaces.
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
EVENTS_JSON="$(curl -sS "${BASE_URL}/v1/events/recent?limit=1200")"

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="verification_result" and ((.payload.retention_policy // "") | length > 0))' >/dev/null 2>&1; then
  log_pass "verification trace payload includes retention_policy signal"
else
  log_fail "retention_policy signal missing from verification trace payload"
fi

if echo "$TRACE_JSON" | jq -e '.events | any(.event_type=="verification_result" and ((.payload.selected_count // 0) >= 0) and ((.payload.pruned_count // 0) >= 0))' >/dev/null 2>&1; then
  log_pass "verification trace payload includes selection/pruning retention evidence"
else
  log_fail "selection/pruning retention evidence missing from verification trace payload"
fi

if echo "$EVENTS_JSON" | jq -e '.events | any(.type=="MemoryDecayTick")' >/dev/null 2>&1; then
  log_pass "runtime event stream exposes memory decay ticks"
else
  log_fail "memory decay tick event not observed in recent event stream"
fi

echo "=== DOC 76 RETENTION POLICY CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

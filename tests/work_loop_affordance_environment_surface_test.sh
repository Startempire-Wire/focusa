#!/bin/bash
# Runtime contract: work-loop status exposes execution-environment facts + first affordance object.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

STATUS_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/status")"

if echo "$STATUS_JSON" | jq -e 'has("execution_environment") and (.execution_environment | type == "object")' >/dev/null 2>&1; then
  log_pass "work-loop status includes execution_environment surface"
else
  log_fail "work-loop status missing execution_environment surface"
fi

if echo "$STATUS_JSON" | jq -e '.execution_environment.facts | any(.fact_key=="git_available") and any(.fact_key=="in_worktree") and any(.fact_key=="transport_session_attached")' >/dev/null 2>&1; then
  log_pass "execution-environment facts include git/worktree/transport prerequisites"
else
  log_fail "execution-environment facts missing expected prerequisite keys"
fi

if echo "$STATUS_JSON" | jq -e '.execution_environment.affordances | any(.id=="affordance_safe_local_code_edit" and .affordance_kind=="safe_local_edit_available")' >/dev/null 2>&1; then
  log_pass "first affordance object is explicitly modeled"
else
  log_fail "first affordance object missing from execution_environment.affordances"
fi

echo "=== WORK-LOOP AFFORDANCE + EXECUTION ENVIRONMENT SURFACE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

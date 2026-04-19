#!/bin/bash
# Runtime contract: work-loop status must expose commitment lifecycle semantics.
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

if echo "$STATUS_JSON" | jq -e 'has("commitment_lifecycle")' >/dev/null 2>&1; then
  log_pass "work-loop status exposes commitment_lifecycle payload"
else
  log_fail "work-loop status missing commitment_lifecycle payload"
fi

if echo "$STATUS_JSON" | jq -e '.commitment_lifecycle | has("creation_semantics") and has("persistence_semantics") and has("decay_semantics") and has("release_semantics")' >/dev/null 2>&1; then
  log_pass "creation/persistence/decay/release semantics are explicitly modeled"
else
  log_fail "one or more commitment lifecycle semantic sections missing"
fi

if echo "$STATUS_JSON" | jq -e '.commitment_lifecycle.decay_semantics.decay_triggers | has("low_productivity_turns") and has("same_subproblem_retries") and has("task_class_failures")' >/dev/null 2>&1; then
  log_pass "decay semantics are grounded in runtime pressure counters"
else
  log_fail "decay semantics missing runtime pressure counters"
fi

echo "=== WORK-LOOP COMMITMENT LIFECYCLE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

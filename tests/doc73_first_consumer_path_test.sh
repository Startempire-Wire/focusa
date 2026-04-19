#!/bin/bash
# Runtime contract: doc73 commitment lifecycle release semantics drive continuity-visible status.
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

if echo "$STATUS_JSON" | jq -e 'has("commitment_lifecycle") and (.commitment_lifecycle | has("release_semantics"))' >/dev/null 2>&1; then
  log_pass "work-loop status exposes commitment release semantics surface"
else
  log_fail "commitment release semantics surface missing from status"
fi

if echo "$STATUS_JSON" | jq -e '.commitment_lifecycle.release_semantics | has("state") and has("release_conditions") and ((.release_conditions|length) > 0)' >/dev/null 2>&1; then
  log_pass "release semantics include state + explicit release conditions"
else
  log_fail "release semantics missing state or release conditions"
fi

if echo "$STATUS_JSON" | jq -e '.commitment_lifecycle.release_semantics.state | IN("released_on_completion","released_on_blocker","released_or_unbound","active","held","bound")' >/dev/null 2>&1; then
  log_pass "release state is in continuity-gating vocabulary"
else
  log_fail "release state not in expected continuity-gating vocabulary"
fi

echo "=== DOC 73 FIRST CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

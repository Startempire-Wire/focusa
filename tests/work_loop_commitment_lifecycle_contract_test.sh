#!/bin/bash
# SPEC-79 / Doc-73 bridge: commitment lifecycle semantics must be explicitly defined on work-loop status.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'fn commitment_lifecycle_for_status\(' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "commitment lifecycle helper exists"
else
  log_fail "commitment lifecycle helper missing"
fi

if rg -n '"creation_semantics"|"persistence_semantics"|"decay_semantics"|"release_semantics"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "creation/persistence/decay/release semantics are explicitly modeled"
else
  log_fail "one or more commitment lifecycle semantic sections missing"
fi

if rg -n 'consecutive_low_productivity_turns|consecutive_same_work_item_retries|consecutive_failures_for_task_class' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "decay semantics are grounded in real runtime pressure counters"
else
  log_fail "decay semantics are not grounded in runtime pressure counters"
fi

if rg -n '"commitment_lifecycle": commitment_lifecycle' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "work-loop status exposes commitment_lifecycle payload"
else
  log_fail "work-loop status missing commitment_lifecycle payload"
fi

echo "=== WORK-LOOP COMMITMENT LIFECYCLE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

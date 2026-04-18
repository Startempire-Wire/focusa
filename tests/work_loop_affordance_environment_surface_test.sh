#!/bin/bash
# SPEC-79 / Doc-66 bridge: status should expose explicit execution-environment facts and first affordance object.
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

if rg -n 'fn execution_environment_for_status\(' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "status route defines execution-environment helper"
else
  log_fail "execution-environment helper missing"
fi

if rg -n '"execution_environment": execution_environment' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "work-loop status payload includes execution_environment surface"
else
  log_fail "work-loop status payload missing execution_environment surface"
fi

if rg -n '"id": "affordance_safe_local_code_edit"|"affordance_kind": "safe_local_edit_available"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "first affordance object is explicitly modeled"
else
  log_fail "first affordance object missing from status surface"
fi

if rg -n '"fact_key": "git_available"|"fact_key": "in_worktree"|"fact_key": "transport_session_attached"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "execution-environment facts include git/worktree/transport prerequisites"
else
  log_fail "execution-environment fact set incomplete"
fi

echo "=== WORK-LOOP AFFORDANCE + EXECUTION ENVIRONMENT SURFACE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

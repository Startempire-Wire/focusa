#!/bin/bash
# SPEC-79 / Doc-66 consumer path: affordance object must be consumed by continuation dispatch decisions.
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

if rg -n 'maybe_dispatch_continuous_turn_prompt' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'execution_environment_for_status\(transport_session_state\.as_deref\(\), &worktree\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "dispatch path consumes execution_environment projection"
else
  log_fail "dispatch path does not consume execution_environment projection"
fi

if rg -n 'affordance_safe_local_code_edit|safe_local_edit_available' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "consumer path reads first affordance object identity"
else
  log_fail "consumer path missing first affordance object lookup"
fi

if rg -n 'TaskClass::Code \| TaskClass::Refactor \| TaskClass::Integration \| TaskClass::Architecture' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "affordance gate is scoped to code-execution-sensitive task classes"
else
  log_fail "affordance gate task-class scoping missing"
fi

if rg -n 'Action::PauseContinuousWork' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'execution affordance blocked before dispatch' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "blocked affordance produces explicit continuation control action"
else
  log_fail "blocked affordance does not influence continuation action selection"
fi

echo "=== WORK-LOOP AFFORDANCE CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

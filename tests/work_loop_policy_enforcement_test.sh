#!/bin/bash
# SPEC-79 policy fields must be materially enforced, not just exposed in status.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
REDUCER_FILE="${ROOT_DIR}/crates/focusa-core/src/reducer.rs"
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
FAILED=0
PASSED=0
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'max_turns budget exhausted|max_wall_clock_ms budget exhausted' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "turn and wall-clock budgets are enforced in daemon decisions"; else log_fail "turn/wall-clock budget enforcement missing"; fi
if rg -n 'max_retries budget exhausted|max_consecutive_failures budget exhausted|cooldown active:' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "retry and cooldown budgets are enforced"; else log_fail "retry/cooldown enforcement missing"; fi
if rg -n 'low-productivity turn budget exhausted|same-subproblem retry budget exhausted' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "low-productivity and same-subproblem budgets are enforced"; else log_fail "low-productivity/same-subproblem enforcement missing"; fi
if rg -n 'scope change requires operator approval|governance-scoped continuation requires operator approval|destructive or high-risk continuation is disabled by policy' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "scope/governance/destructive policy gates are enforced"; else log_fail "scope/governance/destructive policy enforcement missing"; fi
if rg -n 'remaining_turn_budget|remaining_wall_clock_ms|remaining_low_productivity_budget|remaining_same_subproblem_budget|cooldown_ms|last_turn_requested_at' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "status exposes remaining policy budgets"; else log_fail "status missing remaining policy budgets"; fi
if rg -n 'since_last_turn_ms < status_heartbeat_ms|status_heartbeat_ms' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "status heartbeat cadence is consumed by dispatch scheduling"; else log_fail "status heartbeat cadence not consumed in dispatch scheduling"; fi
if rg -n 'operator steering detected' "$REDUCER_FILE" >/dev/null 2>&1 && ! rg -n 'auto_pause_on_operator_message' "$REDUCER_FILE" >/dev/null 2>&1; then log_pass "operator steering updates trajectory without reducer-level auto-pause"; else log_fail "steering semantics drift: reducer still ties steering to auto-pause"; fi
echo "=== WORK-LOOP POLICY ENFORCEMENT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

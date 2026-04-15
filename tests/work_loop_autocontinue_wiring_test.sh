#!/bin/bash
# SPEC-79 auto-continuation wiring: enable/select_next/turn_complete should trigger daemon-owned next-turn dispatch helpers.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
TURN_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/turn.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'pub async fn maybe_dispatch_continuous_turn_prompt' "$WORK_LOOP_FILE" >/dev/null 2>&1; then log_pass "continuous-turn prompt dispatch helper exists"; else log_fail "continuous-turn prompt dispatch helper missing"; fi
if rg -n 'maybe_auto_advance_from_blocked|WorkLoopStatus::Blocked|SelectNextContinuousSubtask' "$WORK_LOOP_FILE" >/dev/null 2>&1; then log_pass "blocked loop state can auto-advance to alternate ready work"; else log_fail "blocked loop state missing auto-advance wiring"; fi
if rg -n 'root_work_item_id: Option<String>' "$WORK_LOOP_FILE" >/dev/null 2>&1 && rg -n 'SelectNextContinuousSubtask' "$WORK_LOOP_FILE" >/dev/null 2>&1 && rg -n 'continuous work enabled with ready work selected' "$WORK_LOOP_FILE" >/dev/null 2>&1; then log_pass "enable route can seed initial ready-work selection and dispatch"; else log_fail "enable route missing initial ready-work auto-selection wiring"; fi
if rg -n 'ready work selected for continuous execution' "$WORK_LOOP_FILE" >/dev/null 2>&1; then log_pass "select_next route triggers next-turn dispatch helper"; else log_fail "select_next route missing next-turn dispatch helper"; fi
if rg -n 'continuous turn outcome evaluated and ready work remains' "$TURN_FILE" >/dev/null 2>&1; then log_pass "turn_complete triggers next-turn dispatch helper after outcome evaluation"; else log_fail "turn_complete missing follow-on dispatch helper"; fi
if rg -n 'turn_end\" \|\| kind == \"agent_end\"|pi rpc turn_end/agent_end observed and ready work remains|ObserveContinuousTurnOutcome' "$WORK_LOOP_FILE" >/dev/null 2>&1; then log_pass "Pi RPC stream events feed continuation outcome observation and follow-on dispatch"; else log_fail "Pi RPC stream events missing continuation wiring"; fi
echo "=== WORK-LOOP AUTO-CONTINUE WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

#!/bin/bash
# SPEC-79 turn completion should feed daemon continuation outcome evaluation.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TURN_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/turn.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'ObserveContinuousTurnOutcome' "$TURN_FILE" >/dev/null 2>&1; then log_pass "turn_complete wires assistant completion into continuous-turn outcome observation"; else log_fail "turn_complete missing ObserveContinuousTurnOutcome wiring"; fi
if rg -n 'work_loop_enabled' "$TURN_FILE" >/dev/null 2>&1 && rg -n 'current_task' "$TURN_FILE" >/dev/null 2>&1; then log_pass "turn_complete gates outcome observation on active work-loop task state"; else log_fail "turn_complete missing work-loop task gating"; fi
echo "=== WORK-LOOP TURN OUTCOME WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

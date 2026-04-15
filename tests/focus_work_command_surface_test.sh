#!/bin/bash
# SPEC-79 /focus-work command and Pi loop UX guardrail
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CMD_FILE="${ROOT_DIR}/apps/pi-extension/src/commands.ts"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'registerCommand\("focus-work"' "$CMD_FILE" >/dev/null 2>&1; then log_pass "/focus-work command registered"; else log_fail "/focus-work command missing"; fi
if rg -n 'on\|off\|pause\|resume\|stop\|status' "$CMD_FILE" >/dev/null 2>&1; then log_pass "/focus-work exposes the full operator command set"; else log_fail "/focus-work command set incomplete"; fi
if rg -n 'work-loop/enable|work-loop/pause|work-loop/resume|work-loop/stop' "$CMD_FILE" >/dev/null 2>&1; then log_pass "/focus-work command dispatches loop control routes"; else log_fail "/focus-work command missing loop route dispatch"; fi
if rg -n 'daemon_supervised_session|Supervision:' "$CMD_FILE" >/dev/null 2>&1 && rg -n 'Focus:' "$CMD_FILE" >/dev/null 2>&1; then log_pass "Pi loop surfaces expose supervision and focus summary"; else log_fail "Pi loop surfaces missing supervision/focus summary"; fi
if rg -n 'workLoopPreset|workLoopMaxTurns|workLoopRequireOperatorForGovernance' "$CMD_FILE" >/dev/null 2>&1; then log_pass "Pi settings panel exposes work-loop policy controls"; else log_fail "Pi settings panel missing work-loop policy controls"; fi
if rg -n 'Loop: .*Status: .*Project: .*Tranche:' "$CMD_FILE" >/dev/null 2>&1 && rg -n 'Budget:' "$CMD_FILE" >/dev/null 2>&1; then log_pass "Pi status surface exposes loop visibility details"; else log_fail "Pi status surface missing loop visibility details"; fi
echo "=== FOCUS-WORK COMMAND SURFACE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

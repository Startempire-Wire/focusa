#!/bin/bash
# SPEC-79/BD: continuous loop should record concrete BD transitions beyond claim-in-progress.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'record_bd_blocked_transition_if_possible|--append-notes' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon has blocked transition note writer for BD"; else log_fail "daemon missing blocked transition BD writer"; fi
if rg -n 'record_bd_completion_transition_if_possible|args\(\["close", work_item_id, "--reason"' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon has completion close transition writer for BD"; else log_fail "daemon missing completion close BD writer"; fi
if rg -n 'record_bd_blocked_transition_if_possible\(id, "required verification not yet satisfied"\)|record_bd_blocked_transition_if_possible\(id, "implementation remains non-conformant with linked spec"\)' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "blocked outcome paths invoke BD transition writer"; else log_fail "blocked outcomes missing BD transition write wiring"; fi
if rg -n 'record_bd_completion_transition_if_possible\(id, "verified completion; continuous loop advanced outcome"\)' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "completion outcome path invokes BD close transition writer"; else log_fail "completion outcome missing BD close transition wiring"; fi
if rg -n 'defer_work_item_for_alternate_switch|--defer", "\+1d"' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'if blocked \{' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "alternate-ready switch path records explicit BD defer transition"; else log_fail "alternate-ready switch missing explicit BD defer transition"; fi
echo "=== WORK-LOOP BD TRANSITION WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

#!/bin/bash
# SPEC-79 / Doc-77 slice B: migration/conformance execution evidence must gate completion transitions.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
WORK_LOOP_ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
TURN_ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/turn.rs"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'fn task_requires_migration_conformance_checks\(|fn migration_conformance_execution_evidenced\(' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "daemon exposes migration/conformance requirement + evidence helpers"
else
  log_fail "daemon missing migration/conformance helper functions"
fi

if rg -n 'migration/conformance execution checks not yet evidenced|checkpoint: blocked pending migration/conformance evidence' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "daemon blocks completion when migration/conformance evidence is missing"
else
  log_fail "daemon missing migration/conformance blocker enforcement"
fi

if rg -n 'record_bd_blocked_transition_if_possible' "$DAEMON_FILE" >/dev/null 2>&1 \
  && rg -n 'migration/conformance execution checks not yet evidenced' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "daemon records BD blocked transition for missing migration/conformance evidence"
else
  log_fail "daemon missing BD transition note for migration/conformance verification failures"
fi

if rg -n 'assistant_excerpt|observed from pi rpc stream: \{assistant_excerpt\}' "$WORK_LOOP_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "pi-rpc work-loop route forwards assistant evidence excerpts into outcome summaries"
else
  log_fail "pi-rpc work-loop route missing assistant evidence excerpt propagation"
fi

if rg -n 'continuous turn completed for \{\}: \{assistant_excerpt\}|evidence: \{assistant_excerpt\}' "$TURN_ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "turn route forwards assistant evidence excerpts into outcome summaries"
else
  log_fail "turn route missing assistant evidence excerpt propagation"
fi

echo "=== WORK-LOOP MIGRATION/CONFORMANCE CHECK RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

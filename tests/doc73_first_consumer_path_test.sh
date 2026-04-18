#!/bin/bash
# Consumer-path contract: doc 73 commitment lifecycle must drive at least one continuity behavior.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md"
WATCHDOG_FILE="${ROOT_DIR}/scripts/work_loop_watchdog.sh"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### Doc 73 — commitment lifecycle' "$DOC_FILE" >/dev/null 2>&1 && rg -n 'Selected first real consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 73 section names a selected first consumer"
else
  log_fail "doc 73 section missing selected first consumer"
fi

if rg -n 'scripts/work_loop_watchdog\.sh|commitment_lifecycle\.release_semantics\.state' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 73 section anchors consumer to watchdog and commitment release state"
else
  log_fail "doc 73 section missing watchdog/release-state anchors"
fi

if rg -n 'commitment_lifecycle\.release_semantics\.state' "$WATCHDOG_FILE" >/dev/null 2>&1; then
  log_pass "watchdog consumes commitment release state"
else
  log_fail "watchdog does not read commitment release state"
fi

if rg -n 'released_on_completion|released_on_blocker|released_or_unbound' "$WATCHDOG_FILE" >/dev/null 2>&1; then
  log_pass "watchdog continuity handoff is gated by commitment release states"
else
  log_fail "watchdog continuity handoff missing commitment release-state gating"
fi

if rg -n 'fn commitment_lifecycle_for_status\(' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n '"release_semantics"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "release-state source exists in work-loop status route"
else
  log_fail "release-state source missing in work-loop status route"
fi

echo "=== DOC 73 FIRST CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

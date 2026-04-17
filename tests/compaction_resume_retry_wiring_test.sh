#!/bin/bash
# SPEC-79 focusa-z0pp: compaction auto-resume retries must be uncapped while still
# bounded by pending/lifecycle governance gates.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPACTION_FILE="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
SESSION_FILE="${ROOT_DIR}/apps/pi-extension/src/session.ts"
COMMANDS_FILE="${ROOT_DIR}/apps/pi-extension/src/commands.ts"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'function scheduleCompactionResumeRetry\(' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction retry scheduler exists"
else
  log_fail "compaction retry scheduler missing"
fi

if rg -n 'if \(!S\.compactResumePending\) return;' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction retries are pending-gated"
else
  log_fail "compaction retries missing pending gate"
fi

if rg -n 'scheduleCompactionResumeRetry\(ctx, steerMessage, nextAttempt\);' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction retries recursively continue while pending"
else
  log_fail "compaction retry recursion missing"
fi

if rg -n 'maxAttempts' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "compaction retry flow still appears hard-capped"
else
  log_pass "no hard retry-cap markers remain in compaction flow"
fi

if rg -n 'Compaction resume exhausted retries' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "artificial compaction exhaustion warning still present"
else
  log_pass "no artificial compaction exhaustion warning remains"
fi

if rg -n 'S\.compactResumePending = false' "$TURNS_FILE" "$SESSION_FILE" "$COMMANDS_FILE" >/dev/null 2>&1; then
  log_pass "lifecycle/governance reset gates bound retry continuation"
else
  log_fail "lifecycle/governance reset gates missing"
fi

echo "=== COMPACTION RESUME RETRY WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

#!/bin/bash
# Post-compaction resume context must never retry a void Pi delivery API or
# start a turn that races Pi's native compaction queue.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPACTION_FILE="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"

FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'function scheduleCompactionResumeRetry\(|scheduleCompactionResumeWatchdog|compactResumeRetryTimer' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "blind post-compaction retry machinery remains"
else
  log_pass "void delivery API has no blind retry machinery"
fi

if rg -n 'triggerTurn: false, deliverAs: "nextTurn"' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "resume projection uses explicit Pi next-turn delivery"
else
  log_fail "resume projection can fall into default steering mode"
fi

if rg -n '\{ triggerTurn: true' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "post-compaction path starts a competing agent turn"
else
  log_pass "post-compaction path contains no triggerTurn=true race"
fi

if rg -n 'Pi owns prompt, queue, cancellation, reconnect, and the next agent turn' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "Pi native lifecycle ownership is explicit"
else
  log_fail "Pi native lifecycle ownership contract missing"
fi

if rg -n 'compactResumeDeliveryState = "unknown_completion"' "$COMPACTION_FILE" >/dev/null 2>&1 \
  && rg -n '\| "unknown_completion"' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "void delivery records typed unknown completion"
else
  log_fail "unknown completion is not durably typed"
fi

if rg -n 'injected on the next natural turn|deferred to the next operator turn' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "delivery failure degrades to next-turn context"
else
  log_fail "delivery failure lacks next-turn fallback"
fi

if rg -n 'focusa-compaction-verification-pending' "$COMPACTION_FILE" >/dev/null 2>&1 \
  && rg -n 'runPostCompactionVerification\(event, ctx\)' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "awaited Pi hook persists one marker and verifies in background"
else
  log_fail "nonblocking verification handoff missing"
fi

echo "=== COMPACTION RESUME DELIVERY WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

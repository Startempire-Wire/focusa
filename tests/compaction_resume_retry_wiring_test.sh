#!/bin/bash
# Post-compaction resume context must remain retryable without starting a turn
# that races Pi's native manual-compaction steering queue.
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

if rg -n 'function queueCompactionResumeContext\(' "$COMPACTION_FILE" >/dev/null 2>&1 \
  && rg -n '\{ triggerTurn: false \}' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "resume packet is queued as context without starting a competing turn"
else
  log_fail "resume packet can still bypass Pi native compaction steering ownership"
fi

if rg -n '\{ triggerTurn: true \}' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_fail "post-compaction path still starts a competing agent turn"
else
  log_pass "post-compaction path contains no triggerTurn=true race"
fi

if rg -n 'Pi 0\.82\+ owns compaction queue flushing|Pi owns flushCompactionQueue' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "Pi native queue ownership is explicit"
else
  log_fail "Pi native queue ownership contract missing"
fi

if rg -n 'if \(!getAttachmentRuntime\(\)\.compactResumePending\) return;' "$COMPACTION_FILE" >/dev/null 2>&1; then
  log_pass "compaction retries are pending-gated"
else
  log_fail "compaction retries missing pending gate"
fi

if rg -n 'scheduleCompactionResumeRetry\(ctx, steerMessage, (nextAttempt|retryAttempt \+ 1)\);' "$COMPACTION_FILE" >/dev/null 2>&1; then
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

if rg -n 'getAttachmentRuntime\(\)\.compactResumePending = false' "$COMPACTION_FILE" "$TURNS_FILE" "$SESSION_FILE" "$COMMANDS_FILE" >/dev/null 2>&1; then
  log_pass "lifecycle/governance reset gates bound retry continuation"
else
  log_fail "lifecycle/governance reset gates missing"
fi

echo "=== COMPACTION RESUME RETRY WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

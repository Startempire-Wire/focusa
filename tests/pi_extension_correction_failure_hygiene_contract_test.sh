#!/bin/bash
# Contract: operator correction steering must not be persisted as canonical failure payload.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n "export function sanitizeFocusFailures" "$STATE_FILE" >/dev/null 2>&1 \
  && rg -n "operator correction:" "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "state sanitizer strips stale operator-correction failure entries"
else
  log_fail "state sanitizer for operator-correction failures missing"
fi

if rg -n "operator_correction_detected" "$TURNS_FILE" >/dev/null 2>&1 \
  && ! rg -n "S\.localFailures\.push\(correction\)|pushDelta\(\{ failures: \[correction\] \}\)" "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "turn handler records correction as telemetry, not canonical failure"
else
  log_fail "turn handler still persists operator correction as failure"
fi

echo "=== PI EXTENSION CORRECTION FAILURE HYGIENE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

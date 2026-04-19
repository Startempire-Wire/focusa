#!/bin/bash
# Contract: `focusa start` is idempotent (no-op success if daemon already running).
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT_DIR}/target/release/focusa"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

"${BIN}" start >/tmp/focusa-start-idempotent-1.out 2>&1 || true
if "${BIN}" start >/tmp/focusa-start-idempotent-2.out 2>&1; then
  if rg -n "already running \(no-op\)" /tmp/focusa-start-idempotent-2.out >/dev/null 2>&1; then
    log_pass "second start returns success with explicit no-op message"
  else
    log_fail "second start succeeded but no explicit no-op message"
  fi
else
  log_fail "second start returned non-zero"
fi

echo "=== FOCUSA START IDEMPOTENT CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

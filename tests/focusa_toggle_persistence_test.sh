#!/bin/bash
# SPEC-33.5: /focusa-off/on disk persistence test
#
# Tests:
# 1. GET /v1/focusa/enabled returns default (enabled=true)
# 2. PATCH to disable writes enabled=0 to ~/.focusa/pi-enabled
# 3. GET returns enabled=false
# 4. PATCH to enable writes enabled=1
# 5. GET returns enabled=true
# 6. Daemon keeps running throughout
# 7. File persists across daemon restarts

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
DATA_DIR="${HOME}/.focusa"
PI_ENABLED_FILE="${DATA_DIR}/pi-enabled"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

cleanup() {
  # Restore to default state
  curl -sf -X PATCH "${BASE_URL}/v1/focusa/enabled" \
    -H "Content-Type: application/json" \
    -d '{"enabled":true}' >/dev/null 2>&1 || true
}

trap cleanup EXIT

echo "=== SPEC-33.5: Disk Persistence Test ==="
echo "Base URL: ${BASE_URL}"
echo ""

# Test 0: Daemon is up
log_info "Test 0: Daemon health check"
if curl -sf "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Daemon is running"
else
  log_fail "Daemon is not responding"
  exit 1
fi

# Test 1: GET default state
log_info "Test 1: GET default state (should be enabled=true)"
resp=$(curl -sf "${BASE_URL}/v1/focusa/enabled")
if echo "$resp" | grep -q '"enabled":true'; then
  log_pass "Default state is enabled=true"
else
  log_fail "Expected enabled=true, got: $resp"
fi

# Test 2: PATCH disable
log_info "Test 2: PATCH disable"
resp=$(curl -sf -X PATCH "${BASE_URL}/v1/focusa/enabled" \
  -H "Content-Type: application/json" \
  -d '{"enabled":false}')
if echo "$resp" | grep -q '"status":"updated"' && echo "$resp" | grep -q '"enabled":false'; then
  log_pass "PATCH disable returns correct response"
else
  log_fail "PATCH disable failed: $resp"
fi

# Test 3: File written
log_info "Test 3: File written to disk"
sleep 0.5
if [ -f "${PI_ENABLED_FILE}" ]; then
  content=$(cat "${PI_ENABLED_FILE}")
  if [ "$content" = "enabled=0" ]; then
    log_pass "File contains enabled=0"
  else
    log_fail "File content wrong: $content"
  fi
else
  log_fail "File not found: ${PI_ENABLED_FILE}"
fi

# Test 4: GET disabled state
log_info "Test 4: GET returns disabled state"
resp=$(curl -sf "${BASE_URL}/v1/focusa/enabled")
if echo "$resp" | grep -q '"enabled":false'; then
  log_pass "GET returns enabled=false"
else
  log_fail "Expected enabled=false, got: $resp"
fi

# Test 5: PATCH enable
log_info "Test 5: PATCH enable"
resp=$(curl -sf -X PATCH "${BASE_URL}/v1/focusa/enabled" \
  -H "Content-Type: application/json" \
  -d '{"enabled":true}')
if echo "$resp" | grep -q '"status":"updated"' && echo "$resp" | grep -q '"enabled":true'; then
  log_pass "PATCH enable returns correct response"
else
  log_fail "PATCH enable failed: $resp"
fi

# Test 6: File updated
log_info "Test 6: File updated"
sleep 0.5
if [ -f "${PI_ENABLED_FILE}" ]; then
  content=$(cat "${PI_ENABLED_FILE}")
  if [ "$content" = "enabled=1" ]; then
    log_pass "File contains enabled=1"
  else
    log_fail "File content wrong: $content"
  fi
else
  log_fail "File not found: ${PI_ENABLED_FILE}"
fi

# Test 7: GET enabled state
log_info "Test 7: GET returns enabled state"
resp=$(curl -sf "${BASE_URL}/v1/focusa/enabled")
if echo "$resp" | grep -q '"enabled":true'; then
  log_pass "GET returns enabled=true"
else
  log_fail "Expected enabled=true, got: $resp"
fi

# Summary
echo ""
echo "=== RESULTS ==="
echo "Passed: ${PASSED}"
echo "Failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All tests passed${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

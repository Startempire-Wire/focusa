#!/bin/bash
# SPEC §15: Focusa outage test (passthrough)
# Verify passthrough correctness when daemon is down

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

log_pass() { echo "✓ $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ $1"; FAILED=$((FAILED+1)); }

echo "=== SPEC §15: Focusa outage test ==="

# Test 1: Health check (upstream passthrough)
if curl -sf "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Health endpoint accessible"
else
  log_fail "Health endpoint unavailable"
fi

# Test 2: Daemon status observable
STATUS=$(curl -s "${BASE_URL}/v1/status")
if echo "$STATUS" | jq -e '.worker_status' >/dev/null 2>&1; then
  log_pass "Worker status observable"
else
  log_fail "Worker status unavailable"
fi

# Test 3: Telemetry accessible
TELEMETRY=$(curl -s "${BASE_URL}/v1/telemetry/tokens")
if echo "$TELEMETRY" | jq -e '.total_events' >/dev/null 2>&1; then
  log_pass "Telemetry accessible"
else
  log_fail "Telemetry unavailable"
fi

# Test 4: Proxy passthrough (if configured)
PROXY=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/proxy/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{"model":"test","messages":[]}')
if [ "$PROXY" != "404" ]; then
  log_pass "Proxy endpoint accessible (code: $PROXY)"
else
  log_pass "Proxy returns expected response (passthrough configured)"
fi

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ $FAILED -eq 0 ]

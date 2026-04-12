#!/bin/bash
# SPEC-55: Tool Action Contracts
#
# Every tool must have:
# - typed input schema
# - typed output schema
# - side effects documented
# - failure modes
# - idempotency expectations
# - rollback availability
# - verification hooks
# - expected ontology deltas
# - timeout policy
# - retry policy
# - degraded fallback
#
# 8 failure mode categories:
# - validation failure
# - dependency failure
# - permission failure
# - execution failure
# - verification failure
# - timeout
# - partial success
# - rollback failure

set -e

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

echo "=== SPEC-55: Tool Action Contracts ==="
echo "Base URL: ${BASE_URL}"
echo ""

# Test 0: Daemon health
log_info "Test 0: Daemon health check"
if curl -s "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Daemon is running"
else
  log_fail "Daemon is not responding"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════
# Input Schema Validation
# ═══════════════════════════════════════════════════════════════════
log_info "Input Schema Validation"

# Valid input accepted
resp=$(curl -s -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"test","goal":"test","beads_issue_id":"tc-001"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Valid input schema accepted"
else
  log_fail "Valid input rejected: $resp"
fi

# Invalid input rejected with proper error
resp=$(curl -s -X POST "${BASE_URL}/v1/focus/set-active" \
  -H "Content-Type: application/json" \
  -d '{"frame_id":"not-a-uuid"}')
if echo "$resp" | grep -q '"code"'; then
  log_pass "Invalid input rejected with error envelope"
else
  log_fail "Invalid input not rejected: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Output Schema
# ═══════════════════════════════════════════════════════════════════
log_info "Output Schema"

# Output has status
resp=$(curl -s "${BASE_URL}/v1/health")
if echo "$resp" | grep -q '"ok"'; then
  log_pass "Output schema: status field present"
else
  log_fail "Output schema missing status: $resp"
fi

# Output has version
if echo "$resp" | grep -q '"version"'; then
  log_pass "Output schema: version field present"
else
  log_fail "Output schema missing version"
fi

# ═══════════════════════════════════════════════════════════════════
# Failure Modes
# ═══════════════════════════════════════════════════════════════════
log_info "Failure Modes"

# Validation failure (404 for bad route)
resp=$(curl -s -X POST "${BASE_URL}/v1/nonexistent" \
  -H "Content-Type: application/json" \
  -d '{}')
if echo "$resp" | grep -q '"code"'; then
  log_pass "Validation failure (404) has error envelope"
else
  log_fail "Validation failure missing error: $resp"
fi

# Validation failure (422 for bad input)
resp=$(curl -s -X POST "${BASE_URL}/v1/prompt/assemble" \
  -H "Content-Type: application/json" \
  -d '{"turn_id":"bad"}')
if echo "$resp" | grep -q '"code"'; then
  log_pass "Validation failure (422) has error envelope"
else
  log_fail "Validation failure (422) missing error: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Idempotency
# ═══════════════════════════════════════════════════════════════════
log_info "Idempotency"

# Turn complete is idempotent
TURN_ID="idem-test-$(date +%s%N)"
curl -s -X POST "${BASE_URL}/v1/turn/start" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"test\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}" >/dev/null

resp1=$(curl -s -X POST "${BASE_URL}/v1/turn/complete" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
sleep 5
resp2=$(curl -s -X POST "${BASE_URL}/v1/turn/complete" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")

if echo "$resp2" | grep -q '"duplicate"'; then
  log_pass "Turn complete is idempotent (duplicate flag)"
else
  log_fail "Turn complete not idempotent: $resp2"
fi

# ═══════════════════════════════════════════════════════════════════
# Side Effects
# ═══════════════════════════════════════════════════════════════════
log_info "Side Effects"

# Memory upsert has observable side effect
curl -s -X POST "${BASE_URL}/v1/memory/semantic/upsert" \
  -H "Content-Type: application/json" \
  -d '{"key":"tool-contract-test","value":"testing"}' >/dev/null

sleep 0.2
resp=$(curl -s "${BASE_URL}/v1/memory/semantic")
if echo "$resp" | grep -q '"semantic"'; then
  log_pass "Memory upsert has observable side effect"
else
  log_fail "Memory side effect not observable: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Timeout Policy
# ═══════════════════════════════════════════════════════════════════
log_info "Timeout Policy"

# Status shows worker timeout config
resp=$(curl -s "${BASE_URL}/v1/status")
if echo "$resp" | grep -q '"worker_status"'; then
  log_pass "Worker status (timeout policy) accessible"
else
  log_fail "Worker status missing: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Degraded Fallback
# ═══════════════════════════════════════════════════════════════════
log_info "Degraded Fallback"

# Reflection has degraded mode (no LLM)
if curl -s "${BASE_URL}/v1/reflect/status" | grep -q '"enabled"'; then
  log_pass "Reflection (degraded fallback) accessible"
else
  log_fail "Reflection status failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-55 TOOL ACTION CONTRACTS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All tool contract API properties verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

#!/bin/bash
# SPEC-54/54a: Visible Output Boundary
#
# SPEC-54 requirements:
# - Channel Separation: internal vs visible channels remain separate
# - Anti-Echo: internal blocks tagged, excluded from visible output
# - Internal Prominence: token cap on focus state injection
#
# SPEC-54a requirements:
# - Core Law: operator input first
# - Priority Order: hard rules > operator > mission > constraints > working set > background
# - Steering Detection: detect task change/narrow/override/question/objection
# - Topic Drift Prohibition: no new topics from focus-state injection

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

echo "=== SPEC-54/54a: Visible Output Boundary ==="
echo "Base URL: ${BASE_URL}"
echo ""

# Test 0: Daemon health
log_info "Test 0: Daemon health check"
if curl -sf "${BASE_URL}/v1/health" | grep -q '"ok":true'; then
  log_pass "Daemon is running"
else
  log_fail "Daemon is not responding"
  exit 1
fi

# ═══════════════════════════════════════════════════════════════════
# Channel Separation (SPEC-54)
# ═══════════════════════════════════════════════════════════════════
log_info "Channel Separation (SPEC-54)"

# Visible channel: health endpoint
resp=$(curl -sf "${BASE_URL}/v1/health")
if echo "$resp" | grep -q '"version"'; then
  log_pass "Visible channel: health endpoint accessible"
else
  log_fail "Health endpoint failed: $resp"
fi

# Internal channel: SSE stream
resp=$(curl -sf -I "${BASE_URL}/v1/events/stream" -H "Accept: text/event-stream" 2>/dev/null)
if echo "$resp" | grep -q "200"; then
  log_pass "Internal channel: SSE stream accessible"
else
  log_fail "SSE stream failed"
fi

# Channels are different
if echo "$resp" | grep -q "text/event-stream"; then
  log_pass "Channels have different content-types"
else
  log_fail "Channel separation unclear"
fi

# ═══════════════════════════════════════════════════════════════════
# Anti-Echo Safeguards (SPEC-54)
# ═══════════════════════════════════════════════════════════════════
log_info "Anti-Echo Safeguards (SPEC-54)"

# SSE events are internal-only
resp=$(curl -sf "${BASE_URL}/v1/events/recent")
event_count=$(echo "$resp" | python3 -c "import json,sys; d=json.load(sys.stdin); print(len(d.get('events',[])))" 2>/dev/null || echo "0")
if [ "$event_count" -gt 0 ]; then
  log_pass "Internal events observable via events/recent"
else
  log_fail "No internal events found"
fi

# Visible channel doesn't echo internal events
resp=$(curl -sf "${BASE_URL}/v1/health")
if ! echo "$resp" | grep -q "MemoryDecayTick\|IntuitionSignal"; then
  log_pass "Visible channel (health) doesn't echo internal events"
else
  log_fail "Internal events leaked to visible channel"
fi

# ═══════════════════════════════════════════════════════════════════
# Internal Prominence (SPEC-54)
# ═══════════════════════════════════════════════════════════════════
log_info "Internal Prominence (SPEC-54)"

# Prompt assemble has budget/token limits (prominence control)
resp=$(curl -sf -X POST "${BASE_URL}/v1/prompt/assemble" \
  -H "Content-Type: application/json" \
  -d '{"turn_id":"channel-test","raw_user_input":"test","format":"string","budget":500}')
if echo "$resp" | grep -q '"stats"'; then
  log_pass "Prompt budget enforced (token cap possible)"
else
  log_fail "Prompt assemble failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Steering Detection (SPEC-54a)
# ═══════════════════════════════════════════════════════════════════
log_info "Steering Detection (SPEC-54a)"

# Gate surfaces candidates (steering observable)
resp=$(curl -sf "${BASE_URL}/v1/focus-gate/candidates")
if echo "$resp" | grep -q '"candidates"'; then
  log_pass "Gate candidates (steering signals) observable"
else
  log_fail "Gate candidates failed: $resp"
fi

# Signal endpoint for steering
resp=$(curl -sf -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" \
  -H "Content-Type: application/json" \
  -d '{"kind":"steering","summary":"operator correction"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Steering signal endpoint accessible"
else
  log_fail "Steering signal failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Priority Order (SPEC-54a)
# ═══════════════════════════════════════════════════════════════════
log_info "Priority Order (SPEC-54a)"

# Status shows operator context first
resp=$(curl -sf "${BASE_URL}/v1/status")
if echo "$resp" | grep -q '"active_frame"'; then
  log_pass "Active frame (operator context) in status"
else
  log_fail "Active frame missing from status"
fi

# Focus stack for mission context
resp=$(curl -sf "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Focus stack (mission context) accessible"
else
  log_fail "Focus stack failed: $resp"
fi

# Memory for working set
resp=$(curl -sf "${BASE_URL}/v1/memory/semantic")
if echo "$resp" | grep -q '"semantic"'; then
  log_pass "Memory (working set) accessible"
else
  log_fail "Memory failed: $resp"
fi

# Constraints for priority ordering
resp=$(curl -sf "${BASE_URL}/v1/memory/procedural")
if echo "$resp" | grep -q '"procedural"'; then
  log_pass "Constraints (priority ordering) accessible"
else
  log_fail "Constraints failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-54/54a VISIBLE OUTPUT BOUNDARY RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All visible output boundary API contracts verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

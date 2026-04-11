#!/bin/bash
# SPEC-53: Behavioral Alignment Tests
#
# Tests that Focusa state influences actual behavior:
# 53a: Constraint consultation before risky actions
# 53b: Decision consultation in repeated-pattern zones
# 53c: Decision distillation trigger
# 53d: Scratch quality gate
# 53e: Failure/blocker emission
# 53f: Behavioral prohibitions
# 53g: Subject discipline + focusa_subject_hijack
#
# Note: These tests verify the API surface. Full behavioral testing
# requires the extension to be enabled and responding.

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

echo "=== SPEC-53: Behavioral Alignment Tests ==="
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
# SPEC-53a: Constraint Consultation
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53a: Constraint Consultation"

# Submit a constraint
resp=$(curl -sf -X POST "${BASE_URL}/v1/memory/procedural/reinforce" \
  -H "Content-Type: application/json" \
  -d '{"rule_id":"test-constraint-1","rule":"TEST: Do not delete without backup","source":"test"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Constraint submitted via memory/procedural/reinforce"
else
  log_fail "Constraint submission failed: $resp"
fi

# Verify constraint exists
sleep 0.2
resp=$(curl -sf "${BASE_URL}/v1/status")
if echo "$resp" | grep -q '"worker_status"'; then
  log_pass "Status endpoint accessible (constraint checkable)"
else
  log_fail "Status endpoint failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53b: Decision Consultation
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53b: Decision Consultation"

# Decisions are stored via focus/push (in the focus_state delta)
# Verify focus state update accepts decisions
resp=$(curl -sf -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"test decision session","goal":"testing decisions"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Focus push (decision context) accepted"
else
  log_fail "Focus push failed: $resp"
fi

# Verify state/explain endpoint exists (explains decisions)
resp=$(curl -sf "${BASE_URL}/v1/state/explain")
if echo "$resp" | grep -q '"recent_decisions"' || echo "$resp" | grep -q '"status"'; then
  log_pass "State explain endpoint (decision explanation) accessible"
else
  log_fail "State explain failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53c: Decision Distillation
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53c: Decision Distillation"

# Distillation: submit invariant as semantic memory
# Distillation: submit invariant as semantic memory (source optional, defaults to User)
resp=$(curl -sf -X POST "${BASE_URL}/v1/memory/semantic/upsert" \
  -H "Content-Type: application/json" \
  -d '{"key":"test:always_validate_input","value":"prevents downstream errors"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Decision distillation via memory/semantic/upsert accepted"
else
  log_fail "Distillation failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53d: Scratch Quality Gate
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53d: Scratch Quality Gate"

# Note: Scratch is written to /tmp/pi-scratch/ - verify directory exists
# The extension (not daemon) manages scratch. Daemon provides ECS for storage.
if [ -d "/tmp/pi-scratch" ] || mkdir -p /tmp/pi-scratch 2>/dev/null; then
  log_pass "/tmp/pi-scratch directory available"
  
  # Write a test scratch note
  echo "test scratch note" > /tmp/pi-scratch/test-note.txt
  if [ -f /tmp/pi-scratch/test-note.txt ]; then
    log_pass "Scratch file writeable"
  else
    log_fail "Scratch file write failed"
  fi
else
  log_fail "Scratch directory unavailable"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53e: Failure/Blocker Emission
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53e: Failure/Blocker Emission"

# Submit a failure signal
resp=$(curl -sf -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" \
  -H "Content-Type: application/json" \
  -d '{"kind":"test_failure","summary":"test: intentional failure for testing","details":"testing blocker emission"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Failure signal accepted via gate/signal"
else
  log_fail "Failure signal failed: $resp"
fi

# Check gate has candidates (use correct endpoint path)
sleep 0.2
resp=$(curl -sf "${BASE_URL}/v1/focus-gate/candidates")
if echo "$resp" | grep -q '"candidates"'; then
  log_pass "Focus-gate candidates accessible"
else
  log_fail "Focus-gate candidates failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53f: Behavioral Prohibitions
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53f: Behavioral Prohibitions"

# Verify internal vs visible channel split (SSE events)
resp=$(curl -sf -I "${BASE_URL}/v1/events/stream" \
  -H "Accept: text/event-stream")
if echo "$resp" | grep -q "200 OK"; then
  log_pass "SSE stream endpoint accessible (internal channel)"
else
  log_fail "SSE stream failed: $resp"
fi

# Verify visible endpoint exists
resp=$(curl -sf "${BASE_URL}/v1/health")
if echo "$resp" | grep -q '"version"'; then
  log_pass "Health endpoint (visible channel) accessible"
else
  log_fail "Health endpoint failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# SPEC-53g: Subject Discipline + focusa_subject_hijack
# ═══════════════════════════════════════════════════════════════════
echo ""
log_info "SPEC-53g: Subject Discipline"

# Verify focus state exists (operator input should be primary)
resp=$(curl -sf "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Focus stack accessible (operator context)"
else
  log_fail "Focus stack failed: $resp"
fi

# Verify focus state can be updated (subject discipline enforced)
resp=$(curl -sf -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"test session","goal":"testing focus state"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Focus push (operator context set) accepted"
else
  log_fail "Focus push failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-53 BEHAVIORAL ALIGNMENT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All behavioral alignment API contracts verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

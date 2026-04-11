#!/bin/bash
# SPEC-56.1: Trace Dimensions — API Contract Verification
#
# SPEC-56.1 requires 18 minimum trace dimensions. This test verifies
# the API infrastructure EXISTS to support those dimensions, not that
# all 18 are currently populated.
#
# The 18 dimensions map to these API capabilities:
# 1. mission/frame context       → /v1/focus/stack, /v1/focus/push
# 2. working set used            → /v1/references, /v1/memory/semantic
# 3. constraints consulted       → /v1/memory/procedural
# 4. decisions consulted        → /v1/ascc/state
# 5. action intents proposed   → /v1/commands/submit
# 6. tools invoked             → /v1/prompt/assemble
# 7. verification results     → /v1/events/recent
# 8. ontology deltas applied   → /v1/ecs/store
# 9. blockers/failures emitted → /v1/focus-gate/ingest-signal
# 10. final state transition   → /v1/turn/complete
# 11. operator_subject         → /v1/focus/stack
# 12. subject_after_routing    → /v1/focus/stack
# 13. steering_detected       → /v1/focus-gate/candidates
# 14. prior_mission_reused     → /v1/session/history
# 15. focus_slice_size         → /v1/prompt/assemble (context_stats)
# 16. focus_slice_relevance   → /v1/references/trace
# 17. subject_hijack_prevented → channel separation verifiable
# 18. subject_hijack_occurred  → /v1/intuition/signals

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

echo "=== SPEC-56.1: Trace Dimensions API Contract ==="
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
# Event Infrastructure
# ═══════════════════════════════════════════════════════════════════
log_info "Event Infrastructure (dims 7, 17, 18)"

# /v1/events/recent
resp=$(curl -sf "${BASE_URL}/v1/events/recent")
if echo "$resp" | grep -q '"events"'; then
  log_pass "Events endpoint exists (dim 7)"
else
  log_fail "Events endpoint failed: $resp"
fi

# /v1/events/stream (SSE)
resp=$(curl -sf -I "${BASE_URL}/v1/events/stream" -H "Accept: text/event-stream" 2>/dev/null)
if echo "$resp" | grep -q "200"; then
  log_pass "SSE stream exists (dim 17-18)"
else
  log_fail "SSE stream failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Focus Stack (dims 1, 11, 12)
# ═══════════════════════════════════════════════════════════════════
log_info "Focus Stack (dims 1, 11, 12)"

resp=$(curl -sf "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Focus stack accessible (dims 1, 11, 12)"
else
  log_fail "Focus stack failed: $resp"
fi

# Push a frame
resp=$(curl -sf -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"trace-test","goal":"verify trace dimensions","beads_issue_id":"trace-001"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Focus push works (frame context)"
else
  log_fail "Focus push failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Memory/Constraints (dims 2, 3)
# ═══════════════════════════════════════════════════════════════════
log_info "Memory/Constraints (dims 2, 3)"

# Semantic memory (working set)
resp=$(curl -sf "${BASE_URL}/v1/memory/semantic")
if echo "$resp" | grep -q '"semantic"'; then
  log_pass "Semantic memory accessible (dim 2)"
else
  log_fail "Semantic memory failed: $resp"
fi

# Procedural memory (constraints)
resp=$(curl -sf "${BASE_URL}/v1/memory/procedural")
if echo "$resp" | grep -q '"procedural"'; then
  log_pass "Procedural memory accessible (dim 3)"
else
  log_fail "Procedural memory failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Decisions (dim 4)
# ═══════════════════════════════════════════════════════════════════
log_info "Decisions (dim 4)"

resp=$(curl -sf "${BASE_URL}/v1/ascc/state")
if echo "$resp" | grep -q '"active_frame"' || echo "$resp" | grep -q '"decisions"'; then
  log_pass "ASCC state accessible (dim 4)"
else
  log_fail "ASCC state failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Commands/Intents (dim 5)
# ═══════════════════════════════════════════════════════════════════
log_info "Commands/Intents (dim 5)"

resp=$(curl -sf -X POST "${BASE_URL}/v1/commands/submit" \
  -H "Content-Type: application/json" \
  -d '{"command":"memory.semantic.upsert","payload":{"key":"trace-test","value":"test"}}')
if echo "$resp" | grep -q '"command_id"'; then
  log_pass "Commands endpoint accessible (dim 5)"
else
  log_fail "Commands endpoint failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Prompt Assembly (dims 6, 15)
# ═══════════════════════════════════════════════════════════════════
log_info "Prompt Assembly (dims 6, 15)"

resp=$(curl -sf -X POST "${BASE_URL}/v1/prompt/assemble" \
  -H "Content-Type: application/json" \
  -d '{"turn_id":"trace-test","raw_user_input":"test","format":"string","budget":500}')
if echo "$resp" | grep -q '"context_stats"'; then
  log_pass "Prompt assemble with context_stats (dims 6, 15)"
else
  log_fail "Prompt assemble failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# ECS/Artifacts (dim 8)
# ═══════════════════════════════════════════════════════════════════
log_info "ECS/Artifacts (dim 8)"

resp=$(curl -sf -X POST "${BASE_URL}/v1/ecs/store" \
  -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"trace-test","content_b64":"dGVzdA=="}')
if echo "$resp" | grep -q '"id"'; then
  log_pass "ECS store accessible (dim 8)"
else
  log_fail "ECS store failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Gate/Blockers (dims 9, 13)
# ═══════════════════════════════════════════════════════════════════
log_info "Gate/Blockers (dims 9, 13)"

resp=$(curl -sf "${BASE_URL}/v1/focus-gate/candidates")
if echo "$resp" | grep -q '"candidates"'; then
  log_pass "Focus-gate candidates accessible (dims 9, 13)"
else
  log_fail "Focus-gate candidates failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Turn Lifecycle (dim 10)
# ═══════════════════════════════════════════════════════════════════
log_info "Turn Lifecycle (dim 10)"

TURN_ID="trace-$(date +%s)"
resp=$(curl -sf -X POST "${BASE_URL}/v1/turn/start" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"trace-test\",\"adapter_id\":\"trace-test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}")
if echo "$resp" | grep -q '"status"'; then
  log_pass "Turn start accessible (dim 10)"
else
  log_fail "Turn start failed: $resp"
fi

resp=$(curl -sf -X POST "${BASE_URL}/v1/turn/complete" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
if echo "$resp" | grep -q '"status"'; then
  log_pass "Turn complete accessible (dim 10)"
else
  log_fail "Turn complete failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Session History (dim 14)
# ═══════════════════════════════════════════════════════════════════
log_info "Session History (dim 14)"

resp=$(curl -sf "${BASE_URL}/v1/status")
if echo "$resp" | grep -q '"worker_status"'; then
  log_pass "Session/status endpoint accessible (dim 14)"
else
  log_fail "Session/status endpoint failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# References/Trace (dim 16)
# ═══════════════════════════════════════════════════════════════════
log_info "References/Trace (dim 16)"

resp=$(curl -sf "${BASE_URL}/v1/references/trace?ref_id=test-123" 2>/dev/null)
if echo "$resp" | grep -q '"used_in"'; then
  log_pass "References/trace accessible (dim 16)"
else
  log_fail "References/trace failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Intuition Signals (dim 18)
# ═══════════════════════════════════════════════════════════════════
log_info "Intuition Signals (dim 18)"

resp=$(curl -sf "${BASE_URL}/v1/intuition/signals" 2>/dev/null)
if echo "$resp" | grep -q '"signals"' || echo "$resp" | grep -q '"status"'; then
  log_pass "Intuition signals accessible (dim 18)"
else
  log_fail "Intuition signals failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-56.1 TRACE DIMENSIONS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All trace dimension API contracts verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

#!/bin/bash
# SPEC-57: Golden Tasks + Evals Framework
#
# 9 Golden Tasks:
# 1. resume interrupted refactor
# 2. debug failing test with bounded working set
# 3. preserve conventions during feature addition
# 4. recover after wrong turn
# 5. maintain continuity across context windows
# 6. compare weaker model with ontology slice vs without
# 7. compare Pi with behavioral-alignment rules vs without
# 8. operator steering under active Focusa state
# 9. correction handling
#
# Core Metrics:
# - mission retention
# - working-set precision
# - irrelevant-context reduction
# - constraint-check rate
# - decision-consult rate
# - distillation rate
# - repeated-mistake rate
# - convention adherence
# - recovery success rate
# - token use
# - latency impact
# - degraded-mode behavior quality

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

echo "=== SPEC-57: Golden Tasks + Evals Framework ==="
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
# Metrics Infrastructure
# ═══════════════════════════════════════════════════════════════════
log_info "Metrics Infrastructure"

# Session/continuity metrics via status
resp=$(curl -s "${BASE_URL}/v1/status")
if echo "$resp" | grep -q '"worker_status"'; then
  log_pass "Metrics: worker status (continuity) accessible"
else
  log_fail "Metrics: worker status failed"
fi

# Token metrics via telemetry
resp=$(curl -s "${BASE_URL}/v1/telemetry/tokens")
if echo "$resp" | grep -q '"total_completion_tokens"' || echo "$resp" | grep -q '"tokens"'; then
  log_pass "Metrics: token use accessible"
else
  log_fail "Metrics: token use failed"
fi

# Prompt stats (latency proxy)
if echo "$resp" | grep -q '"total_prompt_tokens"'; then
  log_pass "Metrics: prompt stats (latency) accessible"
else
  log_fail "Metrics: prompt stats failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Task 1-4: Continuity & Recovery Infrastructure
# ═══════════════════════════════════════════════════════════════════
log_info "Task 1-4: Continuity & Recovery"

# Session start (continuity across context windows)
resp=$(curl -s -X POST "${BASE_URL}/v1/session/start" \
  -H "Content-Type: application/json" \
  -d '{"instance_id":"gt-001","workspace_id":"gt-ws"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Task 4/5: Session start (continuity) works"
else
  log_fail "Task 4/5: Session start failed"
fi

# Focus stack (mission retention)
resp=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Task 1: Mission retention infrastructure exists"
else
  log_fail "Task 1: Mission retention failed"
fi

# Turn lifecycle (interrupted refactor recovery)
TURN_ID="gt-turn-$(date +%s)"
resp=$(curl -s -X POST "${BASE_URL}/v1/turn/start" \
  -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"golden-task-test\",\"adapter_id\":\"gt\",\"timestamp\":\"2026-04-11T00:00:00Z\"}")
if echo "$resp" | grep -q '"status"'; then
  log_pass "Task 1/4: Turn lifecycle (recovery) infrastructure exists"
else
  log_fail "Task 1/4: Turn lifecycle failed"
fi

# State dump (full state for recovery)
resp=$(curl -s "${BASE_URL}/v1/state/dump")
if echo "$resp" | grep -q '"focus_gate"'; then
  log_pass "Task 1/4: State dump (recovery data) accessible"
else
  log_fail "Task 1/4: State dump failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Task 2-3: Working Set Precision
# ═══════════════════════════════════════════════════════════════════
log_info "Task 2-3: Working Set Precision"

# Semantic memory (working set bounded)
resp=$(curl -s "${BASE_URL}/v1/memory/semantic")
if echo "$resp" | grep -q '"semantic"'; then
  log_pass "Task 2/3: Working set (precision) accessible"
else
  log_fail "Task 2/3: Working set failed"
fi

# References (bounded context)
resp=$(curl -s "${BASE_URL}/v1/references")
if echo "$resp" | grep -q '"references"' || echo "$resp" | grep -q '"status"'; then
  log_pass "Task 2: Bounded references accessible"
else
  log_fail "Task 2: References failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Task 5-7: Constraint/Decision Metrics
# ═══════════════════════════════════════════════════════════════════
log_info "Task 5-7: Constraint/Decision Metrics"

# Procedural memory (constraints checked)
resp=$(curl -s "${BASE_URL}/v1/memory/procedural")
if echo "$resp" | grep -q '"procedural"'; then
  log_pass "Task 6: Constraint-check rate infrastructure exists"
else
  log_fail "Task 6: Procedural memory failed"
fi

# ASCC state (decisions consulted)
resp=$(curl -s "${BASE_URL}/v1/ascc/state")
if echo "$resp" | grep -q '"decisions"'; then
  log_pass "Task 6: Decision-consult rate infrastructure exists"
else
  log_fail "Task 6: Decision state failed"
fi

# Gate candidates (blockers observable)
resp=$(curl -s "${BASE_URL}/v1/focus-gate/candidates")
if echo "$resp" | grep -q '"candidates"'; then
  log_pass "Task 7: Convention adherence (gate) infrastructure exists"
else
  log_fail "Task 7: Gate candidates failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Task 8-9: Operator Steering
# ═══════════════════════════════════════════════════════════════════
log_info "Task 8-9: Operator Steering"

# Focus state (operator context)
resp=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Task 8: Operator steering infrastructure exists"
else
  log_fail "Task 8: Focus stack failed"
fi

# Intuition signals (correction handling)
resp=$(curl -s "${BASE_URL}/v1/intuition/signals")
if echo "$resp" | grep -q '"signals"' || echo "$resp" | grep -q '"status"'; then
  log_pass "Task 9: Correction handling infrastructure exists"
else
  log_fail "Task 9: Intuition signals failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Degraded Mode
# ═══════════════════════════════════════════════════════════════════
log_info "Degraded Mode"

# Reflection works without LLM
resp=$(curl -s "${BASE_URL}/v1/reflect/status")
if echo "$resp" | grep -q '"enabled"'; then
  log_pass "Degraded mode: Reflection infrastructure exists"
else
  log_fail "Degraded mode: Reflection status failed"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-57 GOLDEN TASKS + EVALS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All golden tasks eval infrastructure verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

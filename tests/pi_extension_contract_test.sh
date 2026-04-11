#!/bin/bash
# SPEC-52: Pi Extension Contract
#
# Pi Input Contract — daemon must supply:
# 1. active mission
# 2. active frame / thesis
# 3. active working set
# 4. applicable constraints
# 5. recent relevant decisions
# 6. recent verified deltas
# 7. unresolved blockers/open loops
# 8. allowed actions
# 9. degraded-mode flag
#
# Pi Output Contract — Pi may emit (8 types):
# - OntologyProposal
# - OntologyActionIntent
# - VerificationRequest
# - EvidenceLinkedObservation
# - FailureSignal
# - BlockerSignal
# - ScratchReasoningRecord
# - DecisionCandidate

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

echo "=== SPEC-52: Pi Extension Contract ==="
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
# Pi Input Contract (9 fields)
# ═══════════════════════════════════════════════════════════════════
log_info "Pi Input Contract (9 fields)"

# Setup: Create active frame
curl -s -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"pi-contract-test","goal":"testing input contract","beads_issue_id":"pi-ct-001"}' >/dev/null

# 1. active mission (via focus stack)
resp=$(curl -s "${BASE_URL}/v1/focus/stack")
if echo "$resp" | grep -q '"stack"'; then
  log_pass "Input 1: active mission accessible"
else
  log_fail "Input 1: active mission missing"
fi

# 2. active frame / thesis (via ASCC state)
resp=$(curl -s "${BASE_URL}/v1/ascc/state")
if echo "$resp" | grep -q '"active_frame"' || echo "$resp" | grep -q '"decisions"'; then
  log_pass "Input 2: active frame/thesis accessible"
else
  log_fail "Input 2: active frame/thesis missing"
fi

# 3. active working set (via semantic memory)
resp=$(curl -s "${BASE_URL}/v1/memory/semantic")
if echo "$resp" | grep -q '"semantic"'; then
  log_pass "Input 3: active working set accessible"
else
  log_fail "Input 3: active working set missing"
fi

# 4. applicable constraints (via procedural memory)
resp=$(curl -s "${BASE_URL}/v1/memory/procedural")
if echo "$resp" | grep -q '"procedural"'; then
  log_pass "Input 4: applicable constraints accessible"
else
  log_fail "Input 4: applicable constraints missing"
fi

# 5. recent relevant decisions (via ASCC state)
resp=$(curl -s "${BASE_URL}/v1/ascc/state")
if echo "$resp" | grep -q '"decisions"'; then
  log_pass "Input 5: recent decisions accessible"
else
  log_fail "Input 5: recent decisions missing"
fi

# 6. recent verified deltas (via ECS store handles)
resp=$(curl -s "${BASE_URL}/v1/ecs/handles")
if echo "$resp" | grep -q '"handles"'; then
  log_pass "Input 6: recent verified deltas (ECS) accessible"
else
  log_fail "Input 6: recent verified deltas missing"
fi

# 7. unresolved blockers/open loops (via focus-gate)
resp=$(curl -s "${BASE_URL}/v1/focus-gate/candidates")
if echo "$resp" | grep -q '"candidates"'; then
  log_pass "Input 7: unresolved blockers/open loops accessible"
else
  log_fail "Input 7: unresolved blockers missing"
fi

# 8. allowed actions (via commands/submit)
resp=$(curl -s "${BASE_URL}/v1/commands/submit" \
  -H "Content-Type: application/json" \
  -d '{"command":"memory.semantic.upsert","payload":{"key":"pi-test","value":"test"}}')
if echo "$resp" | grep -q '"command_id"'; then
  log_pass "Input 8: allowed actions (commands) accessible"
else
  log_fail "Input 8: allowed actions missing"
fi

# 9. degraded-mode flag (via autonomy status)
resp=$(curl -s "${BASE_URL}/v1/autonomy")
if echo "$resp" | grep -q '"recommendation"' || echo "$resp" | grep -q '"status"'; then
  log_pass "Input 9: degraded-mode flag accessible"
else
  log_fail "Input 9: degraded-mode flag missing"
fi

# ═══════════════════════════════════════════════════════════════════
# Pi Output Contract (8 emit types)
# ═══════════════════════════════════════════════════════════════════
log_info "Pi Output Contract (8 emit types)"

# FailureSignal via gate/ingest-signal
resp=$(curl -s -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" \
  -H "Content-Type: application/json" \
  -d '{"kind":"failure","summary":"pi output contract test"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Output: FailureSignal endpoint accessible"
else
  log_fail "Output: FailureSignal endpoint failed: $resp"
fi

# BlockerSignal via gate/ingest-signal
resp=$(curl -s -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" \
  -H "Content-Type: application/json" \
  -d '{"kind":"blocker","summary":"pi blocker test"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Output: BlockerSignal endpoint accessible"
else
  log_fail "Output: BlockerSignal endpoint failed: $resp"
fi

# EvidenceLinkedObservation via ECS store
resp=$(curl -s -X POST "${BASE_URL}/v1/ecs/store" \
  -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"pi-evidence","content_b64":"cGlvdXRwdXQ="}')
if echo "$resp" | grep -q '"id"'; then
  log_pass "Output: EvidenceLinkedObservation (ECS) endpoint accessible"
else
  log_fail "Output: EvidenceLinkedObservation endpoint failed: $resp"
fi

# DecisionCandidate via semantic memory
resp=$(curl -s -X POST "${BASE_URL}/v1/memory/semantic/upsert" \
  -H "Content-Type: application/json" \
  -d '{"key":"pi-decision-candidate","value":"test decision"}')
if echo "$resp" | grep -q '"status"'; then
  log_pass "Output: DecisionCandidate endpoint accessible"
else
  log_fail "Output: DecisionCandidate endpoint failed: $resp"
fi

# ScratchReasoningRecord via scratch file (extension path)
if [ -d "/tmp/pi-scratch" ] || mkdir -p /tmp/pi-scratch 2>/dev/null; then
  log_pass "Output: ScratchReasoningRecord path accessible"
else
  log_fail "Output: ScratchReasoningRecord path failed"
fi

# OntologyActionIntent via commands
resp=$(curl -s -X POST "${BASE_URL}/v1/commands/submit" \
  -H "Content-Type: application/json" \
  -d '{"command":"memory.procedural.reinforce","payload":{"rule_id":"pi-action-test"}}')
if echo "$resp" | grep -q '"command_id"'; then
  log_pass "Output: OntologyActionIntent (commands) endpoint accessible"
else
  log_fail "Output: OntologyActionIntent endpoint failed: $resp"
fi

# VerificationRequest via reflection status
resp=$(curl -s "${BASE_URL}/v1/reflect/status")
if echo "$resp" | grep -q '"enabled"'; then
  log_pass "Output: VerificationRequest (reflection) accessible"
else
  log_fail "Output: VerificationRequest endpoint failed: $resp"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "=== SPEC-52 PI EXTENSION CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All Pi extension contract API contracts verified${NC}"
  exit 0
else
  echo -e "${RED}Some tests failed${NC}"
  exit 1
fi

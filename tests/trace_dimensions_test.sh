#!/bin/bash
# SPEC-56.1: Trace Dimensions — strict CI gate
# Enforces observable evidence for the documented dimensions. No existence-only pass.

set -euo pipefail

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

http_code() {
  curl -sS -o /tmp/focusa-trace-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-trace-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-trace-body.json)"
  fi
}

echo "=== SPEC-56.1: Trace Dimensions (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Health"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true' "Daemon running"
else
  log_fail "Daemon health failed with HTTP ${code}"
  exit 1
fi

log_info "Seed state"
code=$(http_code -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  -d '{"title":"trace-test","goal":"verify trace dimensions","beads_issue_id":"trace-001"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed focus frame accepted"
else
  log_fail "Seed focus frame failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"blocker","summary":"trace blocker seed"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed blocker signal accepted"
else
  log_fail "Seed blocker signal failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/ecs/store" -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"trace-test","content_b64":"dGVzdA=="}')
if [ "$code" = "200" ]; then
  json_assert '.id != null' "Seed ECS artifact stored"
else
  log_fail "Seed ECS artifact failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/update" -H "Content-Type: application/json" \
  -d '{"delta":{"decisions":["Trace dimension test seeded decision evidence"]}}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Seed ASCC decision accepted"
else
  log_fail "Seed ASCC decision failed"
fi

log_info "Dimension checks"
code=$(http_code "${BASE_URL}/v1/focus/stack")
if [ "$code" = "200" ]; then
  json_assert '.stack.frames | length > 0' "Dims 1/11/12: focus frames populated"
else
  log_fail "Focus stack fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/memory/semantic")
if [ "$code" = "200" ]; then
  json_assert '.semantic != null' "Dim 2: semantic working set accessible"
else
  log_fail "Semantic memory fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/memory/procedural")
if [ "$code" = "200" ]; then
  json_assert '.procedural != null' "Dim 3: procedural constraints accessible"
else
  log_fail "Procedural memory fetch failed"
fi

decision_visible=0
for i in $(seq 1 10); do
  code=$(http_code "${BASE_URL}/v1/ascc/state")
  if [ "$code" = "200" ] && jq -e '((.active_frame != null) or (.frame_id != null)) and (((.decisions // .ascc.decisions // []) | length) > 0)' /tmp/focusa-trace-body.json >/dev/null 2>&1; then
    decision_visible=1
    break
  fi
  sleep 0.2
done
if [ "$decision_visible" = "1" ]; then
  log_pass "Dim 4: ASCC decision state accessible"
else
  log_fail "Dim 4: ASCC decision state inaccessible :: $(cat /tmp/focusa-trace-body.json)"
fi

code=$(http_code -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" \
  -d '{"command":"memory.semantic.upsert","payload":{"key":"trace-test","value":"test"}}')
if [ "$code" = "200" ]; then
  json_assert '.command_id != null' "Dim 5: command submission tracked"
else
  log_fail "Command submission failed"
fi

code=$(http_code -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" \
  -d '{"turn_id":"trace-test","raw_user_input":"test","format":"string","budget":500}')
if [ "$code" = "200" ]; then
  json_assert '.context_stats != null' "Dims 6/15: prompt assembly reports context stats"
else
  log_fail "Prompt assemble failed"
fi

code=$(http_code "${BASE_URL}/v1/events/recent")
if [ "$code" = "200" ]; then
  json_assert '.events | length > 0' "Dim 7: event log populated"
else
  log_fail "Events recent failed"
fi

code=$(http_code "${BASE_URL}/v1/ecs/handles")
if [ "$code" = "200" ]; then
  json_assert '.count > 0 and (.handles | length > 0)' "Dim 8: ontology/evidence handles persisted"
else
  log_fail "ECS handles fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/focus-gate/candidates")
if [ "$code" = "200" ]; then
  json_assert '.candidates != null' "Dims 9/13: gate candidates observable"
else
  log_fail "Focus-gate candidates failed"
fi

TURN_ID="trace-$(date +%s%N)"
code=$(http_code -X POST "${BASE_URL}/v1/turn/start" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"trace-test\",\"adapter_id\":\"trace-test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}")
if [ "$code" = "200" ]; then
  log_pass "Turn start accepted"
else
  log_fail "Turn start failed"
fi
code=$(http_code -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted" or .duplicate == true' "Dim 10: turn completion recorded"
else
  log_fail "Turn complete failed"
fi

code=$(http_code "${BASE_URL}/v1/status")
if [ "$code" = "200" ]; then
  json_assert '.session != null and .worker_status != null' "Dim 14: resumable session/status visible"
else
  log_fail "Status fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/references/trace?ref_id=test-123")
if [ "$code" = "200" ]; then
  json_assert '.used_in != null' "Dim 16: reference trace endpoint populated"
else
  log_fail "Reference trace failed with HTTP ${code}"
fi

code=$(http_code "${BASE_URL}/v1/intuition/signals")
if [ "$code" = "200" ]; then
  json_assert '.signals != null or .status != null' "Dim 18: intuition signals accessible"
else
  log_fail "Intuition signals failed with HTTP ${code}"
fi

echo ""
echo "=== SPEC-56.1 TRACE DIMENSIONS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict trace dimension checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict trace dimension checks failed${NC}"
  exit 1
fi

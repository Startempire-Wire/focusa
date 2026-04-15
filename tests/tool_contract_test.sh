#!/bin/bash
# SPEC-55: Tool Action Contracts — strict CI gate

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
  curl -sS -o /tmp/focusa-tool-contract-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-tool-contract-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-tool-contract-body.json)"
  fi
}

echo "=== SPEC-55: Tool Action Contracts (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'type PushDeltaFailureReason = "offline" \| "no_active_frame" \| "validation_rejected" \| "write_failed"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "PushDelta exposes required write failure reasons"
else
  log_fail "PushDelta failure reasons missing required contract taxonomy"
fi

if rg -n 'response\.status === "no_active_frame"|response\.status === "rejected"|response\.status !== "accepted"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "PushDelta inspects write status envelope before reporting success"
else
  log_fail "PushDelta does not inspect write status envelope faithfully"
fi

if rg -n 'mirrorFailedFocusWrite\("decision"|mirrorFailedFocusWrite\("constraint"|mirrorFailedFocusWrite\("failure"' "$TOOLS_TS" >/dev/null 2>&1; then
  log_pass "Operator-critical write tools mirror unrecoverable failures to scratchpad"
else
  log_fail "Operator-critical write fallback mirroring missing"
fi

log_info "Health"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true and (.version | type == "string")' "Daemon health/version schema"
else
  log_fail "Daemon health failed with HTTP ${code}"
  exit 1
fi

log_info "Input schema validation"
code=$(http_code -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  -d '{"title":"tool-contract-test","goal":"verify contract","beads_issue_id":"tc-001"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Valid focus push accepted"
else
  log_fail "Valid focus push returned HTTP ${code}"
fi

code=$(http_code -X POST "${BASE_URL}/v1/focus/set-active" -H "Content-Type: application/json" \
  -d '{"frame_id":"not-a-uuid"}')
if [ "$code" = "422" ]; then
  log_pass "Invalid UUID rejected with HTTP 422"
else
  log_fail "Invalid UUID expected HTTP 422, got ${code} :: $(cat /tmp/focusa-tool-contract-body.json)"
fi

log_info "Failure modes"
code=$(http_code -X POST "${BASE_URL}/v1/nonexistent" -H "Content-Type: application/json" -d '{}')
if [ "$code" = "404" ]; then
  log_pass "Unknown route rejected with HTTP 404"
else
  log_fail "Unknown route expected HTTP 404, got ${code}"
fi

code=$(http_code -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" -d '{"turn_id":"bad"}')
if [ "$code" = "422" ]; then
  log_pass "Bad prompt payload rejected with HTTP 422"
else
  log_fail "Bad prompt payload expected HTTP 422, got ${code} :: $(cat /tmp/focusa-tool-contract-body.json)"
fi

log_info "Idempotency — strict"
TURN_ID="idem-test-$(date +%s%N)"
code=$(http_code -X POST "${BASE_URL}/v1/turn/start" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"harness_name\":\"test\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-11T00:00:00Z\"}")
if [ "$code" = "200" ]; then
  log_pass "Turn start accepted for idempotency test"
else
  log_fail "Turn start failed for idempotency test"
fi

code=$(http_code -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
if [ "$code" = "200" ]; then
  log_pass "First turn complete accepted"
else
  log_fail "First turn complete failed"
fi

duplicate_seen=0
for _ in 1 2 3 4 5; do
  sleep 0.3
  code=$(http_code -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" \
    -d "{\"turn_id\":\"${TURN_ID}\",\"assistant_output\":\"done\",\"artifacts\":[],\"errors\":[]}")
  if [ "$code" = "200" ] && jq -e '.duplicate == true' /tmp/focusa-tool-contract-body.json >/dev/null 2>&1; then
    duplicate_seen=1
    break
  fi
done
if [ "$duplicate_seen" = "1" ]; then
  log_pass "Turn complete duplicate flagged explicitly"
else
  log_fail "Idempotency duplicate flag missing :: $(cat /tmp/focusa-tool-contract-body.json)"
fi

log_info "Observable side effects"
code=$(http_code -X POST "${BASE_URL}/v1/memory/semantic/upsert" -H "Content-Type: application/json" \
  -d '{"key":"tool-contract-test","value":"testing"}')
if [ "$code" = "200" ]; then
  log_pass "Semantic upsert accepted"
else
  log_fail "Semantic upsert failed"
fi
sleep 1
code=$(http_code "${BASE_URL}/v1/memory/semantic")
if [ "$code" = "200" ]; then
  json_assert '.semantic != null' "Semantic memory observable after upsert"
else
  log_fail "Semantic memory fetch failed"
fi

log_info "Timeout policy / degraded fallback"
code=$(http_code "${BASE_URL}/v1/status")
if [ "$code" = "200" ]; then
  json_assert '.worker_status.queue_size_config != null and .worker_status.job_timeout_ms != null' "Worker timeout policy visible"
else
  log_fail "Status fetch failed"
fi

code=$(http_code "${BASE_URL}/v1/reflect/status")
if [ "$code" = "200" ]; then
  json_assert '.enabled != null' "Reflection degraded fallback status visible"
else
  log_fail "Reflect status fetch failed"
fi

log_info "Action contract matrix"
code=$(http_code "${BASE_URL}/v1/ontology/contracts")
if [ "$code" = "200" ]; then
  json_assert '.contracts | length >= 10' "Ontology action contract catalog exposed"
  json_assert '.contracts | any(.name == "refactor_module" and .input_schema.required != null and .failure_modes != null and .tool_mappings != null)' "Refactor contract exposes schema/failure/tool mappings"
  json_assert '.contracts | any(.name == "modify_schema" and .rollback_availability.available == true and .timeout_policy.job_timeout_ms_field == "worker_status.job_timeout_ms")' "Modify-schema contract exposes rollback + timeout policy"
  json_assert '.contracts | any(.name == "mark_blocked" and (.failure_modes | index("dependency_failure")) and (.verification_hooks | length) >= 1)' "Blocker contract exposes failure/verification semantics"
else
  log_fail "Ontology contracts fetch failed"
fi

echo ""
echo "=== SPEC-55 TOOL ACTION CONTRACTS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict tool contract checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict tool contract checks failed${NC}"
  exit 1
fi

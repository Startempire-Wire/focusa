#!/bin/bash
# SPEC-56.2: Checkpoint Triggers + Resume Semantics — strict CI gate

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
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
  curl -sS -o /tmp/focusa-checkpoint-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-checkpoint-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-checkpoint-body.json)"
  fi
}

echo "=== SPEC-56.2: Checkpoint Triggers + Resume Semantics (strict) ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Health"
code=$(http_code "${BASE_URL}/v1/health")
if [ "$code" = "200" ]; then
  json_assert '.ok == true' "Daemon running"
else
  log_fail "Daemon not responding"
  exit 1
fi

log_info "Checkpoint infrastructure"
code=$(http_code "${BASE_URL}/v1/state/dump")
if [ "$code" = "200" ]; then
  json_assert '.focus_stack != null and .focus_gate != null' "State dump exposes checkpoint-critical state"
else
  log_fail "State dump failed"
fi

log_info "Trigger 1: session start"
# Close any prior active session so SessionStarted is accepted by the reducer.
# (A prior CI run may have left a session alive in the daemon's session
# store; the reducer rejects "Invalid event for current state:
# SessionStarted but an active session already exists".)
http_code -X POST "${BASE_URL}/v1/session/close" -H "Content-Type: application/json" \
  -d '{"reason":"checkpoint-trigger-test-reset"}' >/dev/null 2>&1 || true
WS_ID="checkpoint-test-$(date +%s)-$$"
code=$(http_code -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" \
  -d "{\"workspace_id\":\"${WS_ID}\",\"continuity_id\":\"${WS_ID}\",\"project_root\":\"${ROOT_DIR}\",\"adapter_id\":\"pi\"}")
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Session start accepted"
else
  log_fail "Session start failed"
fi

code=$(http_code "${BASE_URL}/v1/status")
if [ "$code" = "200" ]; then
  json_assert '.session != null and .worker_status != null' "Session state observable after start"
else
  log_fail "Status after start failed"
fi

log_info "Trigger 3: high-impact action completion"
code=$(http_code -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  -d "{\"title\":\"checkpoint-test\",\"goal\":\"testing triggers\",\"beads_issue_id\":\"focusa-032h\",\"project_root\":\"${ROOT_DIR}\",\"continuity_id\":\"checkpoint-test\"}")
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Focus push accepted"
else
  log_fail "Focus push failed"
fi

code=$(http_code "${BASE_URL}/v1/focus/stack")
if [ "$code" = "200" ]; then
  json_assert '.stack.frames | length > 0' "Focus push materialized in stack"
else
  log_fail "Focus stack fetch failed"
fi

log_info "Trigger 5: blocker/failure emergence"
code=$(http_code -X POST "${BASE_URL}/v1/focus-gate/ingest-signal" -H "Content-Type: application/json" \
  -d '{"kind":"blocker","summary":"checkpoint test blocker"}')
if [ "$code" = "200" ]; then
  json_assert '.status == "accepted"' "Blocker signal accepted"
else
  log_fail "Blocker signal failed"
fi
sleep 1
code=$(http_code "${BASE_URL}/v1/focus-gate/candidates")
if [ "$code" = "200" ]; then
  json_assert '.candidates != null' "Gate candidates observable after blocker"
else
  log_fail "Gate candidates failed"
fi

log_info "Trigger 6: explicit resume/fork points"
SESSION_ID=$(curl -sS "${BASE_URL}/v1/status" | jq -r '.session.id // .session.session_id // empty')
if [ -n "$SESSION_ID" ]; then
  code=$(http_code -X POST "${BASE_URL}/v1/session/resume" -H "Content-Type: application/json" \
    -d "{\"session_id\":\"${SESSION_ID}\"}")
  if [ "$code" = "200" ]; then
    json_assert '.status == "accepted"' "Session resume accepted for explicit resume point"
  else
    log_fail "Session resume failed with HTTP ${code}"
  fi
else
  log_fail "No session id exposed in /v1/status for resume semantics"
fi

log_info "Resume semantics"
code=$(http_code "${BASE_URL}/v1/state/dump")
if [ "$code" = "200" ]; then
  json_assert '.focus_stack != null and .memory != null and .focus_gate != null' "State dump contains mission, working set, blockers"
else
  log_fail "State dump resume verification failed"
fi

code=$(http_code "${BASE_URL}/v1/focus/stack")
if [ "$code" = "200" ]; then
  json_assert '.stack != null and .active_frame_id != null' "Focus stack exposes active frame for resume"
else
  log_fail "Focus stack resume verification failed"
fi

missing_ascc=$(curl -sS "${BASE_URL}/v1/ascc/frame/00000000-0000-0000-0000-000000000000")
if echo "$missing_ascc" | jq -e '.status == "blocked" and .failure_class == "frame_unavailable" and .why != null and .recovery_hint != null and .misuse_hint != null and .details.tool_result_v1.ok == false' >/dev/null 2>&1; then
  log_pass "ASCC missing frame exposes no-guess recovery contract"
else
  log_fail "ASCC missing frame missing no-guess recovery contract :: $missing_ascc"
fi

echo ""
echo "=== SPEC-56.2 CHECKPOINT TRIGGERS RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}All strict checkpoint/resume checks passed${NC}"
  exit 0
else
  echo -e "${RED}Strict checkpoint/resume checks failed${NC}"
  exit 1
fi

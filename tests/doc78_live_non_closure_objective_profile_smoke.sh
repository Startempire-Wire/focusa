#!/bin/bash
# Doc78 holistic runtime smoke:
# - starts isolated daemon
# - emits verification_result traces for multiple loop objectives
# - validates non-closure objective profiling in status + closure-bundle surfaces
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/debug/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  DAEMON_BIN="${REPO_ROOT}/target/release/focusa-daemon"
fi

BASE_URL="${FOCUSA_DOC78_OBJECTIVE_BASE_URL:-http://127.0.0.1:18881}"
BIND_ADDR="${FOCUSA_DOC78_OBJECTIVE_BIND:-127.0.0.1:18881}"
DATA_DIR="${FOCUSA_DOC78_OBJECTIVE_DATA_DIR:-$(mktemp -d /tmp/focusa-doc78-objectives.XXXXXX)}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info(){ echo -e "${YELLOW}INFO${NC}: $1"; }

start_daemon() {
  FOCUSA_BASE_URL="$BASE_URL" FOCUSA_BIND="$BIND_ADDR" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-doc78-objective-smoke.log 2>&1 &
  DAEMON_PID=$!
  for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
    if curl -fsS "${BASE_URL}/v1/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.2
  done
  return 1
}

stop_daemon() {
  if [ -n "${DAEMON_PID:-}" ]; then
    kill "$DAEMON_PID" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" 2>/dev/null || true
    unset DAEMON_PID
  fi
}

cleanup() {
  stop_daemon
  rm -rf "$DATA_DIR" >/dev/null 2>&1 || true
}
trap cleanup EXIT

http_json(){ curl -sS "$@"; }

emit_quality_trace(){
  local objective="$1"
  local continuation="$2"
  local trace_turn="$3"
  http_json -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H 'Content-Type: application/json' \
    -d "{\"event_type\":\"verification_result\",\"verification_kind\":\"secondary_loop_quality\",\"loop_objective\":\"${objective}\",\"continuation_decision\":\"${continuation}\",\"turn_id\":\"${trace_turn}\",\"quality\":\"synthetic-runtime-smoke\"}"
}

echo "=== DOC78 LIVE NON-CLOSURE OBJECTIVE PROFILE SMOKE ==="
echo "Daemon: ${DAEMON_BIN}"
echo "Base URL: ${BASE_URL}"
echo "Data dir: ${DATA_DIR}"
echo ""

if [ ! -x "$DAEMON_BIN" ]; then
  log_fail "focusa-daemon binary not executable: $DAEMON_BIN"
  exit 1
fi

log_info "Start isolated daemon"
if start_daemon; then
  log_pass "isolated daemon started"
else
  log_fail "isolated daemon failed to start"
  exit 1
fi

HEALTH=$(http_json "${BASE_URL}/v1/health")
if echo "$HEALTH" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "health route is ready"
else
  log_fail "health route not ready: $HEALTH"
fi

SESSION=$(http_json -X POST "${BASE_URL}/v1/session/start" -H 'Content-Type: application/json' -d '{"workspace_id":"doc78-objective-smoke"}')
if echo "$SESSION" | jq -e '.ok == true or .status == "ok" or .status == "started" or .status == "accepted"' >/dev/null 2>&1; then
  log_pass "session started"
else
  log_fail "session start did not return success markers: $SESSION"
fi

TRACE_A=$(emit_quality_trace "continuous_turn_outcome_quality" "continue" "doc78-objective-1")
if echo "$TRACE_A" | jq -e '.status == "recorded" and .event_type == "verification_result"' >/dev/null 2>&1; then
  log_pass "baseline objective trace recorded"
else
  log_fail "baseline objective trace failed to record: $TRACE_A"
fi

TRACE_B=$(emit_quality_trace "governance_pause_resolution" "defer" "doc78-objective-2")
if echo "$TRACE_B" | jq -e '.status == "recorded" and .event_type == "verification_result"' >/dev/null 2>&1; then
  log_pass "non-closure governance objective trace recorded"
else
  log_fail "governance objective trace failed to record: $TRACE_B"
fi

TRACE_C=$(emit_quality_trace "continuity_handoff_validation" "continue" "doc78-objective-3")
if echo "$TRACE_C" | jq -e '.status == "recorded" and .event_type == "verification_result"' >/dev/null 2>&1; then
  log_pass "non-closure continuity objective trace recorded"
else
  log_fail "continuity objective trace failed to record: $TRACE_C"
fi

TRACES=$(http_json "${BASE_URL}/v1/telemetry/trace?event_type=verification_result&limit=20")
if echo "$TRACES" | jq -e '.count >= 3' >/dev/null 2>&1; then
  log_pass "trace endpoint reports recorded verification_result events"
else
  log_fail "trace endpoint missing expected verification_result events: $TRACES"
fi

STATUS=$(http_json "${BASE_URL}/v1/work-loop/status")
if echo "$STATUS" | jq -e '.secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events >= 3 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events >= 2 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.objective_counts.governance_pause_resolution >= 1 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.objective_counts.continuity_handoff_validation >= 1 and (.secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_rate // 0) > 0' >/dev/null 2>&1; then
  log_pass "status objective profile reflects non-closure runtime traces"
else
  log_fail "status objective profile missing non-closure trace evidence: $STATUS"
fi

BUNDLE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-bundle")
if echo "$BUNDLE" | jq -e '.status == "ok" and .doc == "78" and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events >= 3 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events >= 2 and (.secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_rate // 0) > 0 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.objective_counts.governance_pause_resolution >= 1' >/dev/null 2>&1; then
  log_pass "closure-bundle objective profile reflects non-closure runtime traces"
else
  log_fail "closure-bundle objective profile missing non-closure trace evidence: $BUNDLE"
fi

echo ""
echo "=== DOC78 LIVE NON-CLOSURE OBJECTIVE PROFILE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

#!/bin/bash
# Doc78 holistic runtime smoke:
# - starts isolated daemon
# - verifies closure-bundle-first consumer contract at runtime
# - verifies objective-profile evidence projection is present
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/debug/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  DAEMON_BIN="${REPO_ROOT}/target/release/focusa-daemon"
fi

BASE_URL="${FOCUSA_DOC78_BASE_URL:-http://127.0.0.1:18879}"
BIND_ADDR="${FOCUSA_DOC78_BIND:-127.0.0.1:18879}"
DATA_DIR="${FOCUSA_DOC78_DATA_DIR:-$(mktemp -d /tmp/focusa-doc78-live.XXXXXX)}"
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
  FOCUSA_BASE_URL="$BASE_URL" FOCUSA_BIND="$BIND_ADDR" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-doc78-live-smoke.log 2>&1 &
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

echo "=== DOC78 LIVE RUNTIME CLOSURE BUNDLE SMOKE ==="
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

SESSION=$(http_json -X POST "${BASE_URL}/v1/session/start" -H 'Content-Type: application/json' -d '{"workspace_id":"doc78-live-smoke"}')
if echo "$SESSION" | jq -e '.ok == true or .status == "ok" or .status == "started" or .status == "accepted"' >/dev/null 2>&1; then
  log_pass "session started"
else
  log_fail "session start did not return success markers: $SESSION"
fi

STATUS=$(http_json "${BASE_URL}/v1/work-loop/status")
if echo "$STATUS" | jq -e '.status != null and .secondary_loop_replay_consumer.status != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events != null' >/dev/null 2>&1; then
  log_pass "status projects replay consumer, continuity gate, and objective profile"
else
  log_fail "status missing closure-bundle readiness fields: $STATUS"
fi

BUNDLE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-bundle")
if echo "$BUNDLE" | jq -e '.status == "ok" and .doc == "78" and .secondary_loop_replay_consumer.status != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events != null and (.secondary_loop_eval_bundle.secondary_loop_objective_profile | has("non_closure_objective_rate"))' >/dev/null 2>&1; then
  log_pass "closure-bundle route projects replay gate + objective-profile evidence"
else
  log_fail "closure-bundle route missing expected fields: $BUNDLE"
fi

EVIDENCE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-evidence")
if echo "$EVIDENCE" | jq -e '.status != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_replay_comparative.status != null and .secondary_loop_closure_replay_evidence.status != null' >/dev/null 2>&1; then
  log_pass "closure-evidence fallback route carries continuity gate projection"
else
  log_fail "closure-evidence route missing expected fields: $EVIDENCE"
fi

STATUS_FOR_CTX=$(http_json "${BASE_URL}/v1/work-loop/status")
CTX_WRITER=$(echo "$STATUS_FOR_CTX" | jq -r '.active_writer // empty')
if [ -n "$CTX_WRITER" ]; then
  CTX=$(http_json -X POST "${BASE_URL}/v1/work-loop/context" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${CTX_WRITER}" -d '{"current_ask":"doc78 live bundle smoke","ask_kind":"instruction","scope_kind":"mission_carryover","carryover_policy":"allow_if_relevant","excluded_context_reason":"runtime-smoke","excluded_context_labels":["doc78","live"],"source_turn_id":"doc78-live-closure-bundle","operator_steering_detected":true}')
else
  CTX=$(http_json -X POST "${BASE_URL}/v1/work-loop/context" -H 'Content-Type: application/json' -d '{"current_ask":"doc78 live bundle smoke","ask_kind":"instruction","scope_kind":"mission_carryover","carryover_policy":"allow_if_relevant","excluded_context_reason":"runtime-smoke","excluded_context_labels":["doc78","live"],"source_turn_id":"doc78-live-closure-bundle","operator_steering_detected":true}')
fi
if ! echo "$CTX" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  CTX_RETRY_WRITER=$(http_json "${BASE_URL}/v1/work-loop/status" | jq -r '.active_writer // empty')
  if [ -n "$CTX_RETRY_WRITER" ]; then
    CTX=$(http_json -X POST "${BASE_URL}/v1/work-loop/context" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${CTX_RETRY_WRITER}" -d '{"current_ask":"doc78 live bundle smoke","ask_kind":"instruction","scope_kind":"mission_carryover","carryover_policy":"allow_if_relevant","excluded_context_reason":"runtime-smoke","excluded_context_labels":["doc78","live"],"source_turn_id":"doc78-live-closure-bundle","operator_steering_detected":true}')
  fi
fi
if echo "$CTX" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "context update accepted"
else
  log_fail "context update rejected: $CTX"
fi

STATUS2=$(http_json "${BASE_URL}/v1/work-loop/status")
if echo "$STATUS2" | jq -e '.decision_context.current_ask == "doc78 live bundle smoke" and .secondary_loop_continuity_gate.state != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events != null' >/dev/null 2>&1; then
  log_pass "status reflects context + closure-bundle projections after update"
else
  log_fail "status missing post-context closure projections: $STATUS2"
fi

echo ""
echo "=== DOC78 LIVE RUNTIME CLOSURE BUNDLE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

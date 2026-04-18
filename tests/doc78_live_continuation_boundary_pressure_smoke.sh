#!/bin/bash
# Doc78 holistic runtime smoke:
# - starts isolated daemon
# - validates continuation-boundary behavior under operator steering + governance pressure
# - proves sustained select-next pressure path pauses/blocks before autonomous handoff
# - proves trace evidence accumulates continuation-boundary failures across rounds
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/debug/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  DAEMON_BIN="${REPO_ROOT}/target/release/focusa-daemon"
fi

BASE_URL="${FOCUSA_DOC78_BOUNDARY_BASE_URL:-http://127.0.0.1:18880}"
BIND_ADDR="${FOCUSA_DOC78_BOUNDARY_BIND:-127.0.0.1:18880}"
DATA_DIR="${FOCUSA_DOC78_BOUNDARY_DATA_DIR:-$(mktemp -d /tmp/focusa-doc78-boundary.XXXXXX)}"
WRITER_ID="${FOCUSA_DOC78_BOUNDARY_WRITER:-doc78-live-boundary-smoke}"
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
  FOCUSA_BASE_URL="$BASE_URL" FOCUSA_BIND="$BIND_ADDR" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-doc78-boundary-smoke.log 2>&1 &
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

resolve_writer_id(){
  local active_writer
  active_writer=$(http_json "${BASE_URL}/v1/work-loop/status" | jq -r '.active_writer // empty')
  if [ -n "$active_writer" ]; then
    echo "$active_writer"
  else
    echo "$WRITER_ID"
  fi
}

write_json(){
  local url="$1"
  shift
  local writer_id
  writer_id=$(resolve_writer_id)
  http_json -X POST "$url" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${writer_id}" "$@"
}

echo "=== DOC78 LIVE CONTINUATION BOUNDARY PRESSURE SMOKE ==="
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

SESSION=$(http_json -X POST "${BASE_URL}/v1/session/start" -H 'Content-Type: application/json' -d '{"workspace_id":"doc78-boundary-smoke"}')
if echo "$SESSION" | jq -e '.ok == true or .status == "ok" or .status == "started" or .status == "accepted"' >/dev/null 2>&1; then
  log_pass "session started"
else
  log_fail "session start did not return success markers: $SESSION"
fi

ENABLE_WRITER=$(resolve_writer_id)
ENABLE=$(http_json -X POST "${BASE_URL}/v1/work-loop/enable" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${ENABLE_WRITER}" -H 'x-focusa-approval: approved' -d '{"preset":"balanced"}')
if ! echo "$ENABLE" | jq -e '.ok == true' >/dev/null 2>&1; then
  ENABLE_RETRY_WRITER=$(resolve_writer_id)
  ENABLE=$(http_json -X POST "${BASE_URL}/v1/work-loop/enable" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${ENABLE_RETRY_WRITER}" -H 'x-focusa-approval: approved' -d '{"preset":"balanced"}')
fi
if echo "$ENABLE" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "continuous loop enable accepted"
else
  log_fail "continuous loop enable rejected: $ENABLE"
fi

CTX_OPERATOR=$(write_json "${BASE_URL}/v1/work-loop/context" -d '{"current_ask":"doc78 operator boundary pressure","operator_steering_detected":true,"source_turn_id":"doc78-boundary-operator"}')
if echo "$CTX_OPERATOR" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "operator-steering context update accepted"
else
  log_fail "operator-steering context update rejected: $CTX_OPERATOR"
fi

SEL_OPERATOR=$(write_json "${BASE_URL}/v1/work-loop/select-next" -d '{"parent_work_item_id":"focusa-o8vn"}')
if echo "$SEL_OPERATOR" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "select-next request accepted while operator boundary active"
else
  log_fail "select-next request rejected unexpectedly under operator boundary: $SEL_OPERATOR"
fi
sleep 0.2

STATUS_OPERATOR=""
operator_ok=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  STATUS_OPERATOR=$(http_json "${BASE_URL}/v1/work-loop/status")
  if echo "$STATUS_OPERATOR" | jq -e '.status == "paused" and .decision_context.operator_steering_detected == true and .last_continue_reason == "checkpoint: paused select-next for operator-priority boundary" and .last_blocker_reason == null' >/dev/null 2>&1; then
    operator_ok=1
    break
  fi
  sleep 0.2
done
if [ "$operator_ok" -eq 1 ]; then
  log_pass "operator boundary pauses select-next pressure path with checkpoint evidence"
else
  log_fail "operator boundary pause evidence missing: $STATUS_OPERATOR"
fi

CTX_CLEAR_OPERATOR=$(write_json "${BASE_URL}/v1/work-loop/context" -d '{"operator_steering_detected":false,"source_turn_id":"doc78-boundary-clear-operator"}')
if echo "$CTX_CLEAR_OPERATOR" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "operator steering cleared"
else
  log_fail "operator steering clear rejected: $CTX_CLEAR_OPERATOR"
fi

PAUSE_GOV=$(write_json "${BASE_URL}/v1/work-loop/pause-flags" -d '{"destructive_confirmation_required":false,"governance_decision_pending":true,"operator_override_active":false,"reason":"doc78 governance pressure smoke"}')
if echo "$PAUSE_GOV" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "governance pause flag accepted"
else
  log_fail "governance pause flag rejected: $PAUSE_GOV"
fi

SEL_GOV=$(write_json "${BASE_URL}/v1/work-loop/select-next" -d '{"parent_work_item_id":"focusa-o8vn"}')
if echo "$SEL_GOV" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "select-next request accepted while governance boundary active"
else
  log_fail "select-next request rejected unexpectedly under governance boundary: $SEL_GOV"
fi
sleep 0.2

STATUS_GOV=""
governance_ok=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  STATUS_GOV=$(http_json "${BASE_URL}/v1/work-loop/status")
  if echo "$STATUS_GOV" | jq -e '.status == "blocked" and .pause_flags.governance_decision_pending == true and .last_blocker_reason == "governance decision pending" and .last_blocker_class == "governance" and .last_continue_reason == "checkpoint: blocked select-next on continuation boundary"' >/dev/null 2>&1; then
    governance_ok=1
    break
  fi
  sleep 0.2
done
if [ "$governance_ok" -eq 1 ]; then
  log_pass "governance boundary blocks select-next pressure path with blocker evidence"
else
  log_fail "governance boundary block evidence missing: $STATUS_GOV"
fi

HEARTBEAT_GOV=$(write_json "${BASE_URL}/v1/work-loop/heartbeat" -d '{}')
if echo "$HEARTBEAT_GOV" | jq -e '.ok == true and .dispatched == false' >/dev/null 2>&1; then
  log_pass "heartbeat does not dispatch autonomous turn while governance boundary is active"
else
  log_fail "heartbeat unexpectedly dispatched during governance boundary: $HEARTBEAT_GOV"
fi

PAUSE_GOV2=$(write_json "${BASE_URL}/v1/work-loop/pause-flags" -d '{"destructive_confirmation_required":false,"governance_decision_pending":true,"operator_override_active":false,"reason":"doc78 governance pressure smoke round2"}')
if echo "$PAUSE_GOV2" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "governance pause flag accepted for sustained round"
else
  log_fail "governance pause flag rejected for sustained round: $PAUSE_GOV2"
fi

SEL_GOV2=$(write_json "${BASE_URL}/v1/work-loop/select-next" -d '{"parent_work_item_id":"focusa-o8vn"}')
if echo "$SEL_GOV2" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "select-next request accepted during sustained governance pressure"
else
  log_fail "select-next request rejected during sustained governance pressure: $SEL_GOV2"
fi
sleep 0.2

STATUS_GOV2=""
governance_round2_ok=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  STATUS_GOV2=$(http_json "${BASE_URL}/v1/work-loop/status")
  if echo "$STATUS_GOV2" | jq -e '.status == "blocked" and .pause_flags.governance_decision_pending == true and .last_blocker_reason == "governance decision pending" and .last_blocker_class == "governance" and .last_continue_reason == "checkpoint: blocked select-next on continuation boundary"' >/dev/null 2>&1; then
    governance_round2_ok=1
    break
  fi
  sleep 0.2
done
if [ "$governance_round2_ok" -eq 1 ]; then
  log_pass "sustained governance pressure keeps select-next path blocked"
else
  log_fail "sustained governance pressure evidence missing: $STATUS_GOV2"
fi

HEARTBEAT_GOV2=$(write_json "${BASE_URL}/v1/work-loop/heartbeat" -d '{}')
if echo "$HEARTBEAT_GOV2" | jq -e '.ok == true and .dispatched == false' >/dev/null 2>&1; then
  log_pass "heartbeat remains suppressed during sustained governance pressure"
else
  log_fail "heartbeat unexpectedly dispatched in sustained governance pressure: $HEARTBEAT_GOV2"
fi

TRACE_FAILURES=$(http_json "${BASE_URL}/v1/telemetry/trace?event_type=scope_failure_recorded&limit=200")
if echo "$TRACE_FAILURES" | jq -e '.count >= 3 and ([.events[].payload.reason] | any(. == "operator steering detected")) and ([.events[].payload.reason] | any(. == "governance decision pending")) and ([.events[].payload.path] | any(. == "select_next_continuous_subtask"))' >/dev/null 2>&1; then
  log_pass "trace stream records sustained continuation-boundary failures across operator and governance reasons"
else
  log_fail "trace stream missing sustained continuation-boundary failure evidence: $TRACE_FAILURES"
fi

PAUSE_CLEAR=$(write_json "${BASE_URL}/v1/work-loop/pause-flags" -d '{"destructive_confirmation_required":false,"governance_decision_pending":false,"operator_override_active":false,"reason":"doc78 governance pressure clear"}')
if echo "$PAUSE_CLEAR" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "governance pause flag cleared"
else
  log_fail "governance pause clear rejected: $PAUSE_CLEAR"
fi

BUNDLE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-bundle")
if echo "$BUNDLE" | jq -e '.status == "ok" and .doc == "78" and .secondary_loop_continuity_gate.state != null and .secondary_loop_replay_consumer.status != null' >/dev/null 2>&1; then
  log_pass "closure-bundle remains available during boundary-pressure run"
else
  log_fail "closure-bundle unavailable during boundary-pressure run: $BUNDLE"
fi

echo ""
echo "=== DOC78 LIVE CONTINUATION BOUNDARY PRESSURE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

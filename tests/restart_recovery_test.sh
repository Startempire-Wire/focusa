#!/bin/bash
# SPEC 56: restart / pre-shutdown recovery breadth
# Verifies frame + ASCC state survive daemon shutdown and restart.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/release/focusa-daemon}"
BASE_URL="${FOCUSA_RECOVERY_BASE_URL:-http://127.0.0.1:18796}"
BIND_ADDR="${FOCUSA_RECOVERY_BIND:-127.0.0.1:18796}"
DATA_DIR="${FOCUSA_RECOVERY_DATA_DIR:-$(mktemp -d /tmp/focusa-recovery.XXXXXX)}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

start_daemon() {
  FOCUSA_BASE_URL="$BASE_URL" FOCUSA_BIND="$BIND_ADDR" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-restart-recovery.log 2>&1 &
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

echo "=== SPEC 56: Restart / Pre-shutdown Recovery ==="
echo "Base URL: ${BASE_URL}"
echo "Data dir: ${DATA_DIR}"
echo ""

log_info "Start isolated daemon"
if start_daemon; then
  log_pass "Daemon started"
else
  log_fail "Daemon failed to start"
  exit 1
fi

log_info "Seed session + frame + checkpoint data"
curl -sS -X POST "${BASE_URL}/v1/session/start" \
  -H "Content-Type: application/json" \
  -d '{"workspace_id":"recovery-test"}' >/dev/null
session_ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  if curl -sS "${BASE_URL}/v1/status" | jq -e '.session != null' >/dev/null 2>&1; then
    session_ready=1
    break
  fi
  sleep 0.2
done
if [ "$session_ready" = "1" ]; then
  log_pass "Session materialized before recovery seed"
else
  log_fail "Session did not materialize before recovery seed"
fi
push_resp=$(curl -sS -X POST "${BASE_URL}/v1/focus/push" \
  -H "Content-Type: application/json" \
  -d '{"title":"restart-recovery","goal":"verify restart continuity","beads_issue_id":"recovery-001"}')
frame_id=""
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  frame_id=$(curl -sS "${BASE_URL}/v1/focus/stack" | jq -r '.active_frame_id // empty')
  if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
    break
  fi
  sleep 0.2
done
if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
  log_pass "Focus frame created"
else
  log_fail "Focus frame id missing :: push=$push_resp stack=$(curl -sS "${BASE_URL}/v1/focus/stack")"
fi

if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
  update_resp=$(curl -sS -X POST "${BASE_URL}/v1/ascc/update-delta" \
    -H "Content-Type: application/json" \
    -d "{\"frame_id\":\"${frame_id}\",\"delta\":{\"decisions\":[\"restart-safe decision\"],\"constraints\":[\"restart-safe constraint\"],\"failures\":[\"restart-safe failure\"],\"recent_results\":[\"restart-safe result\"]}}")
  if echo "$update_resp" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
    log_pass "Checkpoint delta accepted"
  else
    log_fail "Checkpoint delta rejected :: $update_resp"
  fi

  seen=0
  for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
    if curl -sS "${BASE_URL}/v1/ascc/frame/${frame_id}" | jq -e '.focus_state.decisions | index("restart-safe decision")' >/dev/null 2>&1; then
      seen=1
      break
    fi
    sleep 0.2
  done
  if [ "$seen" = "1" ]; then
    log_pass "ASCC/frame state materialized before restart"
  else
    log_fail "ASCC/frame state did not materialize before restart"
  fi

  if [ -f "${DATA_DIR}/ascc/${frame_id}.json" ]; then
    log_pass "Checkpoint file persisted before shutdown"
  else
    log_fail "Checkpoint file missing before shutdown"
  fi
fi

log_info "Close session and stop daemon"
close_resp=$(curl -sS -X POST "${BASE_URL}/v1/session/close" \
  -H "Content-Type: application/json" \
  -d '{"reason":"restart-recovery-test"}')
if echo "$close_resp" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "Session close accepted"
else
  log_fail "Session close rejected :: $close_resp"
fi
stop_daemon
log_pass "Daemon stopped"

log_info "Restart daemon with same data dir"
if start_daemon; then
  log_pass "Daemon restarted"
else
  log_fail "Daemon failed to restart"
  exit 1
fi

if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
  restored=$(curl -sS "${BASE_URL}/v1/ascc/frame/${frame_id}")
  if echo "$restored" | jq -e '.focus_state.decisions | index("restart-safe decision")' >/dev/null 2>&1; then
    log_pass "Decision restored after restart"
  else
    log_fail "Decision missing after restart :: $restored"
  fi
  if echo "$restored" | jq -e '.focus_state.constraints | index("restart-safe constraint")' >/dev/null 2>&1; then
    log_pass "Constraint restored after restart"
  else
    log_fail "Constraint missing after restart :: $restored"
  fi
  if echo "$restored" | jq -e '.focus_state.failures | index("restart-safe failure")' >/dev/null 2>&1; then
    log_pass "Failure restored after restart"
  else
    log_fail "Failure missing after restart :: $restored"
  fi
  if echo "$restored" | jq -e '.focus_state.recent_results | index("restart-safe result")' >/dev/null 2>&1; then
    log_pass "Recent result restored after restart"
  else
    log_fail "Recent result missing after restart :: $restored"
  fi
fi

stack_resp=$(curl -sS "${BASE_URL}/v1/focus/stack")
if echo "$stack_resp" | jq -e '.stack.frames | length > 0' >/dev/null 2>&1; then
  log_pass "Focus stack restored after restart"
else
  log_fail "Focus stack missing after restart :: $stack_resp"
fi

echo ""
echo "=== SPEC 56 RESTART / PRE-SHUTDOWN RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Restart / pre-shutdown recovery verified${NC}"
  exit 0
else
  echo -e "${RED}Restart / pre-shutdown recovery failed${NC}"
  exit 1
fi

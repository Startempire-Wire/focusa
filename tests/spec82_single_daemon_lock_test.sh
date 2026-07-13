#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT_DIR}/target/debug/focusa-daemon"
DATA_DIR="$(mktemp -d /tmp/spec82-daemon-lock.XXXXXX)"
PORT1="18791"
PORT2="18792"
LOG1="/tmp/spec82-daemon-lock-1.log"
LOG2="/tmp/spec82-daemon-lock-2.log"
LOG3="/tmp/spec82-daemon-lock-3.log"
PID1=""
PID3=""

cleanup() {
  if [[ -n "$PID1" ]] && kill -0 "$PID1" 2>/dev/null; then
    kill "$PID1" 2>/dev/null || true
    wait "$PID1" 2>/dev/null || true
  fi
  if [[ -n "$PID3" ]] && kill -0 "$PID3" 2>/dev/null; then
    kill "$PID3" 2>/dev/null || true
    wait "$PID3" 2>/dev/null || true
  fi
  rm -rf "$DATA_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"
cargo build -p focusa-api --bin focusa-daemon >/dev/null

FOCUSA_BIND="127.0.0.1:${PORT1}" FOCUSA_DATA_DIR="$DATA_DIR" "$BIN" >"$LOG1" 2>&1 &
PID1=$!

ready=0
for _ in {1..90}; do
  if curl -fsS "http://127.0.0.1:${PORT1}/v1/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" != "1" ]]; then
  echo "✗ FAIL: first daemon did not become healthy" >&2
  tail -n 80 "$LOG1" >&2 || true
  exit 1
fi
echo "✓ PASS: first daemon started"

status_json="$(curl -fsS "http://127.0.0.1:${PORT1}/v1/status")"
if echo "$status_json" | jq -e '.route_tier=="hot" and .summary_only==true and .runtime_process.current_pid and (.runtime_process.daemon_count == null) and (.runtime_process.duplicate_daemon_count == null) and (.runtime_process.single_daemon_ok == null) and (.cold_omitted | index("runtime_process.daemon_pids"))' >/dev/null; then
  echo "✓ PASS: hot status omits duplicate-detection scan fields"
else
  echo "✗ FAIL: hot status should omit duplicate-detection scan fields" >&2
  echo "$status_json" >&2
  exit 1
fi

status_deep_json="$(curl -fsS "http://127.0.0.1:${PORT1}/v1/status/deep")"
if echo "$status_deep_json" | jq -e '.route_tier=="cold" and .summary_only==false and .runtime_process.current_pid and (.runtime_process.daemon_count|type=="number") and (.runtime_process.duplicate_daemon_count|type=="number") and (.runtime_process.single_daemon_ok|type=="boolean")' >/dev/null; then
  echo "✓ PASS: deep status exposes runtime_process duplicate-detection fields"
else
  echo "✗ FAIL: deep status missing runtime_process duplicate-detection fields" >&2
  echo "$status_deep_json" >&2
  exit 1
fi

set +e
FOCUSA_BIND="127.0.0.1:${PORT2}" FOCUSA_DATA_DIR="$DATA_DIR" "$BIN" >"$LOG2" 2>&1
RC=$?
set -e
if [[ "$RC" -eq 0 ]]; then
  echo "✗ FAIL: second daemon unexpectedly started" >&2
  exit 1
fi
if grep -q "\[DAEMON_ALREADY_RUNNING\]" "$LOG2"; then
  echo "✓ PASS: second daemon rejected with typed error"
else
  echo "✗ FAIL: second daemon rejection missing typed error" >&2
  tail -n 80 "$LOG2" >&2 || true
  exit 1
fi

kill "$PID1" >/dev/null 2>&1 || true
wait "$PID1" 2>/dev/null || true
PID1=""

FOCUSA_BIND="127.0.0.1:${PORT2}" FOCUSA_DATA_DIR="$DATA_DIR" "$BIN" >"$LOG3" 2>&1 &
PID3=$!

ready=0
for _ in {1..90}; do
  if curl -fsS "http://127.0.0.1:${PORT2}/v1/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" != "1" ]]; then
  echo "✗ FAIL: daemon did not restart after owner shutdown" >&2
  tail -n 80 "$LOG3" >&2 || true
  exit 1
fi

echo "✓ PASS: daemon lock released on shutdown and restart works"
echo "SPEC82 single-daemon lock test: PASS"

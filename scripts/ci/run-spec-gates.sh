#!/bin/bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:18787}"
export FOCUSA_BASE_URL="$BASE_URL"
export FOCUSA_BIND="${FOCUSA_BIND:-127.0.0.1:18787}"

DAEMON_BIN="${DAEMON_BIN:-./target/release/focusa-daemon}"
if [ ! -x "$DAEMON_BIN" ]; then
  CARGO_BIN="${CARGO_BIN:-cargo}"
  "$CARGO_BIN" build -p focusa-api --release --bin focusa-daemon
fi
"$DAEMON_BIN" >/tmp/focusa-daemon.log 2>&1 &
DAEMON_PID=$!
cleanup() {
  kill "$DAEMON_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for i in $(seq 1 60); do
  if curl -fsS "${BASE_URL}/v1/health" >/dev/null; then
    break
  fi
  sleep 1
  if [ "$i" -eq 60 ]; then
    echo "daemon failed to become healthy"
    exit 1
  fi
done

./tests/focusa_toggle_persistence_test.sh
./tests/tool_contract_test.sh
./tests/pi_extension_contract_test.sh
./tests/channel_separation_test.sh
./tests/checkpoint_trigger_test.sh
./tests/trace_dimensions_test.sh

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIVE_TEST="$ROOT_DIR/tests/spec80_live_flow_runtime_test.sh"

if [[ -n "${FOCUSA_BASE_URL:-}" ]]; then
  echo "Using external daemon: $FOCUSA_BASE_URL"
  bash "$LIVE_TEST"
  exit 0
fi

PORT="${FOCUSA_TEST_PORT:-8791}"
LOG_FILE="/tmp/spec80-live-api-${PORT}-$$.log"

cd "$ROOT_DIR"
FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_TEST_MODE=1 cargo run -p focusa-api >"$LOG_FILE" 2>&1 &
API_PID=$!

cleanup() {
  if kill -0 "$API_PID" >/dev/null 2>&1; then
    kill "$API_PID" >/dev/null 2>&1 || true
    wait "$API_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

ready=0
for _ in {1..180}; do
  if curl -sS "http://127.0.0.1:${PORT}/v1/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done

if [[ "$ready" != "1" ]]; then
  echo "✗ FAIL: focusa-api did not become ready on port ${PORT}" >&2
  echo "--- daemon log ---" >&2
  tail -n 120 "$LOG_FILE" >&2 || true
  exit 1
fi

FOCUSA_BASE_URL="http://127.0.0.1:${PORT}/v1" bash "$LIVE_TEST"
echo "✓ PASS: SPEC80 live flow runtime with ephemeral daemon"

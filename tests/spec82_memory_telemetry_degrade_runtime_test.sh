#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/focusa-daemon"
PORT="18803"
BASE="http://127.0.0.1:${PORT}/v1"
DATA_DIR="$(mktemp -d /tmp/spec82-mem.XXXXXX)"
LOG="/tmp/spec82-mem-${PORT}.log"
PID=""

cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$DATA_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"
cargo build -p focusa-api --bin focusa-daemon >/dev/null

FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$DATA_DIR" \
FOCUSA_MEMORY_BUDGET_MB=1 \
"$BIN" >"$LOG" 2>&1 &
PID=$!

for _ in {1..120}; do
  if curl -fsS "$BASE/health" >/dev/null 2>&1; then break; fi
  sleep 1
done

status_json=$(curl -sS "$BASE/status")

jq -e '.runtime_memory.rss_kb != null and .runtime_memory.memory_budget_mb == 1 and .runtime_memory.host_mem_available_kb != null' <<<"$status_json" >/dev/null || {
  echo "✗ FAIL: runtime_memory fields missing"
  echo "$status_json"
  exit 1
}

jq -e '.runtime_memory.degraded == true' <<<"$status_json" >/dev/null || {
  echo "✗ FAIL: degraded should be true under 1MB budget"
  echo "$status_json"
  exit 1
}

echo "✓ PASS: runtime memory telemetry surfaced with budget/degraded flag"
echo "SPEC82 memory telemetry degrade runtime: PASS"

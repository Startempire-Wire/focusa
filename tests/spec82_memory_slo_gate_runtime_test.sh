#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[spec82-gate] running stabilization tests"
bash tests/spec82_single_daemon_lock_test.sh
bash tests/spec82_persistence_offload_runtime_test.sh
bash tests/spec82_retrieve_pagination_caps_runtime_test.sh
bash tests/spec82_memory_telemetry_degrade_runtime_test.sh

echo "[spec82-gate] checking non-degraded budget scenario"
BIN="$ROOT_DIR/target/debug/focusa-daemon"
PORT="18804"
BASE="http://127.0.0.1:${PORT}/v1"
DATA_DIR="$(mktemp -d /tmp/spec82-gate.XXXXXX)"
LOG="/tmp/spec82-gate-${PORT}.log"
PID=""

cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$DATA_DIR"
}
trap cleanup EXIT

FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$DATA_DIR" \
FOCUSA_MEMORY_BUDGET_MB=4096 \
"$BIN" >"$LOG" 2>&1 &
PID=$!

for _ in {1..120}; do
  if curl -fsS "$BASE/health" >/dev/null 2>&1; then break; fi
  sleep 1
done

status_json=$(curl -sS "$BASE/status")

jq -e '.runtime_memory.degraded == false and .runtime_memory.rss_kb != null and .runtime_memory.memory_budget_mb == 4096' <<<"$status_json" >/dev/null || {
  echo "✗ FAIL: expected non-degraded status under 4GB budget"
  echo "$status_json"
  exit 1
}

rss_kb=$(jq -r '.runtime_memory.rss_kb' <<<"$status_json")
report=$(jq -n --arg rss_kb "$rss_kb" '{spec:"82", gate:"memory_slo", status:"pass", rss_kb:($rss_kb|tonumber)}')
echo "$report" > /tmp/spec82_memory_slo_gate_report.json

echo "✓ PASS: memory SLO gate report /tmp/spec82_memory_slo_gate_report.json"
echo "SPEC82 memory SLO gate runtime: PASS"

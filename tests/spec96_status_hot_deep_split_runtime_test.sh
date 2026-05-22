#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
BIN="${CARGO_TARGET_DIR}/debug/focusa-daemon"
CARGO_BIN="${CARGO_BIN:-cargo}"
DATA_DIR="$(mktemp -d /tmp/spec96-status-split.XXXXXX)"
PORT="18796"
LOG="/tmp/spec96-status-split.log"
PID=""

cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$DATA_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"
"$CARGO_BIN" build --manifest-path "${ROOT_DIR}/Cargo.toml" -p focusa-api --bin focusa-daemon >/dev/null

FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="$DATA_DIR" "$BIN" >"$LOG" 2>&1 &
PID=$!

ready=0
for _ in {1..90}; do
  if curl -fsS "http://127.0.0.1:${PORT}/v1/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" != "1" ]]; then
  echo "✗ FAIL: daemon did not become healthy" >&2
  tail -n 80 "$LOG" >&2 || true
  exit 1
fi

echo "✓ PASS: daemon healthy"

hot_json="$(curl -fsS "http://127.0.0.1:${PORT}/v1/status")"
if echo "$hot_json" | jq -e '
  .status=="ok"
  and .route_tier=="hot"
  and .summary_only==true
  and .deep_status_route=="/v1/status/deep"
  and (.cold_omitted | index("last_event_ts"))
  and (.cold_omitted | index("persisted_event_count"))
  and (.cold_omitted | index("runtime_process.daemon_pids"))
  and (has("last_event_ts")|not)
  and (has("persisted_event_count")|not)
  and (.runtime_process.current_pid|type=="number")
  and (.runtime_process.daemon_pids == null)
  and (.runtime_process.daemon_count == null)
  and (.runtime_process.duplicate_daemon_count == null)
  and (.runtime_process.single_daemon_ok == null)
  and (.resource_mode.mode|type=="string")
' >/dev/null; then
  echo "✓ PASS: /v1/status is hot summary and omits cold diagnostics"
else
  echo "✗ FAIL: /v1/status hot summary contract violated" >&2
  echo "$hot_json" >&2
  exit 1
fi

deep_json="$(curl -fsS "http://127.0.0.1:${PORT}/v1/status/deep")"
if echo "$deep_json" | jq -e '
  .status=="ok"
  and .route_tier=="cold"
  and .summary_only==false
  and .cold_omitted==[]
  and has("last_event_ts")
  and has("persisted_event_count")
  and (.runtime_process.daemon_pids|type=="array")
  and (.runtime_process.daemon_count|type=="number")
  and (.runtime_process.duplicate_daemon_count|type=="number")
  and (.runtime_process.single_daemon_ok|type=="boolean")
' >/dev/null; then
  echo "✓ PASS: /v1/status/deep exposes cold diagnostics explicitly"
else
  echo "✗ FAIL: /v1/status/deep contract violated" >&2
  echo "$deep_json" >&2
  exit 1
fi

query_deep_json="$(curl -fsS "http://127.0.0.1:${PORT}/v1/status?deep=true")"
if echo "$query_deep_json" | jq -e '.route_tier=="cold" and .summary_only==false and (.runtime_process.daemon_pids|type=="array")' >/dev/null; then
  echo "✓ PASS: /v1/status?deep=true remains explicit cold opt-in"
else
  echo "✗ FAIL: /v1/status?deep=true should be cold opt-in" >&2
  echo "$query_deep_json" >&2
  exit 1
fi

query_summary_json="$(curl -fsS "http://127.0.0.1:${PORT}/v1/status?deep=true&summary_only=true")"
if echo "$query_summary_json" | jq -e '.route_tier=="hot" and .summary_only==true and (.runtime_process.daemon_pids == null)' >/dev/null; then
  echo "✓ PASS: summary_only overrides deep query for hot-path callers"
else
  echo "✗ FAIL: summary_only should force hot summary" >&2
  echo "$query_summary_json" >&2
  exit 1
fi

echo "SPEC96 status hot/deep split runtime test: PASS"

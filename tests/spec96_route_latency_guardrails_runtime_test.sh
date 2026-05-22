#!/bin/bash
set -euo pipefail

BASE_URL="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
HOT_MAX_MS="${FOCUSA_HOT_ROUTE_MAX_MS:-2000}"
COLD_TIMEOUT_SECONDS="${FOCUSA_COLD_ROUTE_TIMEOUT_SECONDS:-1}"

measure_ms() {
  local route="$1"
  local output code seconds
  output="$(curl -sS -o /tmp/focusa-latency-body.json -w '%{http_code} %{time_total}' --max-time 5 "${BASE_URL}${route}" 2>/tmp/focusa-latency-curl.err || true)"
  code="${output%% *}"
  seconds="${output#* }"
  if [[ -z "$code" || "$code" == "$output" ]]; then
    code="000"
    seconds="5"
  fi
  python3 - <<PY
seconds = float("${seconds:-5}")
print(int(seconds * 1000))
PY
  echo "$code" >/tmp/focusa-latency-code
}

hot_routes=(
  "/v1/health"
  "/v1/status?summary_only=true"
  "/v1/resource/mode"
  "/v1/work-loop/status?summary_only=true"
  "/v1/workpoint/current"
)

for route in "${hot_routes[@]}"; do
  ms="$(measure_ms "$route")"
  code="$(cat /tmp/focusa-latency-code)"
  if [[ "$code" != "200" ]]; then
    echo "✗ FAIL: hot route $route returned HTTP $code" >&2
    cat /tmp/focusa-latency-curl.err >&2 || true
    exit 1
  fi
  if [[ "$ms" -gt "$HOT_MAX_MS" ]]; then
    echo "✗ FAIL: hot route $route exceeded ${HOT_MAX_MS}ms (${ms}ms)" >&2
    exit 1
  fi
  echo "✓ PASS: hot route $route ${ms}ms"
done

# Cold route pressure may timeout; it must not imply daemon-wide outage when hot health remains OK.
set +e
curl -fsS --max-time "$COLD_TIMEOUT_SECONDS" "${BASE_URL}/v1/status/deep" >/tmp/focusa-latency-cold-status.json 2>/tmp/focusa-latency-cold.err
cold_rc=$?
set -e
if curl -fsS --max-time 2 "${BASE_URL}/v1/health" >/dev/null; then
  echo "✓ PASS: health remains OK after cold route probe rc=${cold_rc}"
else
  echo "✗ FAIL: health unavailable after cold route probe" >&2
  cat /tmp/focusa-latency-cold.err >&2 || true
  exit 1
fi

echo "SPEC96 route latency guardrails runtime test: PASS"

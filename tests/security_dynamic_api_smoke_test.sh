#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"
DAEMON_BIN="${DAEMON_BIN:-}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-security-smoke-target}"
DATA_DIR="${FOCUSA_SECURITY_SMOKE_DATA_DIR:-$(mktemp -d /tmp/focusa-security-smoke-data.XXXXXX)}"
LOG_FILE="${FOCUSA_SECURITY_SMOKE_LOG:-${DATA_DIR}/daemon.log}"
HEALTH_FILE="${DATA_DIR}/health.json"
MALFORMED_FILE="${DATA_DIR}/malformed.out"
OVERSIZED_FILE="${DATA_DIR}/oversized.out"
PORT="${FOCUSA_SECURITY_SMOKE_PORT:-$(python3 - <<'PY'
import socket
s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()
PY
)}"
BASE="http://127.0.0.1:${PORT}"
PID=""
cleanup() {
  if [[ -n "${PID}" ]] && kill -0 "${PID}" 2>/dev/null; then
    kill "${PID}" 2>/dev/null || true
    wait "${PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

command -v curl >/dev/null || { echo "curl required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 required" >&2; exit 1; }
mkdir -p "$DATA_DIR"
if [[ -z "$DAEMON_BIN" || ! -x "$DAEMON_BIN" ]]; then
  [[ -x "$CARGO_BIN" || -n "$(command -v "$CARGO_BIN" 2>/dev/null || true)" ]] || { echo "cargo required; set CARGO_BIN or executable DAEMON_BIN" >&2; exit 1; }
fi

cd "$ROOT_DIR"
if [[ -n "$DAEMON_BIN" && -x "$DAEMON_BIN" ]]; then
  FOCUSA_BIND="127.0.0.1:${PORT}" \
  FOCUSA_DATA_DIR="$DATA_DIR" \
  FOCUSA_API_MAX_BODY_BYTES=4096 \
  "$DAEMON_BIN" >"$LOG_FILE" 2>&1 &
else
  FOCUSA_BIND="127.0.0.1:${PORT}" \
  FOCUSA_DATA_DIR="$DATA_DIR" \
  FOCUSA_API_MAX_BODY_BYTES=4096 \
  CARGO_TARGET_DIR="$TARGET_DIR" \
  "$CARGO_BIN" run -p focusa-api --bin focusa-daemon >"$LOG_FILE" 2>&1 &
fi
PID=$!

for _ in $(seq 1 480); do
  if curl -fsS "$BASE/v1/health" >"$HEALTH_FILE" 2>/dev/null; then
    break
  fi
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "daemon exited before health; log follows" >&2
    tail -120 "$LOG_FILE" >&2 || true
    exit 1
  fi
  sleep 0.25
done
if ! curl -fsS "$BASE/v1/health" >/dev/null; then
  echo "daemon did not become healthy before timeout; log follows" >&2
  tail -160 "$LOG_FILE" >&2 || true
  exit 1
fi

malformed_code=$(printf '{' | curl -sS -o "$MALFORMED_FILE" -w '%{http_code}' \
  -H 'content-type: application/json' --data-binary @- "$BASE/v1/telemetry/trace" || true)
if [[ "$malformed_code" =~ ^2 ]]; then
  echo "malformed JSON unexpectedly succeeded" >&2
  cat "$MALFORMED_FILE" >&2 || true
  exit 1
fi

oversized_code=$(python3 - <<'PY' | curl -sS -o "$OVERSIZED_FILE" -w '%{http_code}' \
  -H 'content-type: application/json' --data-binary @- "$BASE/v1/telemetry/trace" || true
import json
print(json.dumps({"kind":"security_smoke", "payload":"x" * 8192}))
PY
)
if [[ "$oversized_code" != "413" ]]; then
  echo "oversized body expected HTTP 413, got ${oversized_code}" >&2
  cat "$OVERSIZED_FILE" >&2 || true
  exit 1
fi

schema_reject_count=0
expect_schema_reject() {
  local name="$1"
  local path="$2"
  local body="$3"
  local out_file="${DATA_DIR}/schema-${name}.out"
  local code
  code=$(printf '%s' "$body" | curl -sS -o "$out_file" -w '%{http_code}' \
    -H 'content-type: application/json' --data-binary @- "$BASE${path}" || true)
  if [[ "$code" =~ ^2 ]]; then
    echo "schema-level malformed payload unexpectedly succeeded for ${path}" >&2
    cat "$out_file" >&2 || true
    exit 1
  fi
  if [[ "$code" != "400" && "$code" != "422" ]]; then
    echo "schema-level malformed payload expected HTTP 400/422 for ${path}, got ${code}" >&2
    cat "$out_file" >&2 || true
    exit 1
  fi
  schema_reject_count=$((schema_reject_count + 1))
}

expect_schema_reject workpoint_checkpoint /v1/workpoint/checkpoint '{"checkpoint_reason":123}'
expect_schema_reject trajectory_define_goal /v1/trajectory/define-goal '{"long_term_goal":123,"desired_end_state":{}}'
expect_schema_reject prediction_record /v1/predictions '{"prediction_type":123,"confidence":"high"}'
expect_schema_reject metacog_capture /v1/metacognition/capture '{"kind":123,"content":{}}'
expect_schema_reject focus_update /v1/focus/update '{"updates":"not-array"}'

for _ in $(seq 1 10); do
  curl -fsS "$BASE/v1/health" >/dev/null
done

echo "✓ dynamic local API security smoke passed base=$BASE malformed_http=$malformed_code oversized_http=$oversized_code schema_rejects=$schema_reject_count"

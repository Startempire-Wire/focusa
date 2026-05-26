#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-security-smoke-target}"
DATA_DIR="${FOCUSA_SECURITY_SMOKE_DATA_DIR:-$(mktemp -d /tmp/focusa-security-smoke-data.XXXXXX)}"
LOG_FILE="${FOCUSA_SECURITY_SMOKE_LOG:-/tmp/focusa-security-smoke-daemon.log}"
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
[[ -x "$CARGO_BIN" || -n "$(command -v "$CARGO_BIN" 2>/dev/null || true)" ]] || { echo "cargo required; set CARGO_BIN" >&2; exit 1; }

cd "$ROOT_DIR"
FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$DATA_DIR" \
FOCUSA_API_MAX_BODY_BYTES=4096 \
CARGO_TARGET_DIR="$TARGET_DIR" \
"$CARGO_BIN" run -p focusa-api --bin focusa-daemon >"$LOG_FILE" 2>&1 &
PID=$!

for _ in $(seq 1 480); do
  if curl -fsS "$BASE/v1/health" >/tmp/focusa-security-smoke-health.json 2>/dev/null; then
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

malformed_code=$(printf '{' | curl -sS -o /tmp/focusa-security-smoke-malformed.out -w '%{http_code}' \
  -H 'content-type: application/json' --data-binary @- "$BASE/v1/telemetry/trace" || true)
if [[ "$malformed_code" =~ ^2 ]]; then
  echo "malformed JSON unexpectedly succeeded" >&2
  cat /tmp/focusa-security-smoke-malformed.out >&2 || true
  exit 1
fi

oversized_code=$(python3 - <<'PY' | curl -sS -o /tmp/focusa-security-smoke-oversized.out -w '%{http_code}' \
  -H 'content-type: application/json' --data-binary @- "$BASE/v1/telemetry/trace" || true
import json
print(json.dumps({"kind":"security_smoke", "payload":"x" * 8192}))
PY
)
if [[ "$oversized_code" != "413" ]]; then
  echo "oversized body expected HTTP 413, got ${oversized_code}" >&2
  cat /tmp/focusa-security-smoke-oversized.out >&2 || true
  exit 1
fi

for _ in $(seq 1 10); do
  curl -fsS "$BASE/v1/health" >/dev/null
done

echo "✓ dynamic local API security smoke passed base=$BASE malformed_http=$malformed_code oversized_http=$oversized_code"

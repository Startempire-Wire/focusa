#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-route-scope-target}"
DATA_DIR="${FOCUSA_ROUTE_SCOPE_DATA_DIR:-$(mktemp -d /tmp/focusa-route-scope-data.XXXXXX)}"
LOG_FILE="${FOCUSA_ROUTE_SCOPE_LOG:-/tmp/focusa-route-scope-daemon.log}"
TOKEN="${FOCUSA_ROUTE_SCOPE_TOKEN:-route-scope-test-token}"
PORT="${FOCUSA_ROUTE_SCOPE_PORT:-$(python3 - <<'PY'
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

cd "$ROOT_DIR"
FOCUSA_TEST_MODE=1 \
FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$DATA_DIR" \
FOCUSA_AUTH_TOKEN="$TOKEN" \
CARGO_TARGET_DIR="$TARGET_DIR" \
"$CARGO_BIN" run -p focusa-api --bin focusa-daemon >"$LOG_FILE" 2>&1 &
PID=$!

for _ in $(seq 1 480); do
  if curl -fsS "$BASE/v1/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "daemon exited before health; log follows" >&2
    tail -160 "$LOG_FILE" >&2 || true
    exit 1
  fi
  sleep 0.25
done
curl -fsS "$BASE/v1/health" >/dev/null

unauth_code=$(curl -sS -o /tmp/focusa-route-scope-unauth.out -w '%{http_code}' "$BASE/v1/info" || true)
[[ "$unauth_code" == "401" ]] || { echo "expected unauth /v1/info 401 got $unauth_code" >&2; exit 1; }

admin_write_code=$(curl -sS -o /tmp/focusa-route-scope-admin.out -w '%{http_code}' \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  --data '{"kind":"route_scope_test"}' "$BASE/v1/telemetry/trace" || true)
[[ "$admin_write_code" =~ ^2 ]] || { echo "expected daemon admin token write success without permission header got $admin_write_code" >&2; cat /tmp/focusa-route-scope-admin.out >&2 || true; exit 1; }

write_code=$(curl -sS -o /tmp/focusa-route-scope-write.out -w '%{http_code}' \
  -H "authorization: Bearer $TOKEN" -H 'x-focusa-permissions: telemetry:write' -H 'content-type: application/json' \
  --data '{"kind":"route_scope_test"}' "$BASE/v1/telemetry/trace" || true)
[[ "$write_code" =~ ^2 ]] || { echo "expected telemetry:write success got $write_code" >&2; cat /tmp/focusa-route-scope-write.out >&2 || true; exit 1; }

echo "✓ API route-scope dynamic smoke passed base=$BASE unauth=$unauth_code admin_write=$admin_write_code requested_scope_write=$write_code"

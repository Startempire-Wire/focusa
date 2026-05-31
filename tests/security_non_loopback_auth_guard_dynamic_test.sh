#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN="${CARGO_BIN:-cargo}"
DAEMON_BIN="${DAEMON_BIN:-}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-non-loopback-auth-target}"
FAIL_DATA_DIR="${FOCUSA_NON_LOOPBACK_FAIL_DATA_DIR:-$(mktemp -d /tmp/focusa-non-loopback-fail.XXXXXX)}"
PASS_DATA_DIR="${FOCUSA_NON_LOOPBACK_PASS_DATA_DIR:-$(mktemp -d /tmp/focusa-non-loopback-pass.XXXXXX)}"
FAIL_LOG="${FAIL_DATA_DIR}/daemon.log"
PASS_LOG="${PASS_DATA_DIR}/daemon.log"
FAIL_PORT="${FOCUSA_NON_LOOPBACK_FAIL_PORT:-$(python3 - <<'PY'
import socket
s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()
PY
)}"
PASS_PORT="${FOCUSA_NON_LOOPBACK_PASS_PORT:-$(python3 - <<'PY'
import socket
s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()
PY
)}"
TOKEN="${FOCUSA_NON_LOOPBACK_AUTH_TOKEN:-focusa-non-loopback-smoke-token}"
FAIL_PID=""
PASS_PID=""
cleanup() {
  for pid in "$FAIL_PID" "$PASS_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
}
trap cleanup EXIT

command -v curl >/dev/null || { echo "curl required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 required" >&2; exit 1; }
mkdir -p "$FAIL_DATA_DIR" "$PASS_DATA_DIR"
if [[ -z "$DAEMON_BIN" || ! -x "$DAEMON_BIN" ]]; then
  [[ -x "$CARGO_BIN" || -n "$(command -v "$CARGO_BIN" 2>/dev/null || true)" ]] || { echo "cargo required; set CARGO_BIN or executable DAEMON_BIN" >&2; exit 1; }
fi

run_daemon_no_token() {
  if [[ -n "$DAEMON_BIN" && -x "$DAEMON_BIN" ]]; then
    env -u FOCUSA_AUTH_TOKEN \
      FOCUSA_BIND="0.0.0.0:${FAIL_PORT}" \
      FOCUSA_DATA_DIR="$FAIL_DATA_DIR" \
      "$DAEMON_BIN" >"$FAIL_LOG" 2>&1 &
  else
    env -u FOCUSA_AUTH_TOKEN \
      FOCUSA_BIND="0.0.0.0:${FAIL_PORT}" \
      FOCUSA_DATA_DIR="$FAIL_DATA_DIR" \
      CARGO_TARGET_DIR="$TARGET_DIR" \
      "$CARGO_BIN" run -p focusa-api --bin focusa-daemon >"$FAIL_LOG" 2>&1 &
  fi
  FAIL_PID=$!
}

run_daemon_with_token() {
  if [[ -n "$DAEMON_BIN" && -x "$DAEMON_BIN" ]]; then
    FOCUSA_AUTH_TOKEN="$TOKEN" \
      FOCUSA_BIND="0.0.0.0:${PASS_PORT}" \
      FOCUSA_DATA_DIR="$PASS_DATA_DIR" \
      "$DAEMON_BIN" >"$PASS_LOG" 2>&1 &
  else
    FOCUSA_AUTH_TOKEN="$TOKEN" \
      FOCUSA_BIND="0.0.0.0:${PASS_PORT}" \
      FOCUSA_DATA_DIR="$PASS_DATA_DIR" \
      CARGO_TARGET_DIR="$TARGET_DIR" \
      "$CARGO_BIN" run -p focusa-api --bin focusa-daemon >"$PASS_LOG" 2>&1 &
  fi
  PASS_PID=$!
}

cd "$ROOT_DIR"
run_daemon_no_token
FAIL_BASE="http://127.0.0.1:${FAIL_PORT}"
for _ in $(seq 1 40); do
  if ! kill -0 "$FAIL_PID" 2>/dev/null; then
    break
  fi
  if curl -fsS "$FAIL_BASE/v1/health" >/dev/null 2>&1; then
    echo "non-loopback daemon without FOCUSA_AUTH_TOKEN unexpectedly became healthy" >&2
    exit 1
  fi
  sleep 0.25
done
if kill -0 "$FAIL_PID" 2>/dev/null; then
  echo "non-loopback daemon without FOCUSA_AUTH_TOKEN stayed running" >&2
  tail -80 "$FAIL_LOG" >&2 || true
  exit 1
fi
set +e
wait "$FAIL_PID"
FAIL_CODE=$?
set -e
if [[ "$FAIL_CODE" -eq 0 ]]; then
  echo "non-loopback daemon without FOCUSA_AUTH_TOKEN exited successfully; expected failure" >&2
  exit 1
fi
grep -Fq "INSECURE_BIND_WITHOUT_AUTH" "$FAIL_LOG" || { echo "missing INSECURE_BIND_WITHOUT_AUTH in failure log" >&2; tail -80 "$FAIL_LOG" >&2 || true; exit 1; }

run_daemon_with_token
PASS_BASE="http://127.0.0.1:${PASS_PORT}"
for _ in $(seq 1 120); do
  if curl -fsS "$PASS_BASE/v1/health" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$PASS_PID" 2>/dev/null; then
    echo "authenticated non-loopback daemon exited before health" >&2
    tail -120 "$PASS_LOG" >&2 || true
    exit 1
  fi
  sleep 0.25
done
curl -fsS "$PASS_BASE/v1/health" >/dev/null || { echo "authenticated non-loopback daemon did not become healthy" >&2; tail -120 "$PASS_LOG" >&2 || true; exit 1; }

unauth_code="$(curl -sS -o "${PASS_DATA_DIR}/unauth.out" -w "%{http_code}" "$PASS_BASE/v1/project/identity" || true)"
if [[ "$unauth_code" != "401" ]]; then
  echo "unauthenticated non-health route expected 401, got ${unauth_code}" >&2
  cat "${PASS_DATA_DIR}/unauth.out" >&2 || true
  exit 1
fi

auth_code="$(curl -sS -o "${PASS_DATA_DIR}/auth.out" -w "%{http_code}" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "x-focusa-permissions: project:read" \
  "$PASS_BASE/v1/project/identity" || true)"
if [[ "$auth_code" != "200" ]]; then
  echo "authenticated project identity expected 200, got ${auth_code}" >&2
  cat "${PASS_DATA_DIR}/auth.out" >&2 || true
  exit 1
fi

echo "✓ non-loopback auth startup smoke passed fail_port=${FAIL_PORT} pass_port=${PASS_PORT} unauth_http=${unauth_code} auth_http=${auth_code}"

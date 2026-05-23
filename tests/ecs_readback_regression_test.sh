#!/bin/bash
# Regression: ECS store/resolve/content/rehydrate preserves handle metadata and blob readback.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BASE_URL="${FOCUSA_ECS_READBACK_BASE_URL:-http://127.0.0.1:18884}"
BIND_ADDR="${FOCUSA_ECS_READBACK_BIND:-127.0.0.1:18884}"
DATA_DIR="${FOCUSA_ECS_READBACK_DATA_DIR:-$(mktemp -d /tmp/focusa-ecs-readback.XXXXXX)}"
DAEMON_BIN="${DAEMON_BIN:-${ROOT_DIR}/target/debug/focusa-daemon}"
FAILED=0
PASSED=0
RED="\033[0;31m"
GREEN="\033[0;32m"
NC="\033[0m"

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED + 1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED + 1)); }

cleanup() {
  if [ -n "${DAEMON_PID:-}" ]; then
    kill "$DAEMON_PID" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$ROOT_DIR"
cargo build -q -p focusa-api --bin focusa-daemon
cargo build -q -p focusa-cli

FOCUSA_BIND="$BIND_ADDR" FOCUSA_BASE_URL="$BASE_URL" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-ecs-readback.log 2>&1 &
DAEMON_PID=$!

READY=0
for _ in $(seq 1 80); do
  if curl -fsS "$BASE_URL/v1/health" >/dev/null 2>&1; then
    READY=1
    break
  fi
  sleep 0.25
done
if [ "$READY" -eq 1 ]; then
  log_pass "daemon health ready"
else
  log_fail "daemon health not ready"
fi

STORE=$(curl -fsS -X POST "$BASE_URL/v1/ecs/store" -H "Content-Type: application/json" \
  -d "{\"kind\":\"text\",\"label\":\"ecs-readback-regression\",\"content\":\"ECS smoke content roundtrip\"}" || true)
HANDLE_ID=$(echo "$STORE" | jq -r ".id // empty" 2>/dev/null || true)
if [ -n "$HANDLE_ID" ]; then
  log_pass "ecs store returns handle id"
else
  log_fail "ecs store did not return handle id"
fi

RESOLVE=$(curl -fsS "$BASE_URL/v1/ecs/resolve/$HANDLE_ID" || true)
if echo "$RESOLVE" | jq -e ".handle.sha256 | length > 0" >/dev/null 2>&1; then
  log_pass "ecs resolve returns non-empty sha256 metadata"
else
  log_fail "ecs resolve missing full handle metadata"
fi

CONTENT=$(curl -fsS "$BASE_URL/v1/ecs/content/$HANDLE_ID" || true)
if echo "$CONTENT" | jq -e ".content_b64 == \"RUNTIHNtb2tlIGNvbnRlbnQgcm91bmR0cmlw\" and .size == 27" >/dev/null 2>&1; then
  log_pass "ecs content returns exact stored bytes"
else
  log_fail "ecs content failed exact byte readback"
fi

REHYDRATE=$(curl -fsS -X POST "$BASE_URL/v1/ecs/rehydrate/$HANDLE_ID?max_tokens=3" || true)
if echo "$REHYDRATE" | jq -e ".content | startswith(\"ECS smoke\")" >/dev/null 2>&1; then
  log_pass "ecs rehydrate returns budgeted content snippet"
else
  log_fail "ecs rehydrate failed budgeted content readback"
fi

CLI_META=$(FOCUSA_BASE_URL="$BASE_URL" target/debug/focusa --json ecs meta "$HANDLE_ID" 2>/dev/null || true)
if echo "$CLI_META" | jq -e ".handle.sha256 | length > 0" >/dev/null 2>&1; then
  log_pass "CLI ecs meta returns full metadata"
else
  log_fail "CLI ecs meta missing full metadata"
fi

CLI_REHYDRATE=$(FOCUSA_BASE_URL="$BASE_URL" target/debug/focusa --json ecs rehydrate "$HANDLE_ID" --max-tokens 3 2>/dev/null || true)
if echo "$CLI_REHYDRATE" | jq -e ".content | startswith(\"ECS smoke\")" >/dev/null 2>&1; then
  log_pass "CLI ecs rehydrate returns content snippet"
else
  log_fail "CLI ecs rehydrate failed content readback"
fi

echo "=== ECS READBACK REGRESSION RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  echo "Daemon log: /tmp/focusa-ecs-readback.log"
  exit 1
fi

#!/bin/bash
# Regression: reflection CLI/runtime should complete within CLI timeout budget.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/release/focusa-daemon}"
BASE_URL="${FOCUSA_REFLECT_TEST_BASE_URL:-http://127.0.0.1:18886}"
BIND_ADDR="${FOCUSA_REFLECT_TEST_BIND:-127.0.0.1:18886}"
DATA_DIR="${FOCUSA_REFLECT_TEST_DATA_DIR:-$(mktemp -d /tmp/focusa-reflect-cli.XXXXXX)}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

cleanup() {
  if [ -n "${DAEMON_PID:-}" ]; then
    kill "$DAEMON_PID" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$REPO_ROOT"
cargo build -q -p focusa-api --bin focusa-daemon --release

FOCUSA_BIND="$BIND_ADDR" FOCUSA_BASE_URL="$BASE_URL" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-reflection-cli.log 2>&1 &
DAEMON_PID=$!

for _ in $(seq 1 120); do
  if curl -fsS "$BASE_URL/v1/status" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

RUN_JSON=$(FOCUSA_BASE_URL="$BASE_URL" cargo run -q -p focusa-cli -- --json reflect run --window 1h --budget 50 2>/tmp/focusa-reflect-run.err || true)
if echo "$RUN_JSON" | jq -e '.status == "accepted" and .result.iteration_id and .result.stop_reason' >/dev/null 2>&1; then
  log_pass "reflect run completes through CLI and returns accepted payload"
else
  log_fail "reflect run did not return accepted payload"
fi

TICK_JSON=$(FOCUSA_BASE_URL="$BASE_URL" cargo run -q -p focusa-cli -- --json reflect scheduler tick --window 1h --budget 50 2>/tmp/focusa-reflect-tick.err || true)
if echo "$TICK_JSON" | jq -e '.status == "accepted" or (.status == "skipped" and .reason)' >/dev/null 2>&1; then
  log_pass "reflect scheduler tick returns structured result through CLI"
else
  log_fail "reflect scheduler tick did not return structured result"
fi

if [ "$FAILED" -ne 0 ]; then
  echo
  echo "Failed: $FAILED, Passed: $PASSED"
  exit 1
fi

echo
printf "Passed: %s, Failed: %s\n" "$PASSED" "$FAILED"

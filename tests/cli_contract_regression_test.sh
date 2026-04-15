#!/bin/bash
# Regression: CLI JSON contracts + command submit surface.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/release/focusa-daemon}"
BASE_URL="${FOCUSA_CLI_CONTRACT_BASE_URL:-http://127.0.0.1:18883}"
BIND_ADDR="${FOCUSA_CLI_CONTRACT_BIND:-127.0.0.1:18883}"
DATA_DIR="${FOCUSA_CLI_CONTRACT_DATA_DIR:-$(mktemp -d /tmp/focusa-cli-contract.XXXXXX)}"
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

FOCUSA_BIND="$BIND_ADDR" FOCUSA_BASE_URL="$BASE_URL" FOCUSA_DATA_DIR="$DATA_DIR" "$DAEMON_BIN" >/tmp/focusa-cli-contract.log 2>&1 &
DAEMON_PID=$!

for _ in $(seq 1 80); do
  if curl -fsS "$BASE_URL/v1/status" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

run_cli() {
  FOCUSA_BASE_URL="$BASE_URL" cargo run -q -p focusa-cli -- "$@"
}

THREADS_JSON=$(run_cli --json thread list 2>/dev/null || true)
if echo "$THREADS_JSON" | jq -e '.threads and (.threads | type == "array")' >/dev/null 2>&1; then
  log_pass "thread list --json returns API-shaped JSON"
else
  log_fail "thread list --json did not return JSON thread list"
fi

CREATE_JSON=$(run_cli --json thread create --name regression-thread --intent "CLI regression verification" 2>/dev/null || true)
THREAD_ID=$(echo "$CREATE_JSON" | jq -r '.thread.id // empty' 2>/dev/null || true)
if [ -n "$THREAD_ID" ]; then
  log_pass "thread create --json returns created thread payload"
else
  log_fail "thread create --json missing thread payload"
fi

EXPORT_STATUS=$(run_cli --json export status 2>/dev/null || true)
if echo "$EXPORT_STATUS" | jq -e '.status == "not_implemented" and (.dataset_types | type == "array") and has("contribution_enabled") | not' >/dev/null 2>&1; then
  log_pass "export status --json reports export pipeline state, not contribution queue"
else
  log_fail "export status --json still looks miswired"
fi

EXPORT_DRY_RUN=$(run_cli --json export sft --output /tmp/focusa-export.jsonl --dry-run --explain 2>/dev/null || true)
if echo "$EXPORT_DRY_RUN" | jq -e '.status == "not_implemented" and .dry_run == true and (.dataset_flags.min_turns == 3)' >/dev/null 2>&1; then
  log_pass "export sft --dry-run --json returns structured not_implemented payload"
else
  log_fail "export sft dry-run json payload missing expected structure"
fi

CACHE_BUST=$(curl -fsS -X POST "$BASE_URL/v1/commands/submit" -H 'Content-Type: application/json' \
  -d '{"command":"cache.bust","payload":{"category":"FreshEvidence"}}' || true)
if echo "$CACHE_BUST" | jq -e '.status == "dispatched" or .status == "accepted"' >/dev/null 2>&1; then
  log_pass "commands submit accepts cache.bust"
else
  log_fail "commands submit still rejects cache.bust"
fi

curl -fsS -X POST "$BASE_URL/v1/commands/submit" -H 'Content-Type: application/json' \
  -d '{"command":"session.start","payload":{"adapter_id":"pi","workspace_id":"contract-test"}}' >/dev/null
sleep 0.2
INVALID_PUSH_BODY='{"command":"focus.push_frame","payload":{"title":"bad","goal":"bad","beads_issue_id":""}}'
INVALID_PUSH_CODE=$(curl -s -o /tmp/focusa-invalid-push.json -w '%{http_code}' -X POST "$BASE_URL/v1/commands/submit" -H 'Content-Type: application/json' -d "$INVALID_PUSH_BODY")
if [ "$INVALID_PUSH_CODE" = "400" ] && jq -e '.reason == "missing_beads_issue_id"' /tmp/focusa-invalid-push.json >/dev/null 2>&1; then
  log_pass "commands submit rejects invalid focus.push_frame before acceptance"
else
  log_fail "commands submit did not reject invalid focus.push_frame correctly"
fi

curl -fsS -X POST "$BASE_URL/v1/commands/submit" -H 'Content-Type: application/json' \
  -d '{"command":"session.close","payload":{"reason":"done"}}' >/dev/null
for _ in $(seq 1 40); do
  if curl -fsS "$BASE_URL/v1/status" | jq -e '.session.status == "closed"' >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done
INVALID_CLOSE_CODE=$(curl -s -o /tmp/focusa-invalid-close.json -w '%{http_code}' -X POST "$BASE_URL/v1/commands/submit" -H 'Content-Type: application/json' -d '{"command":"session.close","payload":{"reason":"none"}}')
if [ "$INVALID_CLOSE_CODE" = "400" ] && jq -e '.reason == "session_inactive" and .session_status == "closed"' /tmp/focusa-invalid-close.json >/dev/null 2>&1; then
  log_pass "commands submit rejects session.close with inactive session"
else
  log_fail "commands submit did not reject invalid session.close correctly"
fi

if [ "$FAILED" -ne 0 ]; then
  echo
  echo "Failed: $FAILED, Passed: $PASSED"
  exit 1
fi

echo
printf "Passed: %s, Failed: %s\n" "$PASSED" "$FAILED"

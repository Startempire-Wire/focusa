#!/bin/bash
# Contract: Spec80 performance/CLI wrapper gate requires deterministic API timeout behavior and typed JSON error envelope.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
API_CLIENT_FILE="${ROOT_DIR}/crates/focusa-cli/src/api_client.rs"
MAIN_FILE="${ROOT_DIR}/crates/focusa-cli/src/main.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '\[API_TIMEOUT\]|\[API_CONNECT_ERROR\]|FOCUSA_API_TIMEOUT' "$API_CLIENT_FILE" >/dev/null 2>&1; then
  log_pass "api client classifies timeout/connect failures with typed markers"
else
  log_fail "api client missing typed timeout/connect classification"
fi

if rg -n 'classify_cli_error|"status": "error"|"code": code|"reason": reason' "$MAIN_FILE" >/dev/null 2>&1; then
  log_pass "CLI main emits typed JSON error envelope in --json mode"
else
  log_fail "CLI main missing typed JSON error envelope"
fi

ERR_JSON=$(cd "$ROOT_DIR" && FOCUSA_API_URL="http://127.0.0.1:9" FOCUSA_API_TIMEOUT=1 cargo run -q -p focusa-cli -- --json status 2>/dev/null || true)
if echo "$ERR_JSON" | jq -e '.status == "error" and (.code == "API_CONNECT_ERROR" or .code == "API_TIMEOUT" or .code == "API_REQUEST_ERROR") and (.reason | type == "string")' >/dev/null 2>&1; then
  log_pass "runtime JSON envelope returns typed code on unreachable API"
else
  log_fail "runtime JSON envelope not typed for unreachable API"
fi

echo "=== SPEC80 IMPL CLI TIMEOUT/ERROR ENVELOPE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

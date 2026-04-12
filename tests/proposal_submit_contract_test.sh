#!/bin/bash
# SPEC-41 proposal submit contract
# Submit must return proposal identity, not only an advisory status.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

source_id="proposal-submit-contract-$(date +%s%N)"
resp=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"memory_write\",\"source\":\"${source_id}\",\"score\":0.88,\"deadline_ms\":60000,\"payload\":{\"key\":\"submit-contract-key\",\"value\":\"submit-contract-value\"}}")

if echo "$resp" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "Submit accepted"
else
  log_fail "Submit not accepted :: $resp"
fi

proposal_id=$(echo "$resp" | jq -r '.proposal_id // empty')
if [ -n "$proposal_id" ] && [ "$proposal_id" != "null" ]; then
  log_pass "Submit returned proposal_id"
else
  log_fail "Submit missing proposal_id :: $resp"
fi

if echo "$resp" | jq -e '.kind == "memory_write" and .target_class == "memory"' >/dev/null 2>&1; then
  log_pass "Submit returned kind and target_class"
else
  log_fail "Submit missing kind/target_class :: $resp"
fi

if [ -n "$proposal_id" ] && curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg id "$proposal_id" '.proposals | any(.id == $id and .status == "pending")' >/dev/null 2>&1; then
  log_pass "Returned proposal_id matches persisted pending proposal"
else
  log_fail "Returned proposal_id not found in persisted proposals"
fi

# Cleanup: resolve this memory proposal so later strict tests run in isolation.
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg id "$proposal_id" '.proposals | any(.id == $id and .status == "pending")' >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done
curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d '{"kind":"memory_write"}' >/dev/null || true

echo ""
echo "=== PROPOSAL SUBMIT CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Proposal submit contract verified${NC}"
  exit 0
else
  echo -e "${RED}Proposal submit contract failed${NC}"
  exit 1
fi

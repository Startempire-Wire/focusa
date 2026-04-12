#!/bin/bash
# SPEC-50 proposal resolution enforcement
# Accepted proposals must mutate canonical state, not just return scores.

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

http_json() {
  curl -sS "$@"
}

before_count=$(http_json "${BASE_URL}/v1/focus/stack" | jq '.stack.frames | length')
name="proposal-enforced-$(date +%s%N)"

log_info "Submit high-score focus_change proposal"
submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"focus_change\",\"source\":\"spec-50-test\",\"score\":0.95,\"deadline_ms\":60000,\"payload\":{\"title\":\"${name}\",\"goal\":\"${name}\",\"beads_issue_id\":\"spec50-enforcement\",\"tags\":[\"spec50\"]}}")
if echo "$submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "Proposal submission accepted"
else
  log_fail "Proposal submission failed :: $submit"
fi

log_info "Resolve proposals"
resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d '{"kind":"focus_change"}')
if echo "$resolve" | jq -e '.status == "accepted" and .applied_kind == "focus_frame_pushed"' >/dev/null 2>&1; then
  log_pass "Proposal resolution reported canonical application"
else
  log_fail "Proposal resolution did not report canonical application :: $resolve"
fi

after_stack=$(http_json "${BASE_URL}/v1/focus/stack")
after_count=$(echo "$after_stack" | jq '.stack.frames | length')
if [ "$after_count" -gt "$before_count" ]; then
  log_pass "Accepted proposal increased canonical focus stack size"
else
  log_fail "Accepted proposal did not change canonical focus stack"
fi

if echo "$after_stack" | jq -e --arg title "$name" '.stack.frames | any(.title == $title)' >/dev/null 2>&1; then
  log_pass "Applied focus frame visible in canonical stack"
else
  log_fail "Applied focus frame not visible in canonical stack"
fi

if http_json "${BASE_URL}/v1/proposals" | jq -e '.proposals | any(.source == "spec-50-test" and .status == "accepted")' >/dev/null 2>&1; then
  log_pass "Winner proposal status persisted as accepted"
else
  log_fail "Winner proposal status not persisted as accepted"
fi

echo ""
echo "=== PROPOSAL RESOLUTION ENFORCEMENT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Proposal resolution enforcement verified${NC}"
  exit 0
else
  echo -e "${RED}Proposal resolution enforcement failed${NC}"
  exit 1
fi

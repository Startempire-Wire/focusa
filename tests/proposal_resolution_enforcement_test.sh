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

curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d '{"kind":"focus_change","source":"spec50-cleanup"}' >/dev/null 2>&1 || true

before_count=$(http_json "${BASE_URL}/v1/focus/stack" | jq '.stack.frames | length')
name="proposal-enforced-$(date +%s%N)"
source="spec-50-test-${name}"

log_info "Submit high-score focus_change proposal"
submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"focus_change\",\"source\":\"${source}\",\"score\":0.999,\"deadline_ms\":60000,\"payload\":{\"title\":\"${name}\",\"goal\":\"${name}\",\"beads_issue_id\":\"spec50-enforcement\",\"tags\":[\"spec50\"]}}")
if echo "$submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "Proposal submission accepted"
else
  log_fail "Proposal submission failed :: $submit"
fi

visible=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if http_json "${BASE_URL}/v1/proposals" | jq -e --arg source "$source" --arg title "$name" '.proposals | any(.source == $source and .score >= 0.999 and .payload.title == $title and .status == "pending")' >/dev/null 2>&1; then
    visible=1
    break
  fi
  sleep 0.1
done
if [ "$visible" = "1" ]; then
  log_pass "High-score focus proposal visible before resolve"
else
  log_fail "High-score focus proposal not visible before resolve"
fi

log_info "Resolve proposals"
resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"focus_change\",\"source\":\"${source}\"}")
if echo "$resolve" | jq -e '.status == "accepted" and .applied_kind == "focus_frame_pushed"' >/dev/null 2>&1; then
  log_pass "Proposal resolution reported canonical application"
else
  log_fail "Proposal resolution did not report canonical application :: $resolve"
fi

after_stack=''
after_count="$before_count"
frame_visible=0
for _ in $(seq 1 30); do
  after_stack=$(http_json "${BASE_URL}/v1/focus/stack")
  after_count=$(echo "$after_stack" | jq '.stack.frames | length')
  if echo "$after_stack" | jq -e --arg title "$name" '.stack.frames | any(.title == $title)' >/dev/null 2>&1; then
    frame_visible=1
    break
  fi
  sleep 0.25
done
if [ "$after_count" -gt "$before_count" ]; then
  log_pass "Accepted proposal increased canonical focus stack size"
else
  log_fail "Accepted proposal did not change canonical focus stack"
fi

if [ "$frame_visible" = "1" ]; then
  log_pass "Applied focus frame visible in canonical stack"
else
  log_fail "Applied focus frame not visible in canonical stack"
fi

accepted_visible=0
for _ in $(seq 1 30); do
  if http_json "${BASE_URL}/v1/proposals" | jq -e --arg source "$source" '.proposals | any(.source == $source and .status == "accepted")' >/dev/null 2>&1; then
    accepted_visible=1
    break
  fi
  sleep 0.25
done
if [ "$accepted_visible" = "1" ]; then
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

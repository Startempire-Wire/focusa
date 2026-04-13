#!/bin/bash
# SPEC-41/50 proposal governance enforcement
# Accepted autonomy_adjustment and constitution_revision proposals must mutate canonical state.

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
wait_for_jq() {
  local url="$1"
  local expr="$2"
  local tries="${3:-60}"
  local delay="${4:-0.25}"
  for _ in $(seq 1 "$tries"); do
    if curl -sS "$url" | jq -e "$expr" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay"
  done
  return 1
}

run_source="spec-governance-test-$(date +%s%N)"

log_info "Submit autonomy_adjustment proposal"
auto_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"autonomy_adjustment\",\"source\":\"${run_source}\",\"score\":0.98,\"deadline_ms\":60000,\"payload\":{\"level\":\"AL2\",\"scope\":\"spec-governance-scope\",\"ttl_seconds\":3600,\"reason\":\"strict proposal governance test\"}}")
if echo "$auto_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "autonomy_adjustment proposal accepted"
else
  log_fail "autonomy_adjustment proposal rejected :: $auto_submit"
fi

auto_visible=0
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg source "$run_source" '.proposals | any(.source == $source and .kind == "autonomy_adjustment" and .status == "pending")' >/dev/null 2>&1; then
    auto_visible=1
    break
  fi
  sleep 0.1
done
if [ "$auto_visible" != "1" ]; then
  log_fail "autonomy_adjustment proposal not visible as pending before resolve"
fi

auto_resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"autonomy_adjustment\",\"source\":\"${run_source}\"}")
if echo "$auto_resolve" | jq -e '.status == "accepted" and .applied_kind == "autonomy_level_granted"' >/dev/null 2>&1; then
  log_pass "autonomy_adjustment applied canonically"
else
  log_fail "autonomy_adjustment resolve failed :: $auto_resolve"
fi

if wait_for_jq "${BASE_URL}/v1/autonomy" '.level == "AL2" and .granted_scope == "spec-governance-scope"' 240 0.25; then
  log_pass "Autonomy state mutated canonically"
else
  log_fail "Autonomy state not updated canonically"
fi

version="proposal-governance-$(date +%s%N)"
log_info "Submit constitution_revision proposal"
const_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"constitution_revision\",\"source\":\"${run_source}\",\"score\":0.99,\"deadline_ms\":60000,\"payload\":{\"version\":\"${version}\",\"agent_id\":\"wirebot\",\"principles\":[\"Protect operator intent\",\"Prefer verified state transitions\"],\"safety_rules\":[\"No silent mutation\"],\"expression_rules\":[\"Be direct\"]}}")
if echo "$const_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "constitution_revision proposal accepted"
else
  log_fail "constitution_revision proposal rejected :: $const_submit"
fi

const_visible=0
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg source "$run_source" '.proposals | any(.source == $source and .kind == "constitution_revision" and .status == "pending")' >/dev/null 2>&1; then
    const_visible=1
    break
  fi
  sleep 0.1
done
if [ "$const_visible" != "1" ]; then
  log_fail "constitution_revision proposal not visible as pending before resolve"
fi

const_resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"constitution_revision\",\"source\":\"${run_source}\"}")
if echo "$const_resolve" | jq -e '.status == "accepted" and .applied_kind == "constitution_version_activated"' >/dev/null 2>&1; then
  log_pass "constitution_revision applied canonically"
else
  log_fail "constitution_revision resolve failed :: $const_resolve"
fi

if wait_for_jq "${BASE_URL}/v1/constitution/versions" '.active == "'"$version"'" and (.versions | any(. == "'"$version"'"))' 240 0.25; then
  log_pass "Constitution active version mutated canonically"
else
  log_fail "Constitution active version not updated canonically"
fi

if wait_for_jq "${BASE_URL}/v1/constitution/active" '.version == "'"$version"'" and (.principles | length) >= 2' 240 0.25; then
  log_pass "Active constitution payload updated canonically"
else
  log_fail "Active constitution payload not updated canonically"
fi

echo ""
echo "=== PROPOSAL GOVERNANCE ENFORCEMENT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Proposal governance enforcement verified${NC}"
  exit 0
else
  echo -e "${RED}Proposal governance enforcement failed${NC}"
  exit 1
fi

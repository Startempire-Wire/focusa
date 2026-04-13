#!/bin/bash
# SPEC-41/50 proposal kind enforcement
# Accepted thesis_update and memory_write proposals must mutate canonical state.

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

run_source="spec-kind-test-$(date +%s%N)"
thread_name="proposal-kind-thread-$(date +%s%N)"
thread_resp=$(curl -sS -X POST "${BASE_URL}/v1/threads" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"${thread_name}\",\"primary_intent\":\"seed thread for proposal kinds\"}")
thread_id=$(echo "$thread_resp" | jq -r '.thread.id // .thread_id // empty')
if [ -n "$thread_id" ] && [ "$thread_id" != "null" ]; then
  log_pass "Seed thread created for thesis proposal"
else
  log_fail "Failed to seed thread :: $thread_resp"
fi

thesis_intent="thesis-updated-$(date +%s%N)"
log_info "Submit thesis_update proposal"
thesis_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"thesis_update\",\"source\":\"${run_source}\",\"score\":0.96,\"deadline_ms\":60000,\"payload\":{\"thread_id\":\"${thread_id}\",\"primary_intent\":\"${thesis_intent}\",\"secondary_goals\":[\"goal-a\"],\"sources\":[\"spec-kind-test\"]}}")
if echo "$thesis_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "thesis_update proposal accepted"
else
  log_fail "thesis_update proposal rejected :: $thesis_submit"
fi

thesis_visible=0
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg source "$run_source" '.proposals | any(.source == $source and .kind == "thesis_update" and .status == "pending")' >/dev/null 2>&1; then
    thesis_visible=1
    break
  fi
  sleep 0.1
done
if [ "$thesis_visible" != "1" ]; then
  log_fail "thesis_update proposal not visible as pending before resolve"
fi

thesis_resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"thesis_update\",\"source\":\"${run_source}\"}")
if echo "$thesis_resolve" | jq -e '.status == "accepted" and .applied_kind == "thread_thesis_updated"' >/dev/null 2>&1; then
  log_pass "thesis_update proposal applied canonically"
else
  log_fail "thesis_update resolve failed :: $thesis_resolve"
fi

thesis_updated=0
for _ in $(seq 1 60); do
  if curl -sS "${BASE_URL}/v1/threads/${thread_id}" | jq -e --arg intent "$thesis_intent" '.thread.thesis.primary_intent == $intent' >/dev/null 2>&1; then
    thesis_updated=1
    break
  fi
  sleep 0.25
done
if [ "$thesis_updated" = "1" ]; then
  log_pass "Thread thesis mutated canonically"
else
  log_fail "Thread thesis not updated"
fi

mem_key="proposal-kind-key-$(date +%s%N)"
mem_val="proposal-kind-value-$(date +%s%N)"
log_info "Submit memory_write proposal"
mem_submit=$(curl -sS -X POST "${BASE_URL}/v1/proposals" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"memory_write\",\"source\":\"${run_source}\",\"score\":0.999,\"deadline_ms\":60000,\"payload\":{\"key\":\"${mem_key}\",\"value\":\"${mem_val}\",\"source\":\"${run_source}\"}}")
if echo "$mem_submit" | jq -e '.status == "accepted"' >/dev/null 2>&1; then
  log_pass "memory_write proposal accepted"
else
  log_fail "memory_write proposal rejected :: $mem_submit"
fi

mem_visible=0
for _ in 1 2 3 4 5 6 7 8 9 10; do
  if curl -sS "${BASE_URL}/v1/proposals" | jq -e --arg source "$run_source" '.proposals | any(.source == $source and .kind == "memory_write" and .status == "pending")' >/dev/null 2>&1; then
    mem_visible=1
    break
  fi
  sleep 0.1
done
if [ "$mem_visible" != "1" ]; then
  log_fail "memory_write proposal not visible as pending before resolve"
fi

mem_resolve=$(curl -sS -X POST "${BASE_URL}/v1/proposals/resolve" \
  -H "Content-Type: application/json" \
  -d "{\"kind\":\"memory_write\",\"source\":\"${run_source}\"}")
if echo "$mem_resolve" | jq -e '.status == "accepted" and .applied_kind == "semantic_memory_upserted"' >/dev/null 2>&1; then
  log_pass "memory_write proposal applied canonically"
else
  log_fail "memory_write resolve failed :: $mem_resolve"
fi

mem_updated=0
for _ in $(seq 1 30); do
  if curl -sS "${BASE_URL}/v1/memory/semantic" | jq -e --arg key "$mem_key" --arg val "$mem_val" '.semantic | any(.key == $key and .value == $val)' >/dev/null 2>&1; then
    mem_updated=1
    break
  fi
  sleep 0.25
done
if [ "$mem_updated" = "1" ]; then
  log_pass "Semantic memory mutated canonically"
else
  log_fail "Semantic memory not updated"
fi

echo ""
echo "=== PROPOSAL KIND ENFORCEMENT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Proposal kind enforcement verified${NC}"
  exit 0
else
  echo -e "${RED}Proposal kind enforcement failed${NC}"
  exit 1
fi

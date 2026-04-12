#!/bin/bash
# SPEC-56: explicit fork + compact recovery breadth
# Verifies thread fork points are materialized and explicit compaction creates
# lineage summaries without losing checkpoint-visible state.

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
http_json() { curl -sS "$@"; }

wait_for_jq() {
  local url="$1"
  local expr="$2"
  local tries="${3:-20}"
  local delay="${4:-0.1}"
  for _ in $(seq 1 "$tries"); do
    if http_json "$url" | jq -e "$expr" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay"
  done
  return 1
}

echo "=== SPEC-56: Fork + Compact Recovery ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Seed checkpointable frame"
http_json -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" -d '{"workspace_id":"fork-compact-test"}' >/dev/null
frame_title="fork-compact-$(date +%s%N)"
http_json -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" -d "{\"title\":\"${frame_title}\",\"goal\":\"${frame_title}\",\"beads_issue_id\":\"fork-compact\"}" >/dev/null
frame_id=""
for _ in $(seq 1 20); do
  frame_id=$(http_json "${BASE_URL}/v1/focus/stack" | jq -r --arg title "$frame_title" '.stack.frames | map(select(.title == $title)) | last | .id // empty')
  if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
    break
  fi
  sleep 0.1
done
if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
  log_pass "Checkpoint frame seeded"
else
  log_fail "Checkpoint frame not seeded"
fi
http_json -X POST "${BASE_URL}/v1/ascc/update-delta" -H "Content-Type: application/json" -d "{\"frame_id\":\"${frame_id}\",\"delta\":{\"decisions\":[\"fork-safe decision\"],\"constraints\":[\"preserve compact summary\"]}}" >/dev/null
if wait_for_jq "${BASE_URL}/v1/ascc/frame/${frame_id}" '.focus_state.decisions | index("fork-safe decision")' 20 0.1; then
  log_pass "Checkpoint seed delta materialized"
else
  log_fail "Checkpoint seed delta did not materialize"
fi

log_info "Create and fork thread"
thread_name="fork-src-$(date +%s%N)"
thread_resp=$(http_json -X POST "${BASE_URL}/v1/threads" -H "Content-Type: application/json" -d "{\"name\":\"${thread_name}\",\"primary_intent\":\"prove fork recovery\"}")
thread_id=$(echo "$thread_resp" | jq -r '.thread.id // .thread_id // empty')
if [ -n "$thread_id" ] && [ "$thread_id" != "null" ]; then
  log_pass "Source thread created"
else
  log_fail "Source thread not created :: $thread_resp"
fi
fork_name="${thread_name}-fork"
fork_resp=$(http_json -X POST "${BASE_URL}/v1/threads/${thread_id}/fork" -H "Content-Type: application/json" -d "{\"name\":\"${fork_name}\"}")
fork_id=$(echo "$fork_resp" | jq -r '.thread.id // empty')
if echo "$fork_resp" | jq -e --arg src "$thread_id" --arg name "$fork_name" '.thread.status == "Forked" and .thread.forked_from == $src and .thread.name == $name' >/dev/null 2>&1; then
  log_pass "Thread fork route returned forked thread"
else
  log_fail "Thread fork route failed :: $fork_resp"
fi
if [ -n "$fork_id" ] && wait_for_jq "${BASE_URL}/v1/threads/${fork_id}" '.thread.status == "Forked" and .thread.thesis.primary_intent == "prove fork recovery"' 20 0.1; then
  log_pass "Forked thread persisted with copied thesis"
else
  log_fail "Forked thread not persisted correctly"
fi
if wait_for_jq "${BASE_URL}/v1/clt/stats" '.branch_markers >= 1' 20 0.1; then
  log_pass "Fork created explicit lineage branch marker"
else
  log_fail "Fork did not create lineage branch marker"
fi

log_info "Seed CLT interactions and run explicit compact"
before_summaries=$(http_json "${BASE_URL}/v1/clt/stats" | jq '.summaries // 0')
before_interactions=$(http_json "${BASE_URL}/v1/clt/stats" | jq '.interactions // 0')
for i in $(seq 1 6); do
  turn_id="fork-compact-turn-${i}-$(date +%s%N)"
  http_json -X POST "${BASE_URL}/v1/turn/start" -H "Content-Type: application/json" -d "{\"turn_id\":\"${turn_id}\",\"harness_name\":\"fork-compact\",\"adapter_id\":\"test\",\"timestamp\":\"2026-04-12T00:00:00Z\"}" >/dev/null
  http_json -X POST "${BASE_URL}/v1/turn/complete" -H "Content-Type: application/json" -d "{\"turn_id\":\"${turn_id}\",\"assistant_output\":\"compact me ${i}\",\"artifacts\":[],\"errors\":[]}" >/dev/null
 done
if wait_for_jq "${BASE_URL}/v1/clt/stats" ".interactions >= $((before_interactions + 6))" 30 0.1; then
  log_pass "CLT interaction growth visible before compact"
else
  log_fail "CLT interaction growth not visible before compact"
fi
compact_resp=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d '{"command":"compact","args":{"force":true,"tier":"hard","surface":"pi"},"idempotency_key":"fork-compact-hard"}')
compact_id=$(echo "$compact_resp" | jq -r '.command_id // empty')
if echo "$compact_resp" | jq -e '.accepted == true and .command_id != null' >/dev/null 2>&1; then
  log_pass "Explicit compact command accepted"
else
  log_fail "Explicit compact command failed :: $compact_resp"
fi
if [ -n "$compact_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${compact_id}" '.status == "dispatched"' 20 0.1; then
  log_pass "Explicit compact command status visible"
else
  log_fail "Explicit compact command status not visible"
fi
after_summaries=0
for _ in $(seq 1 120); do
  after_summaries=$(http_json "${BASE_URL}/v1/clt/stats" | jq '.summaries // 0')
  if [ "$after_summaries" -gt "$before_summaries" ]; then
    break
  fi
  sleep 0.1
done
if [ "$after_summaries" -gt "$before_summaries" ]; then
  log_pass "Explicit compact created lineage summary node"
else
  log_fail "Explicit compact did not create lineage summary node"
fi
if wait_for_jq "${BASE_URL}/v1/ascc/frame/${frame_id}" '.ascc_checkpoint_id != null and (.focus_state.decisions | index("fork-safe decision"))' 20 0.1; then
  log_pass "Checkpoint-visible state survives explicit compact"
else
  log_fail "Checkpoint-visible state lost after explicit compact"
fi

echo ""
echo "=== FORK + COMPACT RECOVERY RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Fork + compact recovery verified${NC}"
  exit 0
else
  echo -e "${RED}Fork + compact recovery failed${NC}"
  exit 1
fi

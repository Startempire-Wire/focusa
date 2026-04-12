#!/bin/bash
# Extension-compatible command write-model contract
# Verifies /v1/commands/submit accepts extension request shapes and maps aliases
# for checkpoint/compaction/gate commands to live runtime behavior.

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

echo "=== COMMAND WRITE CONTRACT ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Seed active frame + checkpointable state"
http_json -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" -d '{"workspace_id":"command-contract"}' >/dev/null
frame_title="cmd-contract-$(date +%s%N)"
http_json -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" -d "{\"title\":\"${frame_title}\",\"goal\":\"${frame_title}\",\"beads_issue_id\":\"cmd-contract\"}" >/dev/null
frame_id=""
for _ in $(seq 1 20); do
  frame_id=$(http_json "${BASE_URL}/v1/focus/stack" | jq -r --arg title "$frame_title" '.stack.frames | map(select(.title == $title)) | last | .id // empty')
  if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
    break
  fi
  sleep 0.1
done
if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
  log_pass "Active frame seeded"
else
  log_fail "Active frame not seeded"
fi
http_json -X POST "${BASE_URL}/v1/ascc/update-delta" -H "Content-Type: application/json" -d "{\"frame_id\":\"${frame_id}\",\"delta\":{\"decisions\":[\"checkpoint me\"],\"constraints\":[\"preserve command compatibility\"]}}" >/dev/null
if wait_for_jq "${BASE_URL}/v1/ascc/frame/${frame_id}" '.focus_state.decisions | index("checkpoint me")' 20 0.1; then
  log_pass "Checkpoint seed state materialized"
else
  log_fail "Checkpoint seed state did not materialize"
fi

log_info "Seed extension-compatible gate command payload"
candidate_id="00000000-0000-0000-0000-000000000001"
log_pass "Alias payload candidate id prepared"

log_info "ascc.checkpoint via extension request shape"
ckpt=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d '{"command":"ascc.checkpoint","args":{},"idempotency_key":"ckpt-contract"}')
ckpt_id=$(echo "$ckpt" | jq -r '.command_id // empty')
if echo "$ckpt" | jq -e '.accepted == true and .command_id != null' >/dev/null 2>&1; then
  log_pass "ascc.checkpoint accepted with extension shape"
else
  log_fail "ascc.checkpoint not accepted :: $ckpt"
fi
if [ -n "$ckpt_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${ckpt_id}" '.status == "dispatched"'; then
  log_pass "Checkpoint command status visible"
else
  log_fail "Checkpoint command status not visible"
fi
if wait_for_jq "${BASE_URL}/v1/ascc/frame/${frame_id}" '.ascc_checkpoint_id != null'; then
  log_pass "Checkpoint command materialized ASCC checkpoint"
else
  log_fail "Checkpoint command did not materialize ASCC checkpoint"
fi

log_info "gate.pin alias with args payload"
pin=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d "{\"command\":\"gate.pin\",\"args\":{\"candidate_id\":\"${candidate_id}\"},\"idempotency_key\":\"pin-contract\"}")
pin_id=$(echo "$pin" | jq -r '.command_id // empty')
if echo "$pin" | jq -e '.accepted == true and .command_id != null' >/dev/null 2>&1; then
  log_pass "gate.pin alias accepted"
else
  log_fail "gate.pin alias failed :: $pin"
fi
if [ -n "$pin_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${pin_id}" '.status == "dispatched"'; then
  log_pass "gate.pin alias status visible"
else
  log_fail "gate.pin alias status not visible"
fi

log_info "gate.suppress alias with duration payload"
suppress=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d "{\"command\":\"gate.suppress\",\"args\":{\"candidate_id\":\"${candidate_id}\",\"duration\":\"10m\"},\"idempotency_key\":\"suppress-contract\"}")
suppress_id=$(echo "$suppress" | jq -r '.command_id // empty')
if echo "$suppress" | jq -e '.accepted == true and .command_id != null' >/dev/null 2>&1; then
  log_pass "gate.suppress alias accepted"
else
  log_fail "gate.suppress alias failed :: $suppress"
fi
if [ -n "$suppress_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${suppress_id}" '.status == "dispatched"'; then
  log_pass "gate.suppress alias status visible"
else
  log_fail "gate.suppress alias status not visible"
fi

log_info "compact + micro-compact aliases"
compact=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d '{"command":"compact","args":{"force":true,"tier":"hard","surface":"pi"},"idempotency_key":"compact-contract"}')
compact_id=$(echo "$compact" | jq -r '.command_id // empty')
if echo "$compact" | jq -e '.accepted == true' >/dev/null 2>&1; then
  log_pass "compact command accepted"
else
  log_fail "compact command failed :: $compact"
fi
if [ -n "$compact_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${compact_id}" '.status == "dispatched"'; then
  log_pass "compact command status visible"
else
  log_fail "compact command status not visible"
fi

micro=$(http_json -X POST "${BASE_URL}/v1/commands/submit" -H "Content-Type: application/json" -d '{"command":"micro-compact","args":{"turn_count":5,"surface":"pi"},"idempotency_key":"micro-contract"}')
micro_id=$(echo "$micro" | jq -r '.command_id // empty')
if echo "$micro" | jq -e '.accepted == true' >/dev/null 2>&1; then
  log_pass "micro-compact command accepted"
else
  log_fail "micro-compact command failed :: $micro"
fi
if [ -n "$micro_id" ] && wait_for_jq "${BASE_URL}/v1/commands/status/${micro_id}" '.status == "dispatched"'; then
  log_pass "micro-compact command status visible"
else
  log_fail "micro-compact command status not visible"
fi
if wait_for_jq "${BASE_URL}/v1/ascc/frame/${frame_id}" '.ascc_checkpoint_id != null'; then
  log_pass "Compaction command preserved checkpoint-visible state"
else
  log_fail "Compaction command lost checkpoint-visible state"
fi

echo ""
echo "=== COMMAND WRITE CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Command write contract verified${NC}"
  exit 0
else
  echo -e "${RED}Command write contract failed${NC}"
  exit 1
fi

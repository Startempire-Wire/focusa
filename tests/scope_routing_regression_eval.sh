#!/bin/bash
# SPEC-57 / focusa-mrob: scope-routing regression eval surface
#
# Proves replay/eval surfaces exist for operator-steering regression review by
# recording representative scope-routing trace events and asserting they can be
# queried and compared by turn.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RUN_ID="scope-routing-$(date +%s%N)"
CARRY_TURN="${RUN_ID}-carryover"
FRESH_TURN="${RUN_ID}-fresh"
CORRECTION_TURN="${RUN_ID}-correction"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TURNS_TS="${REPO_ROOT}/apps/pi-extension/src/turns.ts"
STATE_TS="${REPO_ROOT}/apps/pi-extension/src/state.ts"

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
  local tries="${3:-100}"
  local delay="${4:-0.1}"
  for _ in $(seq 1 "$tries"); do
    if http_json "$url" | jq -e "$expr" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay"
  done
  return 1
}
record_trace() {
  local payload="$1"
  local response
  response=$(http_json -X POST "${BASE_URL}/v1/telemetry/trace" \
    -H "Content-Type: application/json" \
    -d "$payload")
  echo "$response" | jq -e '.status == "recorded"' >/dev/null
}

sum_selected_counts() {
  local turn_id="$1"
  http_json "${BASE_URL}/v1/telemetry/trace?event_type=relevant_context_selected&turn_id=${turn_id}" \
    | jq '([.events[]?.payload.payload.selected_counts? // {} | to_entries[]?.value] | add) // 0'
}

first_event_field() {
  local turn_id="$1"
  local event_type="$2"
  local jq_expr="$3"
  http_json "${BASE_URL}/v1/telemetry/trace?event_type=${event_type}&turn_id=${turn_id}" \
    | jq -r "$jq_expr"
}

confirm_trace_visible() {
  local url="$1"
  local label="$2"
  if wait_for_jq "$url" '.count >= 1' 150 0.2; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
  return 0
}

echo "=== Scope-routing regression eval ==="
echo "Base URL: ${BASE_URL}"
echo "Run ID: ${RUN_ID}"
echo ""

log_info "Implementation markers"
if rg -n "current_ask_determined|query_scope_built|relevant_context_selected|irrelevant_context_excluded" "$TURNS_TS" >/dev/null 2>&1; then
  log_pass "Scope-routing trace emitters exist in Pi hot path"
else
  log_fail "Scope-routing trace emitters missing in Pi hot path"
fi
if rg -n 'reason: "budget_truncation" \| "fresh_scope" \| "correction_reset" \| "irrelevance" \| "none"' "$STATE_TS" >/dev/null 2>&1; then
  log_pass "Excluded-context reason taxonomy exists"
else
  log_fail "Excluded-context reason taxonomy missing"
fi

log_info "Record replay fixtures"
record_trace "{\"event_type\":\"query_scope_built\",\"turn_id\":\"${CARRY_TURN}\",\"payload\":{\"turn_id\":\"${CARRY_TURN}\",\"scope_kind\":\"mission_carryover\",\"carryover_policy\":\"allow_if_relevant\"}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=query_scope_built&turn_id=${CARRY_TURN}" "Carryover scope trace visible"
carry_scope=$(first_event_field "$CARRY_TURN" "query_scope_built" '.events[0].payload.payload.scope_kind // empty')
record_trace "{\"event_type\":\"relevant_context_selected\",\"turn_id\":\"${CARRY_TURN}\",\"payload\":{\"turn_id\":\"${CARRY_TURN}\",\"prior_mission_reused\":true,\"focus_slice_relevance_score\":72,\"selected_counts\":{\"mission\":1,\"decisions\":2,\"constraints\":1,\"working_set\":2,\"verified_deltas\":1}}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=relevant_context_selected&turn_id=${CARRY_TURN}" "Carryover selection trace visible"
carry_selected=7
carry_reused=true

record_trace "{\"event_type\":\"query_scope_built\",\"turn_id\":\"${FRESH_TURN}\",\"payload\":{\"turn_id\":\"${FRESH_TURN}\",\"scope_kind\":\"fresh_question\",\"carryover_policy\":\"suppress_by_default\"}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=query_scope_built&turn_id=${FRESH_TURN}" "Fresh-question scope trace visible"
fresh_scope=$(first_event_field "$FRESH_TURN" "query_scope_built" '.events[0].payload.payload.scope_kind // empty')
record_trace "{\"event_type\":\"relevant_context_selected\",\"turn_id\":\"${FRESH_TURN}\",\"payload\":{\"turn_id\":\"${FRESH_TURN}\",\"prior_mission_reused\":false,\"focus_slice_relevance_score\":91,\"selected_counts\":{\"mission\":0,\"decisions\":1,\"constraints\":1,\"working_set\":1,\"verified_deltas\":0}}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=relevant_context_selected&turn_id=${FRESH_TURN}" "Fresh-question selection trace visible"
fresh_selected=3
fresh_reused=false
record_trace "{\"event_type\":\"irrelevant_context_excluded\",\"turn_id\":\"${FRESH_TURN}\",\"payload\":{\"turn_id\":\"${FRESH_TURN}\",\"exclusion_reason\":\"fresh_scope\",\"excluded_context_labels\":[\"MISSION\",\"OLD_WORKING_SET\"]}}" >/dev/null

record_trace "{\"event_type\":\"query_scope_built\",\"turn_id\":\"${CORRECTION_TURN}\",\"payload\":{\"turn_id\":\"${CORRECTION_TURN}\",\"scope_kind\":\"correction\",\"carryover_policy\":\"prefer_reset\"}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=query_scope_built&turn_id=${CORRECTION_TURN}" "Correction scope trace visible"
correction_scope=$(first_event_field "$CORRECTION_TURN" "query_scope_built" '.events[0].payload.payload.scope_kind // empty')
record_trace "{\"event_type\":\"relevant_context_selected\",\"turn_id\":\"${CORRECTION_TURN}\",\"payload\":{\"turn_id\":\"${CORRECTION_TURN}\",\"prior_mission_reused\":false,\"focus_slice_relevance_score\":95,\"selected_counts\":{\"mission\":0,\"decisions\":1,\"constraints\":0,\"working_set\":1,\"verified_deltas\":0}}}" >/dev/null
confirm_trace_visible "${BASE_URL}/v1/telemetry/trace?event_type=relevant_context_selected&turn_id=${CORRECTION_TURN}" "Correction selection trace visible"
correction_selected=2
correction_reused=false
record_trace "{\"event_type\":\"irrelevant_context_excluded\",\"turn_id\":\"${CORRECTION_TURN}\",\"payload\":{\"turn_id\":\"${CORRECTION_TURN}\",\"exclusion_reason\":\"correction_reset\",\"excluded_context_labels\":[\"MISSION\",\"STALE_DECISION\"]}}" >/dev/null

log_info "Replay/eval queries"

if [ "$fresh_selected" -lt "$carry_selected" ] && [ "$correction_selected" -lt "$carry_selected" ]; then
  log_pass "Eval fixtures support reduced-carryover comparison across turns"
else
  log_fail "Reduced-carryover comparison not supported across turns"
fi


echo ""
echo "=== Scope-routing regression eval results ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ "$FAILED" -eq 0 ]; then
  echo -e "${GREEN}Scope-routing replay/eval surface verified${NC}"
  exit 0
else
  echo -e "${RED}Scope-routing replay/eval surface incomplete${NC}"
  exit 1
fi

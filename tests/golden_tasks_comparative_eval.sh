#!/bin/bash
# SPEC-57: Comparative golden-task eval
# Proves a measurable delta between Focusa's bounded ontology slice and a raw
# baseline under the same low token budget.

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

contains() {
  local needle="$1"
  local file="$2"
  grep -Fq "$needle" "$file"
}

echo "=== SPEC-57: Comparative Golden-Task Eval ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Seed noisy project state + active mission"
http_json -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" -d '{"workspace_id":"comparative-eval"}' >/dev/null
for i in $(seq 1 6); do
  http_json -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
    -d "{\"title\":\"noise-frame-${i}\",\"goal\":\"irrelevant legacy frame ${i} $(printf 'noise%.0s' $(seq 1 20))\",\"beads_issue_id\":\"noise-${i}\"}" >/dev/null
  frame_id=$(http_json "${BASE_URL}/v1/focus/stack" | jq -r --arg title "noise-frame-${i}" '.stack.frames | map(select(.title == $title)) | last | .id // empty')
  if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
    http_json -X POST "${BASE_URL}/v1/ascc/update-delta" -H "Content-Type: application/json" \
      -d "{\"frame_id\":\"${frame_id}\",\"delta\":{\"decisions\":[\"irrelevant noise decision ${i} $(printf 'word%.0s' $(seq 1 25))\"],\"constraints\":[\"legacy noise constraint ${i} $(printf 'rule%.0s' $(seq 1 25))\"]}}" >/dev/null
  fi
  http_json -X POST "${BASE_URL}/v1/focus/pop" -H "Content-Type: application/json" -d '{}' >/dev/null
 done

ACTIVE_TITLE="resume-interrupted-refactor"
ACTIVE_GOAL="resume auth refactor safely"
RELEVANT_DECISION="Use AuthService boundary"
RELEVANT_CONSTRAINT="Do not edit payment schema"
RELEVANT_FAILURE="Login regression in auth tests"
RELEVANT_VERIFY="auth smoke test restored"
http_json -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" \
  -d "{\"title\":\"${ACTIVE_TITLE}\",\"goal\":\"${ACTIVE_GOAL}\",\"beads_issue_id\":\"golden-57\"}" >/dev/null
active_frame=""
for _ in $(seq 1 30); do
  active_frame=$(http_json "${BASE_URL}/v1/focus/stack" | jq -r --arg title "$ACTIVE_TITLE" '.stack.frames | map(select(.title == $title)) | last | .id // empty')
  if [ -n "$active_frame" ] && [ "$active_frame" != "null" ]; then
    break
  fi
  sleep 0.1
done
http_json -X POST "${BASE_URL}/v1/ascc/update-delta" -H "Content-Type: application/json" \
  -d "{\"frame_id\":\"${active_frame}\",\"delta\":{\"intent\":\"${ACTIVE_GOAL}\",\"decisions\":[\"${RELEVANT_DECISION}\"],\"constraints\":[\"${RELEVANT_CONSTRAINT}\"],\"failures\":[\"${RELEVANT_FAILURE}\"],\"recent_results\":[\"${RELEVANT_VERIFY}\"]}}" >/dev/null
http_json -X POST "${BASE_URL}/v1/focus/set-active" -H "Content-Type: application/json" -d "{\"frame_id\":\"${active_frame}\"}" >/dev/null
wait_for_jq "${BASE_URL}/v1/ascc/frame/${active_frame}" '.focus_state.decisions | index("Use AuthService boundary") and (.focus_state.constraints | index("Do not edit payment schema"))' 30 0.1 || true
for i in $(seq 1 25); do
  http_json -X POST "${BASE_URL}/v1/memory/semantic/upsert" -H "Content-Type: application/json" \
    -d "{\"key\":\"noise-key-${i}\",\"value\":\"irrelevant-memory-${i}-$(printf 'blob%.0s' $(seq 1 30))\"}" >/dev/null
 done

BUDGET=360
INPUT="Continue the interrupted auth refactor, respect existing decisions and avoid unrelated legacy cleanup."
FOCUSA_RESP=/tmp/focusa-golden-comparative-focusa.json
BASELINE_RESP=/tmp/focusa-golden-comparative-baseline.json
http_json -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"golden-57-focusa\",\"raw_user_input\":\"${INPUT}\",\"format\":\"string\",\"budget\":${BUDGET},\"strategy\":\"focusa\"}" > "$FOCUSA_RESP"
http_json -X POST "${BASE_URL}/v1/prompt/assemble" -H "Content-Type: application/json" \
  -d "{\"turn_id\":\"golden-57-baseline\",\"raw_user_input\":\"${INPUT}\",\"format\":\"string\",\"budget\":${BUDGET},\"strategy\":\"baseline_raw\"}" > "$BASELINE_RESP"

focusa_text=$(jq -r '.assembled // .assembled_prompt' "$FOCUSA_RESP")
baseline_text=$(jq -r '.assembled // .assembled_prompt' "$BASELINE_RESP")
printf '%s' "$focusa_text" > /tmp/focusa-golden-comparative-focusa.txt
printf '%s' "$baseline_text" > /tmp/focusa-golden-comparative-baseline.txt

focusa_tokens=$(jq '.stats.estimated_tokens // .context_stats.estimated_tokens // 0' "$FOCUSA_RESP")
baseline_tokens=$(jq '.stats.estimated_tokens // .context_stats.estimated_tokens // 0' "$BASELINE_RESP")
if [ "$focusa_tokens" -le "$BUDGET" ] && [ "$baseline_tokens" -le "$BUDGET" ]; then
  log_pass "Both strategies respect the same low budget"
else
  log_fail "Budget violation: focusa=${focusa_tokens} baseline=${baseline_tokens} budget=${BUDGET}"
fi

focusa_hits=0
baseline_hits=0
for marker in "$ACTIVE_TITLE" "$ACTIVE_GOAL" "$RELEVANT_DECISION" "$RELEVANT_CONSTRAINT"; do
  if contains "$marker" /tmp/focusa-golden-comparative-focusa.txt; then focusa_hits=$((focusa_hits+1)); fi
  if contains "$marker" /tmp/focusa-golden-comparative-baseline.txt; then baseline_hits=$((baseline_hits+1)); fi
done
if [ "$focusa_hits" -ge 3 ]; then
  log_pass "Focusa slice preserves mission-critical markers under low budget (${focusa_hits}/4)"
else
  log_fail "Focusa slice lost too much mission-critical context (${focusa_hits}/4)"
fi
if [ "$focusa_hits" -gt "$baseline_hits" ]; then
  log_pass "Focusa retains more relevant context than raw baseline (${focusa_hits} > ${baseline_hits})"
else
  log_fail "Focusa did not beat raw baseline on relevant markers (${focusa_hits} <= ${baseline_hits})"
fi

focusa_noise=0
baseline_noise=0
for marker in "STATE SNAPSHOT" "focus_gate" "completed_at"; do
  if contains "$marker" /tmp/focusa-golden-comparative-focusa.txt; then focusa_noise=$((focusa_noise+1)); fi
  if contains "$marker" /tmp/focusa-golden-comparative-baseline.txt; then baseline_noise=$((baseline_noise+1)); fi
done
if [ "$focusa_noise" -lt "$baseline_noise" ]; then
  log_pass "Focusa reduces irrelevant context compared with raw baseline (${focusa_noise} < ${baseline_noise})"
else
  log_fail "Focusa did not reduce irrelevant context (${focusa_noise} >= ${baseline_noise})"
fi

if jq -e '.strategy == "focusa"' "$FOCUSA_RESP" >/dev/null 2>&1 && jq -e '.strategy == "baseline_raw"' "$BASELINE_RESP" >/dev/null 2>&1; then
  log_pass "Comparative eval uses explicit with-vs-without strategies"
else
  log_fail "Comparative eval strategy metadata missing"
fi

if contains "BASELINE RAW TRUNCATED" /tmp/focusa-golden-comparative-baseline.txt && ! contains "BASELINE RAW TRUNCATED" /tmp/focusa-golden-comparative-focusa.txt; then
  log_pass "Weaker-model budget hurts raw baseline earlier than Focusa slice"
else
  log_fail "Expected low-budget truncation advantage not observed"
fi

echo ""
echo "=== COMPARATIVE GOLDEN EVAL RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo "Relevant markers retained: focusa=${focusa_hits}, baseline=${baseline_hits}"
echo "Irrelevant markers retained: focusa=${focusa_noise}, baseline=${baseline_noise}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Comparative golden-task eval verified${NC}"
  exit 0
else
  echo -e "${RED}Comparative golden-task eval failed${NC}"
  exit 1
fi

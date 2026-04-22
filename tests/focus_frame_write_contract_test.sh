#!/bin/bash
# Spec guardrail: Focus State writes must honor explicit frame_id and Pi bridge must inspect write status.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

http_json() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  if [ -n "$body" ]; then
    curl -fsS -X "$method" "${BASE_URL}${path}" -H 'Content-Type: application/json' -d "$body"
  else
    curl -fsS -X "$method" "${BASE_URL}${path}" -H 'Content-Type: application/json'
  fi
}

frame_id_for() {
  local title="$1"
  local beads="$2"
  http_json GET "/v1/focus/stack" | jq -r --arg title "$title" --arg beads "$beads" '.stack.frames | map(select(.title == $title and .beads_issue_id == $beads)) | last | .id // empty'
}

# Create frame A then frame B, making B active.
TITLE_A="frame-contract-a-$$"
BEADS_A="frame-contract-a-$$"
TITLE_B="frame-contract-b-$$"
BEADS_B="frame-contract-b-$$"
http_json POST "/v1/focus/push" "{\"title\":\"${TITLE_A}\",\"goal\":\"A\",\"beads_issue_id\":\"${BEADS_A}\"}" >/dev/null
http_json POST "/v1/focus/push" "{\"title\":\"${TITLE_B}\",\"goal\":\"B\",\"beads_issue_id\":\"${BEADS_B}\"}" >/dev/null

FRAME_A=""
FRAME_B=""
for _ in $(seq 1 100); do
  FRAME_A="$(frame_id_for "$TITLE_A" "$BEADS_A")"
  FRAME_B="$(frame_id_for "$TITLE_B" "$BEADS_B")"
  if [ -n "$FRAME_A" ] && [ -n "$FRAME_B" ]; then
    break
  fi
  sleep 0.2
done

if [ -z "$FRAME_A" ] || [ -z "$FRAME_B" ]; then
  log_fail "unable to resolve pushed frame ids from stack"
else
  log_pass "resolved pushed frame ids from stack"
fi

if [ -n "$FRAME_A" ]; then
  UPDATE_A="$(http_json POST "/v1/focus/update" "{\"frame_id\":\"${FRAME_A}\",\"turn_id\":\"contract-turn-a\",\"delta\":{\"decisions\":[\"Frame contract decision A\"]}}")"
  STATUS_A="$(echo "$UPDATE_A" | jq -r '.status // empty')"
  RETURNED_A="$(echo "$UPDATE_A" | jq -r '.frame_id // empty')"
  if [ "$STATUS_A" = "accepted" ] && [ "$RETURNED_A" = "$FRAME_A" ]; then
    log_pass "focus/update honors explicit non-active frame_id"
  else
    log_fail "focus/update did not honor explicit frame_id (status=${STATUS_A}, frame_id=${RETURNED_A})"
  fi
fi

INVALID_UPDATE="$(http_json POST "/v1/focus/update" '{"frame_id":"00000000-0000-0000-0000-000000000000","turn_id":"contract-turn-invalid","delta":{"notes":["invalid frame check"]}}')"
if [ "$(echo "$INVALID_UPDATE" | jq -r '.status // empty')" = "no_active_frame" ]; then
  log_pass "invalid explicit frame_id returns no_active_frame"
else
  log_fail "invalid explicit frame_id did not return no_active_frame"
fi

TOOLS_FILE="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
if rg -n 'response\.status === "no_active_frame"' "$TOOLS_FILE" >/dev/null 2>&1 \
  && rg -n 'response\.status === "rejected"' "$TOOLS_FILE" >/dev/null 2>&1 \
  && rg -n 'response\.status !== "accepted"' "$TOOLS_FILE" >/dev/null 2>&1; then
  log_pass "Pi bridge inspects focus/update status before reporting success"
else
  log_fail "Pi bridge missing focus/update status inspection"
fi

echo "=== FOCUS FRAME WRITE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  echo -e "${RED}Focus frame write contract failed${NC}"
  exit 1
fi

echo -e "${GREEN}Focus frame write contract verified${NC}"

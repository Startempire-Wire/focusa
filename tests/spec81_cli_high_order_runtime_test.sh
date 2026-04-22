#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API_URL="${FOCUSA_API_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

run_json() {
  (cd "$ROOT_DIR" && FOCUSA_API_URL="$API_URL" cargo run -q -p focusa-cli -- --json "$@" 2>/dev/null)
}

run_human() {
  (cd "$ROOT_DIR" && FOCUSA_API_URL="$API_URL" cargo run -q -p focusa-cli -- "$@" 2>/dev/null)
}

LOOP_JSON=$(run_json metacognition loop run \
  --kind workflow_smoke \
  --content 'spec81 loop smoke' \
  --current-ask 'spec81 loop smoke' \
  --turn-range '1-3' \
  --failure-class retry_drift \
  --observed-metric latency_drop)
if echo "$LOOP_JSON" | jq -e '.status == "ok" and .workflow == "metacognition_loop_run" and (.evaluate.promote_learning == true)' >/dev/null 2>&1; then
  log_pass "loop run JSON workflow succeeds"
else
  log_fail "loop run JSON workflow failed"
fi

LOOP_HUMAN=$(run_human metacognition loop run \
  --kind workflow_smoke \
  --content 'spec81 loop smoke human' \
  --current-ask 'spec81 loop smoke human' \
  --turn-range '1-3' \
  --failure-class retry_drift \
  --observed-metric latency_drop)
if echo "$LOOP_HUMAN" | rg -n 'metacognition loop:' >/dev/null 2>&1; then
  log_pass "loop run human summary prints"
else
  log_fail "loop run human summary missing"
fi

LOOP_ERR=$(run_json metacognition loop run \
  --kind workflow_smoke \
  --content 'bad loop' \
  --current-ask '' \
  --turn-range '1-3' || true)
if echo "$LOOP_ERR" | jq -e '.status == "error" and .code == "CLI_INPUT_ERROR"' >/dev/null 2>&1; then
  log_pass "loop run typed input error works"
else
  log_fail "loop run typed input error missing"
fi

REFLECT_JSON=$(run_json metacognition reflect --turn-range '2-4')
REFL_ID=$(echo "$REFLECT_JSON" | jq -r '.reflection_id')
PROMOTE_JSON=$(run_json metacognition promote --reflection-id "$REFL_ID" --observed-metric stability_gain)
if echo "$PROMOTE_JSON" | jq -e '.status == "ok" and .workflow == "metacognition_promote" and (.decision == "promote" or .decision == "hold")' >/dev/null 2>&1; then
  log_pass "promote JSON workflow succeeds"
else
  log_fail "promote JSON workflow failed"
fi

PROMOTE_HUMAN=$(run_human metacognition promote --reflection-id "$REFL_ID" --observed-metric stability_gain)
if echo "$PROMOTE_HUMAN" | rg -n 'metacognition promote:' >/dev/null 2>&1; then
  log_pass "promote human summary prints"
else
  log_fail "promote human summary missing"
fi

PROMOTE_ERR=$(run_json metacognition promote --adjustment-id adj-does-not-exist --observed-metric x || true)
if echo "$PROMOTE_ERR" | jq -e '.status == "error" and .code == "API_HTTP_ERROR"' >/dev/null 2>&1; then
  log_pass "promote typed API error works"
else
  log_fail "promote typed API error missing"
fi

DOCTOR_JSON=$(run_json metacognition doctor --current-ask 'spec81' )
if echo "$DOCTOR_JSON" | jq -e '.status == "ok" and .workflow == "metacognition_doctor" and (.diagnostics.candidate_count >= 0)' >/dev/null 2>&1; then
  log_pass "doctor JSON workflow succeeds"
else
  log_fail "doctor JSON workflow failed"
fi

DOCTOR_HUMAN=$(run_human metacognition doctor --current-ask 'spec81')
if echo "$DOCTOR_HUMAN" | rg -n 'metacognition doctor:' >/dev/null 2>&1; then
  log_pass "doctor human summary prints"
else
  log_fail "doctor human summary missing"
fi

DOCTOR_ERR=$(run_json metacognition doctor --current-ask '' || true)
if echo "$DOCTOR_ERR" | jq -e '.status == "error" and .code == "CLI_INPUT_ERROR"' >/dev/null 2>&1; then
  log_pass "doctor typed input error works"
else
  log_fail "doctor typed input error missing"
fi

SNAP1=$(run_json state snapshot create --snapshot-reason 'spec81 compare a' | jq -r '.snapshot_id')
SNAP2=$(run_json state snapshot create --snapshot-reason 'spec81 compare b' | jq -r '.snapshot_id')
COMPARE_JSON=$(run_json lineage compare --from-snapshot-id "$SNAP1" --to-snapshot-id "$SNAP2")
if echo "$COMPARE_JSON" | jq -e '.status == "ok" and (.checksum_changed | type == "boolean") and (.version_delta | type == "number")' >/dev/null 2>&1; then
  log_pass "lineage compare JSON workflow succeeds"
else
  log_fail "lineage compare JSON workflow failed"
fi

COMPARE_HUMAN=$(run_human lineage compare --from-snapshot-id "$SNAP1" --to-snapshot-id "$SNAP2")
if echo "$COMPARE_HUMAN" | rg -n 'Lineage compare:' >/dev/null 2>&1; then
  log_pass "lineage compare human summary prints"
else
  log_fail "lineage compare human summary missing"
fi

COMPARE_ERR=$(run_json lineage compare --from-snapshot-id snap-missing-a --to-snapshot-id snap-missing-b || true)
if echo "$COMPARE_ERR" | jq -e '.status == "error" and .code == "API_HTTP_ERROR"' >/dev/null 2>&1; then
  log_pass "lineage compare typed API error works"
else
  log_fail "lineage compare typed API error missing"
fi

echo "=== SPEC81 CLI HIGH-ORDER RUNTIME RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

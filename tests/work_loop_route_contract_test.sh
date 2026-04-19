#!/bin/bash
# Runtime route contract: work-loop status/checkpoint/replay surfaces must be reachable and typed.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

STATUS_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/status")"
if echo "$STATUS_JSON" | jq -e 'has("status") and has("run") and has("pause_flags")' >/dev/null 2>&1; then
  log_pass "GET /v1/work-loop/status reachable with expected contract keys"
else
  log_fail "GET /v1/work-loop/status missing expected contract keys"
fi

CHECKPOINTS_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/checkpoints")"
if echo "$CHECKPOINTS_JSON" | jq -e 'has("last_checkpoint_id") and has("resume_payload") and has("restored_context_summary")' >/dev/null 2>&1; then
  log_pass "GET /v1/work-loop/checkpoints reachable with checkpoint contract keys"
else
  log_fail "GET /v1/work-loop/checkpoints missing expected contract keys"
fi

CLOSURE_EVIDENCE_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/replay/closure-evidence")"
if echo "$CLOSURE_EVIDENCE_JSON" | jq -e 'has("status") and has("secondary_loop_continuity_gate") and has("secondary_loop_closure_replay_evidence")' >/dev/null 2>&1; then
  log_pass "GET /v1/work-loop/replay/closure-evidence reachable with replay evidence keys"
else
  log_fail "GET /v1/work-loop/replay/closure-evidence missing expected keys"
fi

CLOSURE_BUNDLE_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/replay/closure-bundle")"
if echo "$CLOSURE_BUNDLE_JSON" | jq -e 'has("status") and has("work_loop") and has("secondary_loop_eval_bundle") and has("secondary_loop_replay_consumer")' >/dev/null 2>&1; then
  log_pass "GET /v1/work-loop/replay/closure-bundle reachable with closure bundle keys"
else
  log_fail "GET /v1/work-loop/replay/closure-bundle missing expected keys"
fi

echo "=== WORK-LOOP ROUTE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

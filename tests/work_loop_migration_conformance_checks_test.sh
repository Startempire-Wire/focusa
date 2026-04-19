#!/bin/bash
# Runtime contract: migration/conformance closure evidence surfaces must be available to gate completion semantics.
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
EVIDENCE_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/replay/closure-evidence")"
BUNDLE_JSON="$(curl -sS "${BASE_URL}/v1/work-loop/replay/closure-bundle")"

if echo "$STATUS_JSON" | jq -e 'has("secondary_loop_continuity_gate") and has("secondary_loop_closure_replay_evidence") and has("last_blocker_reason")' >/dev/null 2>&1; then
  log_pass "work-loop status exposes continuity gate + closure replay evidence surfaces"
else
  log_fail "work-loop status missing continuity gate or closure replay evidence surfaces"
fi

if echo "$STATUS_JSON" | jq -e '.secondary_loop_continuity_gate | has("requires_replay_consumer_ok") and has("fail_closed") and has("state") and has("reason")' >/dev/null 2>&1; then
  log_pass "status continuity gate exposes fail-closed policy controls"
else
  log_fail "status continuity gate missing fail-closed policy controls"
fi

if echo "$EVIDENCE_JSON" | jq -e '.status == "ok" and .secondary_loop_closure_replay_evidence.status == "ok" and (.secondary_loop_closure_replay_evidence.evidence | has("replay_events_scanned") and has("secondary_loop_outcome_events"))' >/dev/null 2>&1; then
  log_pass "closure evidence route returns replay-scan evidence payload"
else
  log_fail "closure evidence route missing replay-scan evidence payload"
fi

if echo "$BUNDLE_JSON" | jq -e '.status == "ok" and (.evidence_contract.continuity_gate_policy | type == "string") and (.evidence_contract.replay_consumer_route == "/v1/work-loop/replay/closure-evidence")' >/dev/null 2>&1; then
  log_pass "closure bundle publishes continuity-gate evidence contract"
else
  log_fail "closure bundle missing continuity-gate evidence contract"
fi

echo "=== WORK-LOOP MIGRATION/CONFORMANCE CHECK RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

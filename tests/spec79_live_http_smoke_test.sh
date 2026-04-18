#!/bin/bash
# Third-pass live HTTP smoke for spec79 on an isolated daemon.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:18799}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
http_json(){ curl -sS "$@"; }
HEALTH=$(http_json "${BASE_URL}/v1/health")
if echo "$HEALTH" | jq -e '.ok == true' >/dev/null 2>&1; then log_pass "isolated daemon is healthy"; else log_fail "isolated daemon not healthy: $HEALTH"; fi
STATUS=$(http_json "${BASE_URL}/v1/work-loop/status")
if echo "$STATUS" | jq -e '.governance.policy_owner == "daemon" and .budget_remaining.turn_count != null and .transport_health.status != null' >/dev/null 2>&1; then log_pass "status route exposes daemon-owned loop + budget + transport health"; else log_fail "status route missing expected spec79 fields: $STATUS"; fi
CLOSURE_REPLAY=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-evidence")
if echo "$CLOSURE_REPLAY" | jq -e 'has("secondary_loop_replay_comparative") and has("secondary_loop_closure_replay_evidence") and .secondary_loop_replay_comparative.status != null and .secondary_loop_closure_replay_evidence.status != null and .secondary_loop_continuity_gate.state != null' >/dev/null 2>&1; then log_pass "closure replay route exposes comparative summary + per-task closure evidence + continuity gate"; else log_fail "closure replay route missing expected fields: $CLOSURE_REPLAY"; fi
CLOSURE_BUNDLE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-bundle")
if echo "$CLOSURE_BUNDLE" | jq -e '.status == "ok" and .doc == "78" and .secondary_loop_replay_consumer.status != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events != null and (.secondary_loop_eval_bundle.secondary_loop_objective_profile | has("non_closure_objective_rate"))' >/dev/null 2>&1; then log_pass "closure bundle route packages replay consumer + continuity gate + objective-profile evidence"; else log_fail "closure bundle route missing expected fields: $CLOSURE_BUNDLE"; fi
CHECKPOINTS=$(http_json "${BASE_URL}/v1/work-loop/checkpoints")
if echo "$CHECKPOINTS" | jq -e 'has("resume_payload") and has("last_checkpoint_id") and has("last_safe_reentry_prompt_basis")' >/dev/null 2>&1; then log_pass "checkpoints route exposes resume/checkpoint surface"; else log_fail "checkpoints route missing expected fields: $CHECKPOINTS"; fi
CTX=$(http_json -X POST "${BASE_URL}/v1/work-loop/context" -H 'Content-Type: application/json' -d '{"current_ask":"triple verify spec79","ask_kind":"instruction","scope_kind":"mission_carryover","carryover_policy":"allow_if_relevant","excluded_context_reason":"none","excluded_context_labels":[],"source_turn_id":"spec79-triple-verify","operator_steering_detected":true}')
if echo "$CTX" | jq -e '.status == "accepted"' >/dev/null 2>&1; then log_pass "context route accepts continuation-input updates"; else log_fail "context route rejected update: $CTX"; fi
STATUS2=$(http_json "${BASE_URL}/v1/work-loop/status")
if echo "$STATUS2" | jq -e '.decision_context.current_ask == "triple verify spec79" and .last_continue_reason == "operator steering detected"' >/dev/null 2>&1; then log_pass "continuation-input update is reflected canonically"; else log_fail "continuation-input update not reflected canonically: $STATUS2"; fi
echo "=== SPEC79 LIVE HTTP SMOKE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi

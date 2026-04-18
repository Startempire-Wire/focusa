#!/bin/bash
# Doc78 production-runtime smoke:
# - targets an already-running daemon (non-isolated)
# - validates governance continuation-boundary pressure in live runtime
# - captures replay/closure/objective evidence routes used for closure signoff
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:18799}"
WRITER_ID="${FOCUSA_DOC78_PROD_WRITER:-doc78-production-runtime-smoke}"
PARENT_WORK_ITEM_ID="${FOCUSA_DOC78_PROD_PARENT_WORK_ITEM_ID:-focusa-o8vn}"
ARTIFACT_DIR="${FOCUSA_DOC78_PROD_ARTIFACT_DIR:-}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info(){ echo -e "${YELLOW}INFO${NC}: $1"; }

http_json(){ curl -sS "$@"; }

resolve_writer_id(){
  local active_writer
  active_writer=$(http_json "${BASE_URL}/v1/work-loop/status" | jq -r '.active_writer // empty')
  if [ -n "$active_writer" ]; then
    echo "$active_writer"
  else
    echo "$WRITER_ID"
  fi
}

write_json(){
  local url="$1"
  shift
  local writer_id
  writer_id=$(resolve_writer_id)
  http_json -X POST "$url" -H 'Content-Type: application/json' -H "x-focusa-writer-id: ${writer_id}" "$@"
}

ORIG_DESTRUCTIVE="false"
ORIG_GOVERNANCE="false"
ORIG_OVERRIDE="false"
RESTORE_PAUSE_FLAGS=0

restore_pause_flags(){
  write_json "${BASE_URL}/v1/work-loop/pause-flags" \
    -d "{\"destructive_confirmation_required\":${ORIG_DESTRUCTIVE},\"governance_decision_pending\":${ORIG_GOVERNANCE},\"operator_override_active\":${ORIG_OVERRIDE},\"reason\":\"doc78 production-runtime restore\"}"
}

cleanup(){
  set +e
  if [ "${RESTORE_PAUSE_FLAGS}" -eq 1 ]; then
    restore_pause_flags >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

write_artifact(){
  local name="$1"
  local payload="$2"
  if [ -n "$ARTIFACT_DIR" ]; then
    printf '%s\n' "$payload" > "${ARTIFACT_DIR}/${name}.json"
  fi
}

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  {
    echo "base_url=${BASE_URL}"
    echo "writer_id=${WRITER_ID}"
    echo "parent_work_item_id=${PARENT_WORK_ITEM_ID}"
    echo "started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } > "${ARTIFACT_DIR}/run.meta"
  log_info "artifact capture enabled: ${ARTIFACT_DIR}"
fi

echo "=== DOC78 PRODUCTION RUNTIME GOVERNANCE + REPLAY SMOKE ==="
echo "Base URL: ${BASE_URL}"
echo ""

HEALTH=$(http_json "${BASE_URL}/v1/health")
write_artifact "health" "$HEALTH"
if echo "$HEALTH" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "daemon health route is ready"
else
  log_fail "daemon health route not ready: $HEALTH"
  echo ""
  echo "=== DOC78 PRODUCTION RUNTIME RESULTS ==="
  echo "Tests passed: $PASSED"
  echo "Tests failed: $FAILED"
  exit 1
fi

STATUS0=$(http_json "${BASE_URL}/v1/work-loop/status")
write_artifact "status_before" "$STATUS0"
if echo "$STATUS0" | jq -e '.governance.policy_owner != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events != null' >/dev/null 2>&1; then
  log_pass "status exposes governance owner + continuity gate + objective profile"
else
  log_fail "status missing expected production-runtime fields: $STATUS0"
fi
ORIG_DESTRUCTIVE=$(echo "$STATUS0" | jq -r '.pause_flags.destructive_confirmation_required // false')
ORIG_GOVERNANCE=$(echo "$STATUS0" | jq -r '.pause_flags.governance_decision_pending // false')
ORIG_OVERRIDE=$(echo "$STATUS0" | jq -r '.pause_flags.operator_override_active // false')

SESSION=$(http_json -X POST "${BASE_URL}/v1/session/start" -H 'Content-Type: application/json' -d '{"workspace_id":"doc78-production-runtime-smoke"}')
write_artifact "session_start" "$SESSION"
if echo "$SESSION" | jq -e '.ok == true or .status == "ok" or .status == "started" or .status == "accepted"' >/dev/null 2>&1; then
  log_pass "session start accepted"
else
  log_fail "session start rejected: $SESSION"
fi

TRACE_BEFORE=$(http_json "${BASE_URL}/v1/telemetry/trace?event_type=scope_failure_recorded&limit=400")
write_artifact "scope_failure_trace_before" "$TRACE_BEFORE"
TRACE_MARKER_IDS_BEFORE=$(echo "$TRACE_BEFORE" | jq -c '[.events[] | select(.payload.reason == "governance decision pending" and .payload.path == "select_next_continuous_subtask") | .event_id]')
TRACE_COUNT_BEFORE=$(echo "$TRACE_BEFORE" | jq -r '(.count // 0) | tonumber')

PAUSE_GOV=$(write_json "${BASE_URL}/v1/work-loop/pause-flags" -d "{\"destructive_confirmation_required\":${ORIG_DESTRUCTIVE},\"governance_decision_pending\":true,\"operator_override_active\":${ORIG_OVERRIDE},\"reason\":\"doc78 production governance pressure smoke\"}")
write_artifact "pause_flags_set_governance" "$PAUSE_GOV"
if echo "$PAUSE_GOV" | jq -e '.ok == true' >/dev/null 2>&1; then
  RESTORE_PAUSE_FLAGS=1
  log_pass "governance pause set for production pressure round"
else
  log_fail "governance pause set rejected: $PAUSE_GOV"
fi

SEL1=$(write_json "${BASE_URL}/v1/work-loop/select-next" -d "{\"parent_work_item_id\":\"${PARENT_WORK_ITEM_ID}\"}")
write_artifact "select_next_round1" "$SEL1"
if echo "$SEL1" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "select-next accepted under governance boundary"
else
  log_fail "select-next rejected under governance boundary: $SEL1"
fi

HEART1=$(write_json "${BASE_URL}/v1/work-loop/heartbeat" -d '{}')
write_artifact "heartbeat_round1" "$HEART1"
if echo "$HEART1" | jq -e '.ok == true and .dispatched == false' >/dev/null 2>&1; then
  log_pass "heartbeat dispatch suppressed while governance boundary active"
else
  log_fail "heartbeat unexpectedly dispatched under governance boundary: $HEART1"
fi

SEL2=$(write_json "${BASE_URL}/v1/work-loop/select-next" -d "{\"parent_work_item_id\":\"${PARENT_WORK_ITEM_ID}\"}")
write_artifact "select_next_round2" "$SEL2"
if echo "$SEL2" | jq -e '.ok == true' >/dev/null 2>&1; then
  log_pass "second select-next accepted for sustained governance pressure"
else
  log_fail "second select-next rejected during sustained governance pressure: $SEL2"
fi

HEART2=$(write_json "${BASE_URL}/v1/work-loop/heartbeat" -d '{}')
write_artifact "heartbeat_round2" "$HEART2"
if echo "$HEART2" | jq -e '.ok == true and .dispatched == false' >/dev/null 2>&1; then
  log_pass "second heartbeat remains suppressed during sustained governance pressure"
else
  log_fail "second heartbeat unexpectedly dispatched: $HEART2"
fi

STATUS_GOV=""
gov_ok=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  STATUS_GOV=$(http_json "${BASE_URL}/v1/work-loop/status")
  if echo "$STATUS_GOV" | jq -e '
    .pause_flags.governance_decision_pending == true
    and .last_blocker_class == "governance"
    and (.status == "blocked" or .status == "paused")
    and ((.last_continue_reason // "") | test("continuation boundary"))
    and ((.last_blocker_reason // "") | test("governance decision pending|governance pressure smoke"))
  ' >/dev/null 2>&1; then
    gov_ok=1
    break
  fi
  sleep 0.2
done
write_artifact "status_governance_blocked" "$STATUS_GOV"
if [ "$gov_ok" -eq 1 ]; then
  log_pass "status reflects governance continuation-boundary semantics during production pressure"
else
  log_fail "status missing governance continuation-boundary semantics: $STATUS_GOV"
fi

TRACE_AFTER=$(http_json "${BASE_URL}/v1/telemetry/trace?event_type=scope_failure_recorded&limit=400")
write_artifact "scope_failure_trace_after" "$TRACE_AFTER"
TRACE_COUNT_AFTER=$(echo "$TRACE_AFTER" | jq -r '(.count // 0) | tonumber')
if echo "$TRACE_AFTER" | jq -e --argjson before_ids "$TRACE_MARKER_IDS_BEFORE" '
  ([.events[] | select(.payload.reason == "governance decision pending" and .payload.path == "select_next_continuous_subtask") | .event_id]) as $after_ids
  | ($after_ids | length) > 0
  and ($after_ids | any(. as $id | ($before_ids | index($id) | not)))
' >/dev/null 2>&1; then
  log_pass "scope_failure_recorded trace stream carries new governance continuation-boundary evidence"
elif [ "$gov_ok" -eq 1 ] && echo "$TRACE_AFTER" | jq -e '([.events[] | .payload.reason // empty] | any(. == "governance decision pending")) and ([.events[] | .payload.path // empty] | any(. == "select_next_continuous_subtask"))' >/dev/null 2>&1; then
  log_pass "scope_failure_recorded trace stream carries governance continuation-boundary markers (retained/deduplicated history)"
else
  log_fail "scope_failure_recorded trace evidence missing governance markers (before_count=${TRACE_COUNT_BEFORE}, after_count=${TRACE_COUNT_AFTER}, before_marker_ids=${TRACE_MARKER_IDS_BEFORE}): $TRACE_AFTER"
fi

NON_CLOSURE_TRACES=$(http_json "${BASE_URL}/v1/telemetry/trace?event_type=verification_result&limit=400")
write_artifact "verification_result_trace_snapshot" "$NON_CLOSURE_TRACES"
if echo "$NON_CLOSURE_TRACES" | jq -e '([.events[].payload.loop_objective // empty] | any(. != "" and . != "continuous_turn_outcome_quality"))' >/dev/null 2>&1; then
  log_pass "verification traces include non-closure loop objectives in production runtime"
else
  log_fail "verification traces missing non-closure loop objectives: $NON_CLOSURE_TRACES"
fi

STATUS_AFTER=$(http_json "${BASE_URL}/v1/work-loop/status")
write_artifact "status_after" "$STATUS_AFTER"
if echo "$STATUS_AFTER" | jq -e '.secondary_loop_eval_bundle.secondary_loop_objective_profile.quality_trace_events > 0 and .secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events > 0 and (.secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_rate // 0) > 0' >/dev/null 2>&1; then
  log_pass "status objective profile reports live non-closure evidence"
else
  log_fail "status objective profile lacks live non-closure evidence: $STATUS_AFTER"
fi

BUNDLE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-bundle")
write_artifact "closure_bundle" "$BUNDLE"
if echo "$BUNDLE" | jq -e '.status == "ok" and .doc == "78" and .secondary_loop_continuity_gate.state != null and .secondary_loop_replay_consumer.status != null and .secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events > 0' >/dev/null 2>&1; then
  log_pass "closure-bundle carries production replay gate + non-closure objective evidence"
else
  log_fail "closure-bundle missing production replay/non-closure evidence: $BUNDLE"
fi

EVIDENCE=$(http_json "${BASE_URL}/v1/work-loop/replay/closure-evidence")
write_artifact "closure_evidence" "$EVIDENCE"
if echo "$EVIDENCE" | jq -e '.status != null and .secondary_loop_continuity_gate.state != null and .secondary_loop_replay_comparative.status != null and .secondary_loop_closure_replay_evidence.status != null' >/dev/null 2>&1; then
  log_pass "closure-evidence fallback route remains available during production pressure"
else
  log_fail "closure-evidence route missing expected payload: $EVIDENCE"
fi

CHECKPOINTS=$(http_json "${BASE_URL}/v1/work-loop/checkpoints")
write_artifact "checkpoints" "$CHECKPOINTS"
if echo "$CHECKPOINTS" | jq -e 'has("resume_payload") and has("last_checkpoint_id") and has("last_safe_reentry_prompt_basis")' >/dev/null 2>&1; then
  log_pass "checkpoints route exposes escalation/reentry artifact fields"
else
  log_fail "checkpoints route missing escalation/reentry fields: $CHECKPOINTS"
fi

RESTORE=$(restore_pause_flags)
write_artifact "pause_flags_restore" "$RESTORE"
if echo "$RESTORE" | jq -e '.ok == true' >/dev/null 2>&1; then
  RESTORE_PAUSE_FLAGS=0
  log_pass "pause flags restored to pre-smoke state"
else
  log_fail "pause flag restore rejected: $RESTORE"
fi

if [ -n "$ARTIFACT_DIR" ]; then
  {
    echo "tests_passed=${PASSED}"
    echo "tests_failed=${FAILED}"
    echo "finished_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } > "${ARTIFACT_DIR}/result.meta"
fi

echo ""
echo "=== DOC78 PRODUCTION RUNTIME RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

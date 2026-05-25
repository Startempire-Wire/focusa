#!/usr/bin/env bash
# Focusa-native dogfood validation.
# Validates Focusa as an agent cognition/continuity system, not WPUIAI.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd -P)"
BASE_URL="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
V1="${BASE_URL%/}/v1"
PROJECT_ROOT="${FOCUSA_DOGFOOD_PROJECT_ROOT:-$ROOT_DIR}"
CONTINUITY_ID="${FOCUSA_DOGFOOD_CONTINUITY_ID:-focusa-dogfood-$(date +%s)-$$}"
WRITER_ID="${FOCUSA_DOGFOOD_WRITER_ID:-focusa-dogfood-$$}"
RUN_SLOW="${FOCUSA_DOGFOOD_SLOW:-0}"
RUN_MUTATING_LOOP="${FOCUSA_DOGFOOD_MUTATING_LOOP:-0}"
TMP_DIR="$(mktemp -d /tmp/focusa-dogfood.XXXXXX)"
PASSED=0
FAILED=0

cleanup() {
  if [[ "${FOCUSA_DOGFOOD_KEEP_ARTIFACTS:-0}" != "1" ]]; then
    rm -rf "$TMP_DIR"
  else
    echo "artifacts kept: $TMP_DIR"
  fi
}
trap cleanup EXIT

cd "$ROOT_DIR"

pass() { echo "✓ PASS: $1"; PASSED=$((PASSED+1)); }
fail() { echo "✗ FAIL: $1${2:+ :: $2}"; FAILED=$((FAILED+1)); }

need_cmd() {
  if command -v "$1" >/dev/null 2>&1; then pass "cmd:$1"; else fail "cmd:$1" "missing"; fi
}
optional_cmd() {
  if command -v "$1" >/dev/null 2>&1; then pass "cmd:$1"; else echo "↷ SKIP: cmd:$1 (${2:-optional})"; fi
}

request() {
  local method="$1" path="$2" body="${3:-}" out="$4" timeout="${5:-15}"
  local code
  if [[ -n "$body" ]]; then
    code="$(curl -sS --max-time "$timeout" -o "$out" -w '%{http_code}' -X "$method" "$V1$path" \
      -H 'content-type: application/json' \
      -H "x-focusa-writer-id: $WRITER_ID" \
      -H 'x-focusa-permissions: admin:*' \
      --data "$body" || true)"
  else
    code="$(curl -sS --max-time "$timeout" -o "$out" -w '%{http_code}' -X "$method" "$V1$path" \
      -H "x-focusa-writer-id: $WRITER_ID" \
      -H 'x-focusa-permissions: admin:*' || true)"
  fi
  [[ "$code" =~ ^2 ]] && jq empty "$out" >/dev/null 2>&1
}

assert_req() {
  local name="$1" method="$2" path="$3" body="${4:-}" jqexpr="${5:-.}" timeout="${6:-15}"
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.json"
  if request "$method" "$path" "$body" "$out" "$timeout" && jq -e "$jqexpr" "$out" >/dev/null 2>&1; then
    pass "$name"
  else
    fail "$name" "$(tail -c 700 "$out" 2>/dev/null || true)"
  fi
}

assert_cmd() {
  local name="$1"; shift
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.log"
  if "$@" >"$out" 2>&1; then pass "$name"; else fail "$name" "$(tail -c 1000 "$out" 2>/dev/null || true)"; fi
}

assert_cmd_or_bounded_degraded() {
  local name="$1"; shift
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.log"
  if "$@" >"$out" 2>&1; then
    pass "$name"
  elif grep -q '"status":"pending"' "$out" && grep -q '"failure_class":"resource_exhausted"' "$out" && grep -q '"retry_posture":"safe_retry"' "$out"; then
    pass "$name bounded_degraded_resource_exhausted"
  else
    fail "$name" "$(tail -c 1000 "$out" 2>/dev/null || true)"
  fi
}

echo "=== FOCUSA DOGFOOD ==="
echo "root=$ROOT_DIR"
echo "project_root=$PROJECT_ROOT"
echo "continuity_id=$CONTINUITY_ID"
echo "base=$V1"
echo "artifacts=$TMP_DIR"
echo

need_cmd curl
need_cmd jq
need_cmd node
optional_cmd cargo "only required with FOCUSA_DOGFOOD_SLOW=1"

# 1. Static/tooling contract: agent-visible tools should stay documented and typed.
assert_cmd "pi_extension_typecheck" bash -lc 'cd apps/pi-extension && npx tsc --noEmit'
assert_cmd "tool_contract_static_registry" node scripts/validate-focusa-tool-contracts.mjs --json

# 2. Daemon health and inspectability.
for _ in $(seq 1 20); do
  if curl -fsS --max-time 3 "$V1/health" >/dev/null 2>&1; then break; fi
  sleep 0.5
done
assert_req "daemon_health" GET /health '' '.ok == true' 5
assert_req "daemon_status_summary" GET '/status?summary_only=true' '' '. != null' 5
assert_req "resource_mode_read" GET /resource/mode '' '.resource_mode != null or .mode != null' 5
assert_req "tool_contracts_live" GET /ontology/tool-contracts '' '. != null' 10

# 3. Trajectory: operator mission should be clear and project-bound.
assert_req "trajectory_view_summary" GET "/trajectory/view?project_root=$PROJECT_ROOT&mode=summary" '' '. != null' 10
ASSESS_BODY="$(jq -nc --arg root "$PROJECT_ROOT" '{project_root:$root, observed_state:"Focusa dogfood run exercising agent UX gates", evidence_refs:["tests/focusa_dogfood_test.sh"]}')"
assert_req "trajectory_assess" POST /trajectory/assess "$ASSESS_BODY" '. != null' 10

# 4. Workpoint continuity: checkpoint -> visible/current -> evidence -> resume.
KEY="focusa-dogfood-$CONTINUITY_ID"
CHECKPOINT_BODY="$(jq -nc \
  --arg root "$PROJECT_ROOT" \
  --arg cont "$CONTINUITY_ID" \
  --arg key "$KEY" \
  '{project_root:$root, continuity_id:$cont, session_id:$cont, checkpoint_reason:"manual", canonical:true, promote:true, idempotency_key:$key, mission:"Focusa dogfood validation", next_slice:"Prove Focusa-native agent UX loops", active_object_refs:["tests/focusa_dogfood_test.sh","docs/current/FOCUSA_DOGFOOD.md"], action_intent:{action_type:"dogfood_validate", target_ref:"FocusaToolSuite", verification_hooks:["health","trajectory","workpoint","evidence","metacog","resource"], status:"ready"}}')"
WP_OUT="$TMP_DIR/workpoint_checkpoint.json"
if request POST /workpoint/checkpoint "$CHECKPOINT_BODY" "$WP_OUT" 15 && WID="$(jq -r '.workpoint_id // empty' "$WP_OUT")" && [[ -n "$WID" ]]; then
  pass "workpoint_checkpoint"
else
  WID=""
  fail "workpoint_checkpoint" "$(cat "$WP_OUT" 2>/dev/null || true)"
fi

if [[ -n "$WID" ]]; then
  CURRENT_OUT="$TMP_DIR/workpoint_current.json"
  visible=0
  for _ in $(seq 1 30); do
    if request GET "/workpoint/current?project_root=$PROJECT_ROOT&continuity_id=$CONTINUITY_ID" '' "$CURRENT_OUT" 5 && jq -e --arg wid "$WID" '.workpoint_id == $wid or .active_workpoint_id == $wid' "$CURRENT_OUT" >/dev/null 2>&1; then
      visible=1; break
    fi
    sleep 0.25
  done
  [[ "$visible" == "1" ]] && pass "workpoint_current_visibility" || fail "workpoint_current_visibility" "$(cat "$CURRENT_OUT" 2>/dev/null || true)"

  EVIDENCE_BODY="$(jq -nc --arg wid "$WID" '{workpoint_id:$wid,target_ref:"tests/focusa_dogfood_test.sh",result:"Focusa dogfood evidence link exercised",evidence_ref:"tests/focusa_dogfood_test.sh:dogfood"}')"
  assert_req "workpoint_evidence_link" POST /workpoint/evidence/link "$EVIDENCE_BODY" '(.status == "accepted") or (.status == "pending")' 10
fi

RESUME_BODY="$(jq -nc --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" '{mode:"operator_summary", project_root:$root, continuity_id:$cont}')"
assert_req "workpoint_resume" POST /workpoint/resume "$RESUME_BODY" '.canonical == true or .resume_packet != null or .resume_packet_v2 != null or .status == "completed" or (.status == "pending" and .failure_class == "resource_exhausted" and .retry_posture == "safe_retry")' 15

# 5. Evidence/traverse UX: proof should be recoverable without transcript memory.
assert_req "traverse_recent_evidence" POST /traverse '{"surface":"evidence","selector":"recent","limit":5,"fields":["id","label","summary"]}' '.status == "completed" or .items != null or .results != null' 10
assert_req "active_object_context" POST /ontology/context '{"current_ask":"Focusa dogfood: identify active object and next proof","budget_tokens":320,"view_profile":"pi_operator_view","slice_type":"active_mission"}' '. != null' 10

# 6. Metacognition + prediction loop: learning surfaces should accept bounded signals.
PRED_BODY="$(jq -nc '{prediction_type:"dogfood_gate_success",predicted_outcome:"Focusa dogfood gates identify actionable subsystem health",confidence:0.74,recommended_action:"Run focusa_dogfood_test before release claims",why:"Dogfood composes health, trajectory, workpoint, evidence, metacog, and resource gates"}')"
PRED_OUT="$TMP_DIR/predict_record.json"
if request POST /predictions "$PRED_BODY" "$PRED_OUT" 10; then
  PID="$(jq -r '.prediction.prediction_id // .prediction_id // .id // empty' "$PRED_OUT")"
  pass "predict_record"
else
  PID=""
  fail "predict_record" "$(cat "$PRED_OUT" 2>/dev/null || true)"
fi
CAPTURE_BODY="$(jq -nc '{kind:"dogfood_signal",content:"Focusa dogfood composes agent UX gates across health, trajectory, Workpoint, evidence, metacog, and resource modes.",rationale:"Reusable release/stress validation should cover agent-facing recovery, not just isolated unit checks.",confidence:0.82,strategy_class:"focusa_dogfood",evidence_refs:["tests/focusa_dogfood_test.sh"]}')"
assert_req "metacog_capture" POST /metacognition/capture "$CAPTURE_BODY" '.capture_id != null or .id != null or .status == "accepted"' 10
assert_req "metacog_retrieve" POST /metacognition/retrieve '{"current_ask":"Focusa dogfood agent UX validation","scope_tags":["focusa_dogfood"],"k":5}' '.candidates != null or .results != null' 10
if [[ -n "$PID" ]]; then
  EVAL_BODY="$(jq -nc --arg pid "$PID" '{prediction_id:$pid, actual_outcome:"Dogfood script executed prediction record path", score:0.8}')"
  assert_req "predict_evaluate" POST "/predictions/$PID/evaluate" "$EVAL_BODY" '. != null' 10
fi

# 7. Resource pressure: all official tools remain callable; hot routes recover after cold pressure.
assert_cmd_or_bounded_degraded "lowmem_surgical_agent_stress" bash tests/spec96_lowmem_surgical_agent_stress_test.sh

# 8. Existing wider live stress is optional because it writes many bounded records.
if [[ "$RUN_MUTATING_LOOP" == "1" ]]; then
  assert_cmd "focusa_tool_stress_existing" bash tests/focusa_tool_stress_test.sh
else
  echo "↷ SKIP: focusa_tool_stress_existing (set FOCUSA_DOGFOOD_MUTATING_LOOP=1)"
fi

# 9. Cargo gate can be slow; opt in for local release proof.
if [[ "$RUN_SLOW" == "1" ]]; then
  if command -v cargo >/dev/null 2>&1; then
    assert_cmd "cargo_test_workspace" cargo test --workspace
  else
    fail "cargo_test_workspace" "cargo missing but FOCUSA_DOGFOOD_SLOW=1"
  fi
else
  echo "↷ SKIP: cargo_test_workspace (set FOCUSA_DOGFOOD_SLOW=1)"
fi

echo
echo "=== FOCUSA DOGFOOD RESULTS ==="
echo "passed=$PASSED failed=$FAILED artifacts=$TMP_DIR"
if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "FOCUSA_DOGFOOD=PASS"

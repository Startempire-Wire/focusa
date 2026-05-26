#!/usr/bin/env bash
# Bounded live CLI smoke for model-facing Focusa tool parity paths.
set -euo pipefail
CLI="${FOCUSA_CLI:-target/release/focusa}"
PROJECT_ROOT="${FOCUSA_SMOKE_PROJECT_ROOT:-$(pwd -P)}"
CONTINUITY_ID="${FOCUSA_SMOKE_CONTINUITY_ID:-cli-smoke-$(date +%s)-$$}"
KEY="cli-smoke-$(date +%s)-$$"
TMP_DIR="${TMPDIR:-/tmp}/focusa-cli-smoke-$$"
mkdir -p "$TMP_DIR"
PASSED=0; FAILED=0
pass(){ echo "✓ PASS: $1"; PASSED=$((PASSED+1)); }
fail(){ echo "✗ FAIL: $1 :: ${2:-}"; FAILED=$((FAILED+1)); }
run_json(){
  local name="$1" jqexpr="$2"; shift 2
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.json"
  local err="$out.err"
  if "$@" >"$out" 2>"$err" && jq -e "$jqexpr" "$out" >/dev/null 2>&1; then
    pass "$name"
  else
    fail "$name" "$(tail -c 400 "$err" 2>/dev/null) $(tail -c 600 "$out" 2>/dev/null)"
  fi
}
run_json_retry(){
  local name="$1" jqexpr="$2"; shift 2
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.json"
  local err="$out.err"
  for _ in $(seq 1 3); do
    if "$@" >"$out" 2>"$err" && jq -e "$jqexpr" "$out" >/dev/null 2>&1; then
      pass "$name"
      return 0
    fi
    sleep 1
  done
  fail "$name" "$(tail -c 400 "$err" 2>/dev/null) $(tail -c 600 "$out" 2>/dev/null)"
}
wait_current(){
  local out="$TMP_DIR/workpoint_current_wait.json"
  for _ in $(seq 1 30); do
    if "$CLI" workpoint current --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --json >"$out" 2>/dev/null \
      && jq -e '(.status == "active" or .status == "completed") and .canonical == true' "$out" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  return 1
}
wait_health(){
  local out="$TMP_DIR/health_wait.json"
  local consecutive=0
  for _ in $(seq 1 40); do
    if curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health >"$out" 2>"$out.err" \
      && jq -e '.ok == true or .status == "ok"' "$out" >/dev/null 2>&1; then
      consecutive=$((consecutive+1))
      [[ "$consecutive" -ge 2 ]] && return 0
    else
      consecutive=0
    fi
    sleep 1
  done
  echo "health readiness failed: $(tail -c 240 "$out.err" 2>/dev/null) $(tail -c 240 "$out" 2>/dev/null)" >&2
  return 1
}

wait_health || { echo "=== FOCUSA CLI PARITY SMOKE RESULTS ==="; echo "passed=$PASSED failed=1 artifacts=$TMP_DIR"; exit 1; }
run_json health '.status == "ok" or .ok == true' curl -fsS --max-time 8 http://127.0.0.1:8787/v1/health
run_json tool_contracts '.contracts | length == 59' curl -fsS --max-time 8 http://127.0.0.1:8787/v1/ontology/tool-contracts
run_json tool_choreography '.schema == "focusa.tool_choreography.v1" and .tool_count == 59 and .edge_count >= 59 and (.per_tool_next_tools.focusa_project_identity | length > 0)' curl -fsS --max-time 8 http://127.0.0.1:8787/v1/ontology/tool-choreography
run_json project_identity '.status == "completed" and .project_identity.status == "verified"' "$CLI" project identity --project-root "$PROJECT_ROOT" --json
run_json trajectory_view '.project_identity != null and (.trajectory != null or .status != null)' "$CLI" trajectory view --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --mode summary --json
run_json resource_status '.mode != null or .resource_mode != null' "$CLI" resource status --json
run_json_retry focus_update '.status == "accepted"' "$CLI" focus update --turn-id "$KEY-focus" --note "CLI parity smoke note." --json
run_json workpoint_checkpoint '(.status == "accepted" or .status == "pending") and .canonical == true' "$CLI" workpoint checkpoint --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --mission "Focusa CLI smoke" --next-action "Complete CLI parity smoke" --action-type smoke_verify --target-ref FocusaCliSmoke --idempotency-key "$KEY-wp" --json
if wait_current; then pass workpoint_current_visible; else fail workpoint_current_visible "$(cat "$TMP_DIR/workpoint_current_wait.json" 2>/dev/null)"; fi
run_json workpoint_resume '.status == "completed" and .canonical == true' "$CLI" workpoint resume --project-root "$PROJECT_ROOT" --continuity-id "$CONTINUITY_ID" --json
run_json metacog_recent_reflections '.reflections != null' "$CLI" metacognition recent-reflections --limit 1 --json
run_json metacog_recent_adjustments '.adjustments != null' "$CLI" metacognition recent-adjustments --limit 1 --json
run_json lineage_extract '.signals != null and .next_tools != null' "$CLI" lineage extract --max-candidates 3 --json
run_json snapshot_recent '.snapshots != null' "$CLI" state snapshot recent --limit 1 --json
run_json snapshot_compare_latest '.status != null or .checksum_changed != null or .created != null' "$CLI" state snapshot compare-latest --snapshot-reason "cli-smoke" --json
PREDICT_RECORD_OUT="$TMP_DIR/predict_record.json"
PREDICT_RECORD_ERR="$PREDICT_RECORD_OUT.err"
if "$CLI" predict record --prediction-type smoke --predicted-outcome "safe smoke succeeds" --confidence 0.8 --recommended-action "continue smoke" --why "safe fixture prediction for CLI parity" --context-refs "$KEY,tool_edge:focusa_project_identity->focusa_trajectory_view" --json >"$PREDICT_RECORD_OUT" 2>"$PREDICT_RECORD_ERR" \
  && jq -e '.status == "recorded" and .prediction.prediction_id != null' "$PREDICT_RECORD_OUT" >/dev/null 2>&1; then
  pass predict_record
  PREDICTION_ID="$(jq -r '.prediction.prediction_id' "$PREDICT_RECORD_OUT")"
  run_json predict_evaluate '.status == "evaluated" and .prediction.score == 1' "$CLI" predict evaluate "$PREDICTION_ID" --actual-outcome "safe smoke succeeds" --score 1 --learning-signal-ref "$KEY" --json
else
  fail predict_record "$(tail -c 400 "$PREDICT_RECORD_ERR" 2>/dev/null) $(tail -c 600 "$PREDICT_RECORD_OUT" 2>/dev/null)"
fi
run_json tool_choreography_dynamic '.runtime_weight_adjustments | map(select(.edge == "focusa_project_identity->focusa_trajectory_view" and .samples >= 1)) | length >= 1' curl -fsS --max-time 8 http://127.0.0.1:8787/v1/ontology/tool-choreography
run_json predict_recent '.predictions != null or .items != null or .total != null' "$CLI" predict recent --limit 1 --json
run_json predict_stats '.status != null or .stats != null or .prediction_count != null or .total_predictions != null' "$CLI" predict stats --json

echo "=== FOCUSA CLI PARITY SMOKE RESULTS ==="
echo "passed=$PASSED failed=$FAILED artifacts=$TMP_DIR"
if [[ "$FAILED" -ne 0 ]]; then exit 1; fi

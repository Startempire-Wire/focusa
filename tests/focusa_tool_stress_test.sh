#!/usr/bin/env bash
# Live stress suite for Focusa tool surfaces. Non-destructive except bounded Focusa state/snapshot/metacog test records.
set -euo pipefail
BASE="${FOCUSA_BASE:-http://127.0.0.1:8787/v1}"
WRITER_ID="${FOCUSA_STRESS_WRITER_ID:-focusa-tool-stress}"
FAILED=0; PASSED=0
TMP_DIR="${TMPDIR:-/tmp}/focusa-tool-stress-$$"
mkdir -p "$TMP_DIR"
ACTIVE_WRITER=$(curl -sS -m 3 "$BASE/work-loop/status" 2>/dev/null | jq -r '.active_writer // empty' 2>/dev/null || true)
if [[ -n "$ACTIVE_WRITER" ]]; then WRITER_ID="$ACTIVE_WRITER"; fi

pass(){ echo "✓ PASS: $1"; PASSED=$((PASSED+1)); }
fail(){ echo "✗ FAIL: $1 :: ${2:-}"; FAILED=$((FAILED+1)); }
request(){
  local method="$1" path="$2" body="${3:-}" out="$4"
  local code
  if [[ -n "$body" ]]; then
    code=$(curl -sS -m 8 -o "$out" -w '%{http_code}' -X "$method" "$BASE$path" -H 'content-type: application/json' -H "x-focusa-writer-id: $WRITER_ID" -d "$body" || true)
  else
    code=$(curl -sS -m 8 -o "$out" -w '%{http_code}' -X "$method" "$BASE$path" -H "x-focusa-writer-id: $WRITER_ID" || true)
  fi
  [[ "$code" =~ ^2 ]] || { echo "HTTP $code" >> "$out"; return 1; }
  jq empty "$out" >/dev/null 2>&1 || { echo "invalid json" >> "$out"; return 1; }
}
assert_req(){
  local name="$1" method="$2" path="$3" body="${4:-}" jqexpr="${5:-.}"
  local out="$TMP_DIR/${name//[^A-Za-z0-9_.-]/_}.json"
  if request "$method" "$path" "$body" "$out" && jq -e "$jqexpr" "$out" >/dev/null 2>&1; then pass "$name"; else fail "$name" "$(tail -c 500 "$out" 2>/dev/null)"; fi
}

KEY="stress-$(date +%s)-$$"

# Core health/session/focus state writes
assert_req health GET /health '' '.ok == true'
assert_req status GET /status '' '.session != null'
assert_req focus_stack GET /focus/stack '' '.stack.frames != null'
assert_req focus_update POST /focus/update "{\"turn_id\":\"stress-$KEY\",\"delta\":{\"recent_results\":[\"Stress focus write passed.\"],\"notes\":[\"Stress note.\"]}}" '.status == "accepted"'
assert_req focus_update_constraint POST /focus/update "{\"turn_id\":\"stress-constraint-$KEY\",\"delta\":{\"constraints\":[\"Operator directive must not demote existing Focusa tools.\"]}}" '.status == "accepted"'

# Workpoint API and idempotency
WP1="$TMP_DIR/workpoint1.json"; WP2="$TMP_DIR/workpoint2.json"
request POST /workpoint/checkpoint "{\"mission\":\"Focusa tool stress\",\"next_slice\":\"Complete stress suite\",\"checkpoint_reason\":\"manual\",\"confidence\":\"high\",\"canonical\":true,\"promote\":true,\"idempotency_key\":\"$KEY\",\"action_intent\":{\"action_type\":\"stress_verify\",\"target_ref\":\"FocusaToolSuite\",\"verification_hooks\":[\"api\",\"cli\",\"pi\"],\"status\":\"ready\"}}" "$WP1" && jq -e '.status == "accepted" and .canonical == true' "$WP1" >/dev/null && pass workpoint_checkpoint || fail workpoint_checkpoint "$(cat "$WP1" 2>/dev/null)"
sleep 1
request POST /workpoint/evidence/link "{\"target_ref\":\"tests/focusa_tool_stress_test.sh\",\"result\":\"stress evidence link passed\",\"evidence_ref\":\"tests/focusa_tool_stress_test.sh:1\"}" "$TMP_DIR/workpoint_link.json" && jq -e '.status == "accepted" and .canonical == true' "$TMP_DIR/workpoint_link.json" >/dev/null && pass workpoint_evidence_link || fail workpoint_evidence_link "$(cat "$TMP_DIR/workpoint_link.json" 2>/dev/null)"
request POST /workpoint/checkpoint "{\"mission\":\"Duplicate\",\"next_slice\":\"Duplicate\",\"idempotency_key\":\"$KEY\"}" "$WP2" && jq -e '.idempotent_replay == true' "$WP2" >/dev/null && pass workpoint_idempotency || fail workpoint_idempotency "$(cat "$WP2" 2>/dev/null)"
sleep 1
assert_req workpoint_current GET /workpoint/current '' '.status == "completed" and .canonical == true'
assert_req workpoint_resume POST /workpoint/resume '{"mode":"operator"}' '.status == "completed" and .resume_packet != null'
assert_req workpoint_drift_no POST /workpoint/drift-check '{"latest_action":"stress verify FocusaToolSuite api cli pi","expected_action_type":"stress_verify","emit":false}' '.status == "no_drift" and .drift_detected == false'
assert_req workpoint_drift_yes POST /workpoint/drift-check '{"latest_action":"Updated notes for unrelated object","expected_action_type":"patch_component_binding","active_object_refs":["Component:homepage.audio_widget"],"emit":false}' '.status == "drift_detected" and .drift_detected == true'

# Lineage/tree and snapshots
HEAD_JSON="$TMP_DIR/head.json"
if request GET /lineage/head '' "$HEAD_JSON" && HEAD_ID=$(jq -r 'if (.head|type)=="string" then .head else (.head.node_id // .head.id // .node_id // .id // empty) end' "$HEAD_JSON") && [[ -n "$HEAD_ID" ]]; then
  pass lineage_head
  assert_req lineage_tree GET '/lineage/tree?max_nodes=20' '' '.nodes != null or .tree != null or .status == "ok"'
  assert_req lineage_path GET "/lineage/path/$HEAD_ID" '' '.path != null or .nodes != null or .status == "ok"'
else fail lineage_head "$(cat "$HEAD_JSON" 2>/dev/null)"; HEAD_ID=""; fi
SNAP1="$TMP_DIR/snap1.json"; SNAP2="$TMP_DIR/snap2.json"
request POST /focus/snapshots '{"snapshot_reason":"stress-one"}' "$SNAP1" && S1=$(jq -r '.snapshot_id // empty' "$SNAP1") && [[ -n "$S1" ]] && pass snapshot_create_one || fail snapshot_create_one "$(cat "$SNAP1" 2>/dev/null)"
request POST /focus/snapshots '{"snapshot_reason":"stress-two"}' "$SNAP2" && S2=$(jq -r '.snapshot_id // empty' "$SNAP2") && [[ -n "$S2" ]] && pass snapshot_create_two || fail snapshot_create_two "$(cat "$SNAP2" 2>/dev/null)"
assert_req snapshots_recent GET '/focus/snapshots/recent?limit=5' '' '.snapshots != null'
if [[ -n "${S1:-}" && -n "${S2:-}" ]]; then assert_req snapshot_diff POST /focus/snapshots/diff "{\"from_snapshot_id\":\"$S1\",\"to_snapshot_id\":\"$S2\"}" '.checksum_changed != null or .status == "ok"'; fi

# Metacognition suite
CAP="$TMP_DIR/capture.json"; REF="$TMP_DIR/reflect.json"; ADJ="$TMP_DIR/adjust.json"
request POST /metacognition/capture "{\"kind\":\"stress_signal\",\"content\":\"Focusa stress signal $KEY\",\"rationale\":\"stress coverage\",\"confidence\":0.8,\"strategy_class\":\"tool_stress\"}" "$CAP" && CID=$(jq -r '.capture_id // empty' "$CAP") && [[ -n "$CID" ]] && pass metacog_capture || fail metacog_capture "$(cat "$CAP" 2>/dev/null)"
assert_req metacog_retrieve POST /metacognition/retrieve '{"current_ask":"Focusa stress signal","scope_tags":["tool_stress"],"k":5}' '.candidates != null'
request POST /metacognition/reflect '{"turn_range":"last 5","failure_classes":["tool_stress"]}' "$REF" && RID=$(jq -r '.reflection_id // empty' "$REF") && [[ -n "$RID" ]] && pass metacog_reflect || fail metacog_reflect "$(cat "$REF" 2>/dev/null)"
assert_req metacog_recent_reflections GET '/metacognition/reflections/recent?limit=5' '' '.reflections != null'
if [[ -n "${RID:-}" ]]; then
  request POST /metacognition/adjust "{\"reflection_id\":\"$RID\",\"selected_updates\":[\"prefer live stress probes\"]}" "$ADJ" && AID=$(jq -r '.adjustment_id // empty' "$ADJ") && [[ -n "$AID" ]] && pass metacog_adjust || fail metacog_adjust "$(cat "$ADJ" 2>/dev/null)"
  assert_req metacog_recent_adjustments GET '/metacognition/adjustments/recent?limit=5' '' '.adjustments != null'
  if [[ -n "${AID:-}" ]]; then assert_req metacog_evaluate POST /metacognition/evaluate "{\"adjustment_id\":\"$AID\",\"observed_metrics\":[\"stress_pass\"]}" '.result != null or .status == "ok"'; fi
fi

# Work-loop read/control-safe surfaces. Avoid enabling autonomous loop.
assert_req work_loop_status GET /work-loop/status '' '.status != null'
assert_req work_loop_context POST /work-loop/context '{"current_ask":"Focusa tool stress","ask_kind":"instruction","scope_kind":"mission_carryover","carryover_policy":"allow_if_relevant"}' '.status != null'
assert_req work_loop_checkpoint POST /work-loop/checkpoint '{"summary":"Focusa tool stress checkpoint"}' '.checkpoint_id != null or .status != null or .ok == true'
assert_req work_loop_pause POST /work-loop/pause '{"reason":"stress suite safe pause"}' '.status != null or .ok == true'
assert_req work_loop_resume POST /work-loop/resume '{}' '.status != null or .ok == true'
assert_req work_loop_stop POST /work-loop/stop '{"reason":"stress suite cleanup"}' '.status != null or .ok == true'

# Ontology/read surfaces
assert_req ontology_primitives GET /ontology/primitives '' '. != null'
assert_req ontology_world GET /ontology/world '' '. != null'
assert_req ontology_slices GET /ontology/slices '' '. != null'

# CLI smoke
if target/release/focusa workpoint current >/dev/null 2>&1; then pass cli_workpoint_current; else fail cli_workpoint_current; fi
if target/release/focusa workpoint resume >/dev/null 2>&1; then pass cli_workpoint_resume; else fail cli_workpoint_resume; fi
if target/release/focusa workpoint drift-check --latest-action 'stress verify FocusaToolSuite' --expected-action-type stress_verify >/dev/null 2>&1; then pass cli_workpoint_drift_check; else fail cli_workpoint_drift_check; fi

echo "=== FOCUSA TOOL STRESS RESULTS ==="
echo "passed=$PASSED failed=$FAILED artifacts=$TMP_DIR"
if [[ "$FAILED" -ne 0 ]]; then exit 1; fi

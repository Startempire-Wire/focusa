#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787/v1}"
AUTH_TOKEN="${FOCUSA_AUTH_TOKEN:-}"

hdrs=(-H "content-type: application/json" -H "x-focusa-permissions: admin:*")
if [[ -n "$AUTH_TOKEN" ]]; then
  hdrs+=(-H "authorization: Bearer $AUTH_TOKEN")
fi

post_json() {
  local path="$1"
  local body="$2"
  local raw
  raw=$(curl -sS "${hdrs[@]}" -X POST "$BASE_URL$path" --data "$body" -w $'\n%{http_code}')
  local status body_json
  status=$(echo "$raw" | tail -n1)
  body_json=$(echo "$raw" | sed '$d')
  printf '%s\n%s\n' "$status" "$body_json"
}

assert_jq() {
  local json="$1"
  local expr="$2"
  local msg="$3"
  if echo "$json" | jq -e "$expr" >/dev/null 2>&1; then
    echo "✓ PASS: $msg"
  else
    echo "✗ FAIL: $msg"
    echo "  expr: $expr"
    echo "  json: $json"
    exit 1
  fi
}

# health gate
health=$(curl -sS "$BASE_URL/health")
assert_jq "$health" '.ok == true' "daemon health is ok"

# snapshot create A
mapfile -t out < <(post_json "/focus/snapshots" '{"snapshot_reason":"live-flow-a"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot create A http ${out[0]}"; exit 1; }
snap_a_json="${out[1]}"
assert_jq "$snap_a_json" '.status == "ok" and (.snapshot_id|type=="string") and (.checksum|type=="string")' "snapshot create A envelope"
snap_a=$(echo "$snap_a_json" | jq -r '.snapshot_id')

# snapshot create B
mapfile -t out < <(post_json "/focus/snapshots" '{"snapshot_reason":"live-flow-b"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot create B http ${out[0]}"; exit 1; }
snap_b_json="${out[1]}"
assert_jq "$snap_b_json" '.status == "ok" and (.snapshot_id|type=="string")' "snapshot create B envelope"
snap_b=$(echo "$snap_b_json" | jq -r '.snapshot_id')

# snapshot diff A->B
mapfile -t out < <(post_json "/focus/snapshots/diff" "{\"from_snapshot_id\":\"$snap_a\",\"to_snapshot_id\":\"$snap_b\"}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot diff http ${out[0]}"; exit 1; }
diff_json="${out[1]}"
assert_jq "$diff_json" '.status == "ok" and (.checksum_changed|type=="boolean") and (.version_delta|type=="number")' "snapshot diff envelope"

# snapshot restore B merge
mapfile -t out < <(post_json "/focus/snapshots/restore" "{\"snapshot_id\":\"$snap_b\",\"restore_mode\":\"merge\"}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot restore http ${out[0]}"; exit 1; }
restore_json="${out[1]}"
assert_jq "$restore_json" '.status == "ok" and .restored == true and (.conflicts|type=="array")' "snapshot restore envelope"

# invalid restore mode
mapfile -t out < <(post_json "/focus/snapshots/restore" "{\"snapshot_id\":\"$snap_b\",\"restore_mode\":\"bad\"}")
[[ "${out[0]}" == "400" ]] || { echo "✗ FAIL: invalid restore mode expected 400 got ${out[0]}"; exit 1; }
assert_jq "${out[1]}" '.code == "DIFF_INPUT_INVALID"' "invalid restore mode returns typed code"

# metacog capture
mapfile -t out < <(post_json "/metacognition/capture" '{"kind":"lesson","content":"verify before close","confidence":0.8}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: metacog capture http ${out[0]}"; exit 1; }
capture_json="${out[1]}"
assert_jq "$capture_json" '.stored == true and (.capture_id|type=="string")' "metacog capture envelope"

# metacog retrieve
mapfile -t out < <(post_json "/metacognition/retrieve" '{"current_ask":"verify","scope_tags":["close"],"k":3}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: metacog retrieve http ${out[0]}"; exit 1; }
retrieve_json="${out[1]}"
assert_jq "$retrieve_json" '(.candidates|type=="array") and (.ranked_by|type=="string")' "metacog retrieve envelope"

# metacog reflect
mapfile -t out < <(post_json "/metacognition/reflect" '{"turn_range":"1..5","failure_classes":["scope_contamination"]}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: metacog reflect http ${out[0]}"; exit 1; }
reflect_json="${out[1]}"
assert_jq "$reflect_json" '(.reflection_id|type=="string") and (.strategy_updates|type=="array")' "metacog reflect envelope"
refl_id=$(echo "$reflect_json" | jq -r '.reflection_id')

# metacog adjust
mapfile -t out < <(post_json "/metacognition/adjust" "{\"reflection_id\":\"$refl_id\",\"selected_updates\":[\"mitigate scope_contamination\"]}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: metacog adjust http ${out[0]}"; exit 1; }
adjust_json="${out[1]}"
assert_jq "$adjust_json" '(.adjustment_id|type=="string") and (.expected_deltas|type=="object")' "metacog adjust envelope"
adj_id=$(echo "$adjust_json" | jq -r '.adjustment_id')

# metacog evaluate
mapfile -t out < <(post_json "/metacognition/evaluate" "{\"adjustment_id\":\"$adj_id\",\"observed_metrics\":[\"failed_turn_ratio\"]}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: metacog evaluate http ${out[0]}"; exit 1; }
eval_json="${out[1]}"
assert_jq "$eval_json" '(.evaluation_id|type=="string") and (.promote_learning==true)' "metacog evaluate envelope"

# negative: adjust unknown reflection
mapfile -t out < <(post_json "/metacognition/adjust" '{"reflection_id":"refl-missing","selected_updates":[]}')
[[ "${out[0]}" == "404" ]] || { echo "✗ FAIL: missing reflection expected 404 got ${out[0]}"; exit 1; }
assert_jq "${out[1]}" '.code == "REFLECTION_NOT_FOUND"' "metacog adjust missing reflection typed code"

echo "SPEC80 live flow runtime: PASS"

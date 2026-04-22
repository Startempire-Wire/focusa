#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/focusa-daemon"
PORT="18801"
BASE="http://127.0.0.1:${PORT}/v1"
DATA_DIR="$(mktemp -d /tmp/spec82-offload.XXXXXX)"
LOG="/tmp/spec82-offload-${PORT}.log"
PID=""

cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$DATA_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"
cargo build -p focusa-api --bin focusa-daemon >/dev/null

FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$DATA_DIR" \
FOCUSA_SNAPSHOT_MAX=1 \
FOCUSA_SNAPSHOT_TTL_MINUTES=1440 \
FOCUSA_METACOG_MAX_CAPTURES=1 \
FOCUSA_METACOG_MAX_REFLECTIONS=1 \
FOCUSA_METACOG_MAX_ADJUSTMENTS=1 \
FOCUSA_METACOG_TTL_MINUTES=1440 \
"$BIN" >"$LOG" 2>&1 &
PID=$!

for _ in {1..120}; do
  if curl -fsS "$BASE/health" >/dev/null 2>&1; then break; fi
  sleep 1
done

post_json() {
  local path="$1"
  local body="$2"
  local raw
  raw=$(curl -sS -H 'content-type: application/json' -H 'x-focusa-permissions: admin:*' -X POST "$BASE$path" --data "$body" -w $'\n%{http_code}')
  local status body_json
  status=$(echo "$raw" | tail -n1)
  body_json=$(echo "$raw" | sed '$d')
  printf '%s\n%s\n' "$status" "$body_json"
}

# Snapshot offload fallback
mapfile -t out < <(post_json "/focus/snapshots" '{"snapshot_reason":"a","clt_node_id":"clt-test-a"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot A create http ${out[0]}"; exit 1; }
snap_a=$(echo "${out[1]}" | jq -r '.snapshot_id')

mapfile -t out < <(post_json "/focus/snapshots" '{"snapshot_reason":"b","clt_node_id":"clt-test-b"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot B create http ${out[0]}"; exit 1; }
snap_b=$(echo "${out[1]}" | jq -r '.snapshot_id')

mapfile -t out < <(post_json "/focus/snapshots/diff" "{\"from_snapshot_id\":\"$snap_a\",\"to_snapshot_id\":\"$snap_b\"}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: snapshot diff fallback http ${out[0]}"; echo "${out[1]}"; exit 1; }
echo "✓ PASS: snapshot diff works after in-memory eviction via disk fallback"

# Metacog capture retrieval fallback
mapfile -t out < <(post_json "/metacognition/capture" '{"kind":"note","content":"alpha memory signal"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: capture alpha http ${out[0]}"; exit 1; }

mapfile -t out < <(post_json "/metacognition/capture" '{"kind":"note","content":"beta memory signal"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: capture beta http ${out[0]}"; exit 1; }

mapfile -t out < <(post_json "/metacognition/retrieve" '{"current_ask":"alpha","scope_tags":[],"k":5}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: retrieve fallback http ${out[0]}"; exit 1; }
if echo "${out[1]}" | jq -e '.candidates | map(.summary) | join(" ") | contains("alpha memory signal")' >/dev/null; then
  echo "✓ PASS: metacog retrieve loads evicted capture from disk"
else
  echo "✗ FAIL: retrieve missing evicted capture content"
  echo "${out[1]}"
  exit 1
fi

# Reflection/adjustment existence fallback
mapfile -t out < <(post_json "/metacognition/reflect" '{"turn_range":"1..2"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: reflect r1 http ${out[0]}"; exit 1; }
r1=$(echo "${out[1]}" | jq -r '.reflection_id')

mapfile -t out < <(post_json "/metacognition/reflect" '{"turn_range":"3..4"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: reflect r2 http ${out[0]}"; exit 1; }

mapfile -t out < <(post_json "/metacognition/adjust" "{\"reflection_id\":\"$r1\",\"selected_updates\":[]}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: adjust should find evicted reflection on disk http ${out[0]}"; echo "${out[1]}"; exit 1; }
a1=$(echo "${out[1]}" | jq -r '.adjustment_id')

echo "✓ PASS: adjust works with evicted reflection via disk existence fallback"

mapfile -t out < <(post_json "/metacognition/adjust" "{\"reflection_id\":\"$r1\",\"selected_updates\":[\"x\"]}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: adjust second http ${out[0]}"; exit 1; }

mapfile -t out < <(post_json "/metacognition/evaluate" "{\"adjustment_id\":\"$a1\",\"observed_metrics\":[\"m1\"]}")
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: evaluate should find evicted adjustment on disk http ${out[0]}"; echo "${out[1]}"; exit 1; }

echo "✓ PASS: evaluate works with evicted adjustment via disk existence fallback"

echo "SPEC82 persistence offload runtime: PASS"

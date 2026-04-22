#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/focusa-daemon"
PORT="18802"
BASE="http://127.0.0.1:${PORT}/v1"
DATA_DIR="$(mktemp -d /tmp/spec82-paging.XXXXXX)"
LOG="/tmp/spec82-paging-${PORT}.log"
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
FOCUSA_METACOG_RETRIEVE_MAX_K=2 \
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

long_text() {
  python3 - <<'PY'
print('alpha-signal-' + ('x' * 320))
PY
}

TXT=$(long_text)
for n in 1 2 3; do
  mapfile -t out < <(post_json "/metacognition/capture" "{\"kind\":\"note\",\"content\":\"${TXT}${n}\"}")
  [[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: capture ${n} http ${out[0]}"; exit 1; }
done

mapfile -t out < <(post_json "/metacognition/retrieve" '{"current_ask":"alpha-signal","k":10,"summary_only":true}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: retrieve page1 http ${out[0]}"; exit 1; }

jq -e '.page_size == 2 and (.candidates | length) == 2 and .next_cursor == "2" and .retrieval_budget.truncated == true' <<<"${out[1]}" >/dev/null || {
  echo "✗ FAIL: page1 cap/pagination mismatch"
  echo "${out[1]}"
  exit 1
}

jq -e '(.candidates[0].summary | length) <= 240 and (.candidates[1].summary | length) <= 240' <<<"${out[1]}" >/dev/null || {
  echo "✗ FAIL: summary_only truncation mismatch"
  echo "${out[1]}"
  exit 1
}

echo "✓ PASS: retrieve page1 enforces cap + summary truncation"

mapfile -t out < <(post_json "/metacognition/retrieve" '{"current_ask":"alpha-signal","k":10,"summary_only":true,"cursor":"2"}')
[[ "${out[0]}" == "200" ]] || { echo "✗ FAIL: retrieve page2 http ${out[0]}"; exit 1; }

jq -e '.page_size == 2 and (.candidates | length) == 1 and .next_cursor == null and .candidates[0].rank == 3 and .retrieval_budget.truncated == false' <<<"${out[1]}" >/dev/null || {
  echo "✗ FAIL: page2 pagination mismatch"
  echo "${out[1]}"
  exit 1
}

echo "✓ PASS: retrieve cursor pagination returns final page"
echo "SPEC82 retrieve pagination caps runtime: PASS"

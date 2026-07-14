#!/usr/bin/env bash
# Compiled-product regression: hostile Focus Stack writes must not cross project/workstream scope.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="${FOCUSA_DAEMON_BIN:-$ROOT_DIR/target/x86_64-unknown-linux-musl/release/focusa-daemon}"
PORT="${FOCUSA_TEST_PORT:-18788}"
BASE="http://127.0.0.1:$PORT"
TMP="$(mktemp -d /tmp/focusa-hostile-scope-XXXXXX)"
PID=""
SOURCE_ROOT="$ROOT_DIR"
SOURCE_CONT="scope-write-source-$$"
HOSTILE_ROOT="${ROOT_DIR}-hostile"
HOSTILE_CONT="scope-write-hostile-$$"

cleanup() {
  if [[ -n "$PID" ]]; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  if command -v trash >/dev/null 2>&1; then
    trash "$TMP" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

[[ -x "$BIN" ]] || { echo "FAIL: daemon binary is not executable: $BIN" >&2; exit 1; }

FOCUSA_BIND="127.0.0.1:$PORT" \
FOCUSA_DATA_DIR="$TMP/data" \
FOCUSA_HOME="$TMP/home" \
FOCUSA_TEST_MODE=1 \
RUST_LOG=warn \
"$BIN" >"$TMP/daemon.log" 2>&1 &
PID=$!

ready=0
for _ in $(seq 1 40); do
  if curl --max-time 1 -fsS "$BASE/v1/health" >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 0.1
done
if [[ "$ready" -ne 1 ]]; then
  echo "FAIL: isolated daemon did not become healthy" >&2
  tail -40 "$TMP/daemon.log" >&2
  exit 1
fi

headers() {
  local root="$1" continuity="$2"
  printf '%s\n' \
    -H "content-type: application/json" \
    -H "X-Scope-Project-Root: $root" \
    -H "X-Scope-Continuity-Id: $continuity" \
    -H "X-Scope-Session-Id: hostile-scope-proof-$$"
}

request() {
  local method="$1" path="$2" root="$3" continuity="$4" payload="${5:-}"
  local args=(-X "$method" "$BASE$path" --max-time 2 -fsS)
  while IFS= read -r arg; do args+=("$arg"); done < <(headers "$root" "$continuity")
  [[ -n "$payload" ]] && args+=(--data "$payload")
  curl "${args[@]}"
}

session_payload=$(jq -nc \
  --arg root "$SOURCE_ROOT" --arg continuity "$SOURCE_CONT" \
  '{adapter_id:"hostile-scope-test",workspace_id:"focusa",project_root:$root,continuity_id:$continuity}')
request POST /v1/session/start "$SOURCE_ROOT" "$SOURCE_CONT" "$session_payload" >"$TMP/session.json"

push_payload=$(jq -nc \
  --arg root "$SOURCE_ROOT" --arg continuity "$SOURCE_CONT" \
  '{title:"hostile scope source",goal:"prove exact write isolation",beads_issue_id:"focusa-s6x25.2",tags:["integration","hostile-scope"],project_root:$root,continuity_id:$continuity}')
request POST /v1/focus/push "$SOURCE_ROOT" "$SOURCE_CONT" "$push_payload" >"$TMP/push.json"
FRAME_ID=$(jq -er 'select(.status=="accepted") | .frame_id' "$TMP/push.json")

stack_count() {
  request GET '/v1/focus/stack?limit=20' "$SOURCE_ROOT" "$SOURCE_CONT" | jq '.stack.frames | length'
}
assert_rejected() {
  local file="$1" label="$2"
  if [[ "$(jq -r '.status // ""' "$file")" == "accepted" ]]; then
    echo "FAIL: hostile $label was accepted" >&2
    cat "$file" >&2
    exit 1
  fi
}

BEFORE_COUNT=$(stack_count)

# Hostile project and continuity cannot push into the source body scope.
request POST /v1/focus/push "$HOSTILE_ROOT" "$HOSTILE_CONT" "$push_payload" >"$TMP/hostile-push.json"
assert_rejected "$TMP/hostile-push.json" push
[[ "$(stack_count)" -eq "$BEFORE_COUNT" ]] || { echo "FAIL: hostile push changed source stack" >&2; exit 1; }

hostile_update=$(jq -nc \
  --arg frame "$FRAME_ID" --arg root "$SOURCE_ROOT" --arg continuity "$SOURCE_CONT" \
  '{frame_id:$frame,project_root:$root,continuity_id:$continuity,delta:{current_state:"HOSTILE_MUTATION"}}')
request POST /v1/focus/update "$HOSTILE_ROOT" "$HOSTILE_CONT" "$hostile_update" >"$TMP/hostile-update.json"
assert_rejected "$TMP/hostile-update.json" update

# Same project but wrong continuity must also be rejected.
request POST /v1/focus/update "$SOURCE_ROOT" "$HOSTILE_CONT" "$hostile_update" >"$TMP/cross-continuity-update.json"
assert_rejected "$TMP/cross-continuity-update.json" cross-continuity-update

# Hostile pop cannot complete the source active frame.
request POST /v1/focus/pop "$HOSTILE_ROOT" "$HOSTILE_CONT" '{}' >"$TMP/hostile-pop.json"
assert_rejected "$TMP/hostile-pop.json" pop
source_stack=$(request GET '/v1/focus/stack?limit=20' "$SOURCE_ROOT" "$SOURCE_CONT")
printf '%s' "$source_stack" | jq -e --arg frame "$FRAME_ID" '.stack.frames[] | select(.id==$frame and .status=="active")' >/dev/null

# Exact-scope update and pop remain functional.
valid_update=$(jq -nc \
  --arg frame "$FRAME_ID" --arg root "$SOURCE_ROOT" --arg continuity "$SOURCE_CONT" \
  '{frame_id:$frame,project_root:$root,continuity_id:$continuity,delta:{current_state:"Exact scope proof"}}')
request POST /v1/focus/update "$SOURCE_ROOT" "$SOURCE_CONT" "$valid_update" >"$TMP/valid-update.json"
jq -e 'select(.status=="accepted")' "$TMP/valid-update.json" >/dev/null

# Root frames are intentionally non-completable; prove exact pop with a child frame.
child_payload=$(jq -nc \
  --arg root "$SOURCE_ROOT" --arg continuity "$SOURCE_CONT" \
  '{title:"exact scope child",goal:"prove exact pop",beads_issue_id:"focusa-s6x25.2",tags:["integration","exact-scope-child"],project_root:$root,continuity_id:$continuity}')
request POST /v1/focus/push "$SOURCE_ROOT" "$SOURCE_CONT" "$child_payload" >"$TMP/valid-child-push.json"
CHILD_ID=$(jq -er 'select(.status=="accepted") | .frame_id' "$TMP/valid-child-push.json")
request POST /v1/focus/pop "$SOURCE_ROOT" "$SOURCE_CONT" '{}' >"$TMP/valid-pop.json"
jq -e 'select(.status=="accepted")' "$TMP/valid-pop.json" >/dev/null

printf 'PASS: compiled daemon rejected hostile push/update/pop and allowed exact-scope writes (root=%s child=%s)\n' "$FRAME_ID" "$CHILD_ID"

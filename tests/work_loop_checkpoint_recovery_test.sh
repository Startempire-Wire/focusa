#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/debug/focusa-daemon}"
BIND_ADDR="${FOCUSA_CHECKPOINT_TEST_BIND:-127.0.0.1:18884}"
BASE_URL="http://${BIND_ADDR}"
PROJECT_ROOT="${REPO_ROOT}"
CONTINUITY_ID="checkpoint-recovery-live-test"
WORK_ITEM_ID="focusa-workloop-completion.4"
CHECKPOINT_ID="019f7446-6137-7560-8850-c9191580274b"
PARTIAL_RESPONSE_ID="019f7446-6137-7560-8850-c9191580274c"
DATA_DIR="$(mktemp -d /tmp/focusa-checkpoint-recovery.XXXXXX)"
LOG_FILE="${DATA_DIR}/daemon.log"

cleanup() {
  if [[ -n "${DAEMON_PID:-}" ]]; then
    kill "${DAEMON_PID}" 2>/dev/null || true
    wait "${DAEMON_PID}" 2>/dev/null || true
  fi
  rm -rf "${DATA_DIR}"
}
trap cleanup EXIT

start_daemon() {
  FOCUSA_BIND="${BIND_ADDR}" FOCUSA_DATA_DIR="${DATA_DIR}" \
    "${DAEMON_BIN}" >>"${LOG_FILE}" 2>&1 &
  DAEMON_PID=$!
  for _ in $(seq 1 100); do
    curl -fsS "${BASE_URL}/v1/health" >/dev/null 2>&1 && return
    sleep 0.1
  done
  cat "${LOG_FILE}" >&2
  return 1
}

scope_headers=(
  -H "x-scope-project-root: ${PROJECT_ROOT}"
  -H "x-scope-continuity-id: ${CONTINUITY_ID}"
)

enable_writer() {
  local writer_id="$1"
  curl -fsS -X POST "${BASE_URL}/v1/work-loop/enable" \
    "${scope_headers[@]}" -H 'content-type: application/json' \
    -H "x-focusa-writer-id: ${writer_id}" -H 'x-focusa-approval: approved' \
    -d "{\"preset\":\"balanced\",\"root_work_item_id\":\"${WORK_ITEM_ID}\"}"
}

checkpoint() {
  local writer_id="$1" token="$2" checkpoint_id="${3:-${CHECKPOINT_ID}}"
  curl -fsS -X POST "${BASE_URL}/v1/work-loop/checkpoint" \
    "${scope_headers[@]}" -H 'content-type: application/json' \
    -H "x-focusa-writer-id: ${writer_id}" \
    -H "x-focusa-fencing-token: ${token}" \
    -d "{\"checkpoint_id\":\"${checkpoint_id}\",\"summary\":\"atomic checkpoint recovery proof\"}"
}

[[ -x "${DAEMON_BIN}" ]] || { echo "missing daemon: ${DAEMON_BIN}" >&2; exit 1; }
start_daemon
workpoint_body="$(jq -n --arg root "${PROJECT_ROOT}" --arg continuity "${CONTINUITY_ID}" --arg work_item "${WORK_ITEM_ID}" '{project_root:$root,continuity_id:$continuity,mission:"checkpoint recovery live proof",current_action:"checkpoint_test",next_slice:"prove atomic replay",canonical:true,work_item_id:$work_item}')"
curl -fsS -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  "${scope_headers[@]}" -H 'content-type: application/json' \
  -d "${workpoint_body}" >/dev/null

first_enable="$(enable_writer checkpoint-writer-one)"
first_token="$(jq -er '.fencing_token' <<<"${first_enable}")"
first_checkpoint="$(checkpoint checkpoint-writer-one "${first_token}")"
jq -e '.ok == true and .idempotent_replay == false' <<<"${first_checkpoint}" >/dev/null

# Crash immediately after the accepted atomic checkpoint, preserving the same SQLite store.
kill -KILL "${DAEMON_PID}"
wait "${DAEMON_PID}" 2>/dev/null || true
DAEMON_PID=""
start_daemon

replacement_enable="$(enable_writer checkpoint-writer-two)"
replacement_token="$(jq -er '.fencing_token' <<<"${replacement_enable}")"
retry_checkpoint="$(checkpoint checkpoint-writer-two "${replacement_token}")"
jq -e '.ok == true and .idempotent_replay == true' <<<"${retry_checkpoint}" >/dev/null

# Event id and snapshot must both survive restart, with exactly one event application.
event_count="$(sqlite3 "${DATA_DIR}/focusa.sqlite" "SELECT COUNT(*) FROM events WHERE event_id='${CHECKPOINT_ID}';")"
[[ "${event_count}" == "1" ]]
state_json="$(sqlite3 "${DATA_DIR}/focusa.sqlite" "SELECT state_json FROM snapshots WHERE name='focusa';")"
event_payload="$(sqlite3 "${DATA_DIR}/focusa.sqlite" "SELECT payload_json FROM events WHERE event_id='${CHECKPOINT_ID}';")"
jq -e '.type == "ContinuousLoopRecoveryCheckpointed" and .summary == "atomic checkpoint recovery proof"' <<<"${event_payload}" >/dev/null
jq -e --arg root "${PROJECT_ROOT}" --arg continuity "${CONTINUITY_ID}" '
  .work_loop.run.last_checkpoint_id != null and
  .work_loop.execution_scope.root_scope.root_path == $root and
  .work_loop.execution_scope.continuity_id == $continuity
' <<<"${state_json}" >/dev/null

# Send a complete request and deliberately close the socket without reading any
# response. Retrying the same id must converge to exactly one durable event.
python3 - "${BIND_ADDR%:*}" "${BIND_ADDR##*:}" "${PROJECT_ROOT}" "${CONTINUITY_ID}" "${replacement_token}" "${PARTIAL_RESPONSE_ID}" <<'PY'
import json, socket, sys
host, port, root, continuity, token, checkpoint_id = sys.argv[1:]
body = json.dumps({"checkpoint_id": checkpoint_id, "summary": "atomic checkpoint recovery proof"}).encode()
request = (
    f"POST /v1/work-loop/checkpoint HTTP/1.1\r\nHost: {host}:{port}\r\n"
    f"Content-Type: application/json\r\nContent-Length: {len(body)}\r\n"
    f"x-scope-project-root: {root}\r\nx-scope-continuity-id: {continuity}\r\n"
    f"x-focusa-writer-id: checkpoint-writer-two\r\nx-focusa-fencing-token: {token}\r\n"
    "Connection: close\r\n\r\n"
).encode() + body
with socket.create_connection((host, int(port)), timeout=3) as sock:
    sock.sendall(request)
PY
sleep 0.3
partial_retry="$(checkpoint checkpoint-writer-two "${replacement_token}" "${PARTIAL_RESPONSE_ID}")"
jq -e '.ok == true' <<<"${partial_retry}" >/dev/null
partial_count="$(sqlite3 "${DATA_DIR}/focusa.sqlite" "SELECT COUNT(*) FROM events WHERE event_id='${PARTIAL_RESPONSE_ID}';")"
[[ "${partial_count}" == "1" ]]

jq -n \
  --arg checkpoint_id "${CHECKPOINT_ID}" \
  --arg partial_response_id "${PARTIAL_RESPONSE_ID}" \
  --argjson event_count "${event_count}" \
  --argjson partial_count "${partial_count}" \
  --argjson first_token "${first_token}" \
  --argjson replacement_token "${replacement_token}" \
  '{status:"passed",checkpoint_id:$checkpoint_id,event_count:$event_count,idempotent_retry:true,restart_recovered:true,partial_response_id:$partial_response_id,partial_response_event_count:$partial_count,first_token:$first_token,replacement_token:$replacement_token}'

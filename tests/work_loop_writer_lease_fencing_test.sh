#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${REPO_ROOT}/target/debug/focusa-daemon}"
BIND_ADDR="${FOCUSA_LEASE_TEST_BIND:-127.0.0.1:18883}"
BASE_URL="http://${BIND_ADDR}"
PROJECT_ROOT="${REPO_ROOT}"
CONTINUITY_ID="lease-fencing-live-test"
WORK_ITEM_ID="focusa-workloop-completion.3"
DATA_DIR="$(mktemp -d /tmp/focusa-lease-fencing.XXXXXX)"
LOG_FILE="${DATA_DIR}/daemon.log"

cleanup() {
  if [[ -n "${DAEMON_PID:-}" ]]; then
    kill "${DAEMON_PID}" 2>/dev/null || true
    wait "${DAEMON_PID}" 2>/dev/null || true
  fi
  rm -rf "${DATA_DIR}"
}
trap cleanup EXIT

if [[ ! -x "${DAEMON_BIN}" ]]; then
  echo "missing daemon binary: ${DAEMON_BIN}" >&2
  exit 1
fi

start_daemon() {
  FOCUSA_BIND="${BIND_ADDR}" FOCUSA_DATA_DIR="${DATA_DIR}" \
    "${DAEMON_BIN}" >>"${LOG_FILE}" 2>&1 &
  DAEMON_PID=$!
  for _ in $(seq 1 100); do
    if curl -fsS "${BASE_URL}/v1/health" >/dev/null 2>&1; then
      return
    fi
    sleep 0.1
  done
  cat "${LOG_FILE}" >&2
  return 1
}
start_daemon

scope_headers=(
  -H "x-scope-project-root: ${PROJECT_ROOT}"
  -H "x-scope-continuity-id: ${CONTINUITY_ID}"
)
workpoint_body="$(jq -n \
  --arg root "${PROJECT_ROOT}" \
  --arg continuity "${CONTINUITY_ID}" \
  --arg work_item "${WORK_ITEM_ID}" \
  '{project_root:$root,continuity_id:$continuity,mission:"writer lease fencing live proof",current_action:"lease_test",next_slice:"prove takeover fencing",canonical:true,work_item_id:$work_item}')"
curl -fsS -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  "${scope_headers[@]}" -H 'content-type: application/json' \
  -d "${workpoint_body}" >/dev/null

enable_writer() {
  local writer_id="$1"
  curl -fsS -X POST "${BASE_URL}/v1/work-loop/enable" \
    "${scope_headers[@]}" \
    -H 'content-type: application/json' \
    -H "x-focusa-writer-id: ${writer_id}" \
    -H 'x-focusa-approval: approved' \
    -d "{\"preset\":\"balanced\",\"root_work_item_id\":\"${WORK_ITEM_ID}\"}"
}

first_enable="$(enable_writer writer-one)"
first_token="$(jq -er '.fencing_token | select(. > 0 and . <= 9007199254740991)' <<<"${first_enable}")"
curl -fsS -X POST "${BASE_URL}/v1/work-loop/heartbeat" \
  "${scope_headers[@]}" \
  -H 'x-focusa-writer-id: writer-one' \
  -H "x-focusa-fencing-token: ${first_token}" >/dev/null
# Simulate an ungraceful owner/daemon crash. Claims are intentionally lost, while
# reducer-owned execution scope and Workpoint state recover from the same data dir.
kill -KILL "${DAEMON_PID}"
wait "${DAEMON_PID}" 2>/dev/null || true
DAEMON_PID=""
start_daemon

replacement_enable="$(enable_writer writer-two)"
replacement_token="$(jq -er '.fencing_token | select(. > 0 and . <= 9007199254740991)' <<<"${replacement_enable}")"
if (( replacement_token <= first_token )); then
  echo "replacement fencing token did not increase" >&2
  exit 1
fi

late_status="$(curl -sS -o "${DATA_DIR}/late-writer.json" -w '%{http_code}' \
  -X POST "${BASE_URL}/v1/work-loop/pause" \
  "${scope_headers[@]}" -H 'content-type: application/json' \
  -H 'x-focusa-writer-id: writer-one' \
  -H "x-focusa-fencing-token: ${first_token}" \
  -d '{"reason":"late stale writer"}')"
if [[ "${late_status}" != "409" ]]; then
  echo "late writer was not fenced: HTTP ${late_status}" >&2
  cat "${DATA_DIR}/late-writer.json" >&2
  exit 1
fi

curl -fsS -X POST "${BASE_URL}/v1/work-loop/pause" \
  "${scope_headers[@]}" -H 'content-type: application/json' \
  -H 'x-focusa-writer-id: writer-two' \
  -H "x-focusa-fencing-token: ${replacement_token}" \
  -d '{"reason":"replacement writer verified"}' >/dev/null

jq -n \
  --argjson first_token "${first_token}" \
  --argjson replacement_token "${replacement_token}" \
  --argjson late_writer_status "${late_status}" \
  '{status:"passed",takeover:"after_forced_daemon_crash",first_token:$first_token,replacement_token:$replacement_token,late_writer_status:$late_writer_status}'

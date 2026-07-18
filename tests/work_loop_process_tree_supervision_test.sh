#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" == "MINGW"* || "$(uname -s)" == "MSYS"* ]]; then
  echo '{"status":"skipped","reason":"Unix watchdog proof; Windows uses taskkill /T /F"}'
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${DAEMON_BIN:-${ROOT}/target/debug/focusa-daemon}"
BIND="${FOCUSA_PROCESS_TREE_TEST_BIND:-127.0.0.1:18885}"
BASE="http://${BIND}"
TMP="$(mktemp -d /tmp/focusa-process-tree.XXXXXX)"
CONTINUITY="process-tree-supervision-test"
WORK_ITEM="focusa-workloop-completion.8"
WRITER="process-tree-writer"
cleanup() {
  [[ -n "${DAEMON_PID:-}" ]] && kill "${DAEMON_PID}" 2>/dev/null || true
  [[ -n "${DAEMON_PID:-}" ]] && wait "${DAEMON_PID}" 2>/dev/null || true
  rm -rf "${TMP}"
}
trap cleanup EXIT

cat >"${TMP}/fake-pi" <<'SH'
#!/usr/bin/env bash
echo "$$" >"${FOCUSA_TEST_LEADER_PID_FILE}"
sleep 300 &
echo "$!" >"${FOCUSA_TEST_DESCENDANT_PID_FILE}"
if [[ -f "${FOCUSA_TEST_EXIT_AFTER_INIT_FILE}" ]]; then
  sleep 0.2
  exit 0
fi
while IFS= read -r line; do
  id="$(jq -r '.id // empty' <<<"${line}")"
  if [[ -n "${id}" ]]; then
    printf '{"jsonrpc":"2.0","id":%s,"result":{"ok":true}}\n' "${id}"
  fi
  [[ -f "${FOCUSA_TEST_EXIT_AFTER_INIT_FILE}" ]] && exit 0
done
SH
chmod +x "${TMP}/fake-pi"

FOCUSA_BIND="${BIND}" FOCUSA_DATA_DIR="${TMP}/data" FOCUSA_PI_BIN="${TMP}/fake-pi" \
FOCUSA_TEST_LEADER_PID_FILE="${TMP}/leader.pid" \
FOCUSA_TEST_DESCENDANT_PID_FILE="${TMP}/descendant.pid" \
FOCUSA_TEST_EXIT_AFTER_INIT_FILE="${TMP}/exit-after-init" \
  "${BIN}" >"${TMP}/daemon.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do
  curl -fsS "${BASE}/v1/health" >/dev/null 2>&1 && break
  sleep 0.1
done
scope=(-H "x-scope-project-root: ${ROOT}" -H "x-scope-continuity-id: ${CONTINUITY}")
curl -fsS -X POST "${BASE}/v1/workpoint/checkpoint" "${scope[@]}" -H 'content-type: application/json' \
  -d "$(jq -n --arg root "${ROOT}" --arg continuity "${CONTINUITY}" --arg item "${WORK_ITEM}" '{project_root:$root,continuity_id:$continuity,mission:"process tree proof",current_action:"supervision_test",next_slice:"prove crash cleanup",canonical:true,work_item_id:$item}')" >/dev/null
enable="$(curl -fsS -X POST "${BASE}/v1/work-loop/enable" "${scope[@]}" -H 'content-type: application/json' -H "x-focusa-writer-id: ${WRITER}" -H 'x-focusa-approval: approved' -d "{\"root_work_item_id\":\"${WORK_ITEM}\"}")"
token="$(jq -er '.fencing_token' <<<"${enable}")"
curl -fsS -X POST "${BASE}/v1/work-loop/driver/start" "${scope[@]}" -H 'content-type: application/json' -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" -d "$(jq -n --arg cwd "${ROOT}" '{cwd:$cwd}')" >/dev/null
for _ in $(seq 1 100); do
  [[ -s "${TMP}/leader.pid" && -s "${TMP}/descendant.pid" ]] && break
  sleep 0.1
done
leader="$(cat "${TMP}/leader.pid")"
descendant="$(cat "${TMP}/descendant.pid")"
wait_for_tree_exit() {
  local leader_pid="$1" descendant_pid="$2"
  for _ in $(seq 1 40); do
    leader_alive=0; descendant_alive=0
    kill -0 "${leader_pid}" 2>/dev/null && [[ "$(ps -o stat= -p "${leader_pid}" 2>/dev/null)" != Z* ]] && leader_alive=1
    kill -0 "${descendant_pid}" 2>/dev/null && [[ "$(ps -o stat= -p "${descendant_pid}" 2>/dev/null)" != Z* ]] && descendant_alive=1
    [[ "${leader_alive}" -eq 0 && "${descendant_alive}" -eq 0 ]] && return 0
    sleep 0.2
  done
  ps -o pid=,ppid=,pgid=,stat=,command= -p "${leader_pid},${descendant_pid}" >&2 || true
  return 1
}

curl -fsS -X POST "${BASE}/v1/work-loop/driver/stop" "${scope[@]}" \
  -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" | jq -e '.process_tree_terminated == true' >/dev/null
wait_for_tree_exit "${leader}" "${descendant}"

rm -f "${TMP}/leader.pid" "${TMP}/descendant.pid"
curl -fsS -X POST "${BASE}/v1/work-loop/driver/start" "${scope[@]}" -H 'content-type: application/json' \
  -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" -d "$(jq -n --arg cwd "${ROOT}" '{cwd:$cwd}')" >/dev/null
for _ in $(seq 1 100); do [[ -s "${TMP}/leader.pid" && -s "${TMP}/descendant.pid" ]] && break; sleep 0.1; done
leader="$(cat "${TMP}/leader.pid")"; descendant="$(cat "${TMP}/descendant.pid")"
curl -fsS -X POST "${BASE}/v1/work-loop/driver/abort" "${scope[@]}" \
  -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" | jq -e '.process_tree_terminated == true' >/dev/null
wait_for_tree_exit "${leader}" "${descendant}"

# Transport EOF must clear the stale session and terminate the remaining process group.
touch "${TMP}/exit-after-init"
rm -f "${TMP}/leader.pid" "${TMP}/descendant.pid"
curl -fsS -X POST "${BASE}/v1/work-loop/driver/start" "${scope[@]}" -H 'content-type: application/json' \
  -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" -d "$(jq -n --arg cwd "${ROOT}" '{cwd:$cwd}')" >/dev/null
for _ in $(seq 1 100); do [[ -s "${TMP}/leader.pid" && -s "${TMP}/descendant.pid" ]] && break; sleep 0.1; done
leader="$(cat "${TMP}/leader.pid")"; descendant="$(cat "${TMP}/descendant.pid")"
wait_for_tree_exit "${leader}" "${descendant}"
for _ in $(seq 1 50); do
  status="$(curl -fsS "${BASE}/v1/work-loop/status?summary_only=true" "${scope[@]}")"
  [[ "$(jq -r '.transport.daemon_supervised_session // empty' <<<"${status}")" == "" ]] && break
  sleep 0.1
done
[[ "$(jq -r '.transport.daemon_supervised_session // empty' <<<"${status}")" == "" ]]
rm -f "${TMP}/exit-after-init"

rm -f "${TMP}/leader.pid" "${TMP}/descendant.pid"
curl -fsS -X POST "${BASE}/v1/work-loop/driver/start" "${scope[@]}" -H 'content-type: application/json' \
  -H "x-focusa-writer-id: ${WRITER}" -H "x-focusa-fencing-token: ${token}" -d "$(jq -n --arg cwd "${ROOT}" '{cwd:$cwd}')" >/dev/null
for _ in $(seq 1 100); do [[ -s "${TMP}/leader.pid" && -s "${TMP}/descendant.pid" ]] && break; sleep 0.1; done
leader="$(cat "${TMP}/leader.pid")"; descendant="$(cat "${TMP}/descendant.pid")"
kill -9 "${DAEMON_PID}"
wait "${DAEMON_PID}" 2>/dev/null || true
DAEMON_PID=""
wait_for_tree_exit "${leader}" "${descendant}"
printf '{"status":"passed","leader_pid":%s,"descendant_pid":%s,"cleanup":["stop","abort","transport_eof","daemon_sigkill"]}\n' "${leader}" "${descendant}"

#!/bin/bash
# Spec98 runtime proof: two daemon instances reconcile same-root CRDT events via HTTP sync export/import.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-${ROOT_DIR}/target/release/focusa-daemon}"
CARGO_BIN="${CARGO:-$(command -v cargo || command -v /root/.cargo/bin/cargo || command -v /usr/bin/cargo || true)}"
PROJECT_ROOT_KEY="${PROJECT_ROOT_KEY:-${ROOT_DIR}}"
WORKSTREAM_KEY="${WORKSTREAM_KEY:-spec98-runtime-crdt}"
DATA_A="$(mktemp -d /tmp/focusa-spec98-crdt-a.XXXXXX)"
DATA_B="$(mktemp -d /tmp/focusa-spec98-crdt-b.XXXXXX)"
BODY_A="$(mktemp /tmp/focusa-spec98-crdt-a-body.XXXXXX.json)"
BODY_B="$(mktemp /tmp/focusa-spec98-crdt-b-body.XXXXXX.json)"
EXPORT_A="$(mktemp /tmp/focusa-spec98-crdt-export-a.XXXXXX.json)"
EXPORT_B="$(mktemp /tmp/focusa-spec98-crdt-export-b.XXXXXX.json)"
FINAL_A="$(mktemp /tmp/focusa-spec98-crdt-final-a.XXXXXX.json)"
FINAL_B="$(mktemp /tmp/focusa-spec98-crdt-final-b.XXXXXX.json)"
WRONG_SCOPE="$(mktemp /tmp/focusa-spec98-crdt-wrong-scope.XXXXXX.json)"
PID_A=""
PID_B=""

cleanup() {
  [ -n "$PID_A" ] && kill "$PID_A" >/dev/null 2>&1 || true
  [ -n "$PID_B" ] && kill "$PID_B" >/dev/null 2>&1 || true
  rm -rf "$DATA_A" "$DATA_B" "$BODY_A" "$BODY_B" "$EXPORT_A" "$EXPORT_B" "$FINAL_A" "$FINAL_B" "$WRONG_SCOPE" >/dev/null 2>&1 || true
}
trap cleanup EXIT

free_port() {
  python3 - <<'PY'
import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

PORT_A="$(free_port)"
PORT_B="$(free_port)"
BASE_A="http://127.0.0.1:${PORT_A}"
BASE_B="http://127.0.0.1:${PORT_B}"

if [ -z "$CARGO_BIN" ]; then echo "cargo not found" >&2; exit 1; fi
"$CARGO_BIN" build -p focusa-api --release --bin focusa-daemon

FOCUSA_BIND="127.0.0.1:${PORT_A}" FOCUSA_DATA_DIR="$DATA_A" "$DAEMON_BIN" >/tmp/focusa-spec98-crdt-a.log 2>&1 &
PID_A=$!
FOCUSA_BIND="127.0.0.1:${PORT_B}" FOCUSA_DATA_DIR="$DATA_B" "$DAEMON_BIN" >/tmp/focusa-spec98-crdt-b.log 2>&1 &
PID_B=$!

wait_health() {
  local base="$1"
  for _ in $(seq 1 80); do
    if curl -fsS "${base}/v1/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  echo "daemon failed health: ${base}" >&2
  return 1
}

wait_health "$BASE_A"
wait_health "$BASE_B"

make_seed_body() {
  local peer="$1"
  local machine="$2"
  local key="$3"
  local value="$4"
  local lamport="$5"
  python3 - "$peer" "$PROJECT_ROOT_KEY" "$WORKSTREAM_KEY" "$machine" "$key" "$value" "$lamport" <<'PY'
import datetime, json, sys, uuid
peer, project, workstream, machine, key, value, lamport = sys.argv[1:]
now = datetime.datetime.now(datetime.UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")
event = {
    "entry": {
        "id": str(uuid.uuid4()),
        "timestamp": now,
        "type": "SemanticMemoryUpserted",
        "key": key,
        "value": value,
        "source": "spec98-runtime-multi-daemon-crdt-sync",
        "correlation_id": f"project_root={project}|continuity_id={workstream}",
        "origin": "daemon",
        "machine_id": machine,
        "is_observation": False,
    },
    "vector_clock": {"clocks": {machine: int(lamport)}},
    "lamport_ts": int(lamport),
}
json.dump({"peer_id": peer, "project_root_key": project, "workstream_key": workstream, "events": [event]}, sys.stdout)
PY
}

make_seed_body seed-a daemon-a spec98-daemon-a "event from daemon A" 1 > "$BODY_A"
make_seed_body seed-b daemon-b spec98-daemon-b "event from daemon B" 1 > "$BODY_B"

curl -fsS -X POST "${BASE_A}/v1/sync/crdt/import" -H 'Content-Type: application/json' -d "@$BODY_A" | jq -e '.status == "ok" and .imported == 1' >/dev/null
curl -fsS -X POST "${BASE_B}/v1/sync/crdt/import" -H 'Content-Type: application/json' -d "@$BODY_B" | jq -e '.status == "ok" and .imported == 1' >/dev/null

ENC_PROJECT="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$PROJECT_ROOT_KEY")"
ENC_WORK="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$WORKSTREAM_KEY")"

curl -fsS "${BASE_A}/v1/sync/crdt/export?project_root_key=${ENC_PROJECT}&workstream_key=${ENC_WORK}&limit=100" > "$EXPORT_A"
curl -fsS "${BASE_B}/v1/sync/crdt/export?project_root_key=${ENC_PROJECT}&workstream_key=${ENC_WORK}&limit=100" > "$EXPORT_B"
jq -e '.status == "ok" and .count == 1' "$EXPORT_A" >/dev/null
jq -e '.status == "ok" and .count == 1' "$EXPORT_B" >/dev/null

jq -c --arg peer daemon-b --arg project "$PROJECT_ROOT_KEY" --arg workstream "$WORKSTREAM_KEY" '{peer_id:$peer, project_root_key:$project, workstream_key:$workstream, events:.events}' "$EXPORT_B" \
  | curl -fsS -X POST "${BASE_A}/v1/sync/crdt/import" -H 'Content-Type: application/json' -d @- \
  | jq -e '.status == "ok" and .imported == 1 and .considered == 1' >/dev/null
jq -c --arg peer daemon-a --arg project "$PROJECT_ROOT_KEY" --arg workstream "$WORKSTREAM_KEY" '{peer_id:$peer, project_root_key:$project, workstream_key:$workstream, events:.events}' "$EXPORT_A" \
  | curl -fsS -X POST "${BASE_B}/v1/sync/crdt/import" -H 'Content-Type: application/json' -d @- \
  | jq -e '.status == "ok" and .imported == 1 and .considered == 1' >/dev/null

curl -fsS "${BASE_A}/v1/sync/crdt/export?project_root_key=${ENC_PROJECT}&workstream_key=${ENC_WORK}&limit=100" > "$FINAL_A"
curl -fsS "${BASE_B}/v1/sync/crdt/export?project_root_key=${ENC_PROJECT}&workstream_key=${ENC_WORK}&limit=100" > "$FINAL_B"

jq -e '.count == 2' "$FINAL_A" >/dev/null
jq -e '.count == 2' "$FINAL_B" >/dev/null

IDS_A="$(jq -r '.events[].entry.id' "$FINAL_A" | sort | tr '\n' ' ')"
IDS_B="$(jq -r '.events[].entry.id' "$FINAL_B" | sort | tr '\n' ' ')"
if [ "$IDS_A" != "$IDS_B" ]; then
  echo "CRDT exports did not converge" >&2
  echo "A: $IDS_A" >&2
  echo "B: $IDS_B" >&2
  exit 1
fi

jq -c --arg peer wrong-scope --arg project "/tmp/not-focusa" --arg workstream "$WORKSTREAM_KEY" '{peer_id:$peer, project_root_key:$project, workstream_key:$workstream, events:.events}' "$FINAL_A" > "$WRONG_SCOPE"
curl -fsS -X POST "${BASE_B}/v1/sync/crdt/import" -H 'Content-Type: application/json' -d "@$WRONG_SCOPE" \
  | jq -e '.status == "ok" and .imported == 0 and .skipped == 2' >/dev/null

printf '✓ PASS: Spec98 runtime multi-daemon same-root CRDT sync converged via HTTP export/import (%s)\n' "$IDS_A"

#!/usr/bin/env bash
# Spec 109 / focusa-ux2qx.10 — advertised agent, memory, and turn route parity.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
FIXTURE="$(mktemp -d)"
PORT="${FOCUSA_AGENT_TOOLS_TEST_PORT:-18797}"
BASE="http://127.0.0.1:${PORT}"
DAEMON_PID=""
cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  rm -rf "$FIXTURE"
}
trap cleanup EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }

cargo build -q -p focusa-api --bin focusa-daemon
HOME="$FIXTURE/home" FOCUSA_DATA_DIR="$FIXTURE/data" FOCUSA_BIND="127.0.0.1:${PORT}" \
  "$ROOT/target/debug/focusa-daemon" >"$FIXTURE/daemon.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 120); do
  curl -fsS --max-time 1 "$BASE/v1/health" >/dev/null 2>&1 && break
  sleep 0.1
done
curl -fsS --max-time 2 "$BASE/v1/health" >/dev/null \
  || { tail -100 "$FIXTURE/daemon.log" >&2; fail "isolated daemon did not become healthy"; }

curl -fsS "$BASE/v1/agent/tools" > "$FIXTURE/tools.json"
jq -e '
  .schema=="focusa.agent_capabilities.index.v1" and
  (.families|index("turn"))!=null and
  (.families|index("memory"))!=null and
  ([.operations[].path] | contains([
    "/v1/turn/start", "/v1/turn/append", "/v1/turn/complete",
    "/v1/memory/semantic", "/v1/memory/semantic/upsert",
    "/v1/memory/procedural", "/v1/memory/procedural/reinforce"
  ]))
' "$FIXTURE/tools.json" >/dev/null \
  || { cat "$FIXTURE/tools.json" >&2; fail "agent tools index missing turn/memory operations"; }

curl -fsS "$BASE/v1/agent/schemas" > "$FIXTURE/schemas.json"
jq -e '
  .schema=="focusa.agent_schema_index.v1" and
  (.schemas|index("focusa.turn_start.request.v1"))!=null and
  (.schemas|index("focusa.turn_start.response.v1"))!=null and
  (.schemas|index("focusa.turn_append.request.v1"))!=null and
  (.schemas|index("focusa.turn_complete.response.v1"))!=null
' "$FIXTURE/schemas.json" >/dev/null \
  || { cat "$FIXTURE/schemas.json" >&2; fail "turn schemas missing from agent schema index"; }

curl -fsS "$BASE/v1/agent/schemas/focusa.turn_start.request.v1" > "$FIXTURE/schema.json"
jq -e '.schema_id=="focusa.turn_start.request.v1"' "$FIXTURE/schema.json" >/dev/null \
  || { cat "$FIXTURE/schema.json" >&2; fail "turn schema detail unavailable"; }

for route in /v1/memory/semantic /v1/memory/procedural; do
  code="$(curl -sS -o "$FIXTURE/memory.json" -w '%{http_code}' "$BASE$route")"
  [[ "$code" == "200" ]] \
    || { cat "$FIXTURE/memory.json" >&2; fail "$route returned HTTP $code"; }
done

curl -fsS "$BASE/v1/openapi.json" > "$FIXTURE/openapi.json"
jq -e '
  .paths["/v1/turn/start"].post.operationId=="focusa.turn.start" and
  .paths["/v1/memory/semantic"].get.operationId=="focusa.memory.semantic.read"
' "$FIXTURE/openapi.json" >/dev/null \
  || { cat "$FIXTURE/openapi.json" >&2; fail "OpenAPI missing turn/memory operations"; }

echo "PASS: agent tools, turn schemas, memory routes, and OpenAPI are reconciled"

#!/usr/bin/env bash
# phone_bridge_callback_e2e_live_test.sh — V2 P1 #11.
#
# Behavior-based end-to-end test for the mac_callback fast path.
#
#   1. Start a fresh focusa-daemon on a free loopback port.
#   2. Open an ephemeral HTTP listener (python3 -m http.server is enough
#      for "is the callback reachable" but we need a listener that
#      ACCEPTS the POST and records the body. We use a small Python
#      inline script for that.)
#   3. POST /v1/connect/room/create on the daemon.
#   4. POST /v1/connect/room/{id}/join with mac_callback pointing at the
#      ephemeral listener.
#   5. POST /v1/connect/room/approve with a Mac name.
#   6. Assert the listener received a POST whose body contains the
#      protocol=focusa-connect-v1, role=mac_completion_payload marker.
#   7. Tear down.
#
# This replaces the grep-only static test in
# tests/phone_bridge_automatic_callback_static_test.sh with a real
# network-level proof. Pass criteria: callback body contains
# "mac_completion_payload" AND "token" (non-empty).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${TMPDIR:-/tmp}/focusa-cb-e2e-$$"
mkdir -p "$LOG_DIR"
trap 'rm -rf "$LOG_DIR"' EXIT

DAEMON_BIN="$ROOT_DIR/target/debug/focusa-daemon"
if [ ! -x "$DAEMON_BIN" ]; then
  echo "✗ FAIL: $DAEMON_BIN not built. Run: cargo build --workspace" >&2
  exit 1
fi

PORT_DAEMON=18977
PORT_LISTENER=18978
DATA_DIR="$LOG_DIR/data"
mkdir -p "$DATA_DIR"

# 1. Ephemeral HTTP listener: listens for one POST, records body to file.
LISTENER_LOG="$LOG_DIR/listener.log"
LISTENER_BODY="$LOG_DIR/listener_body.json"
python3 - "$PORT_LISTENER" "$LISTENER_BODY" >"$LISTENER_LOG" 2>&1 <<'PY' &
import http.server, json, sys, threading
PORT = int(sys.argv[1])
BODY_PATH = sys.argv[2]
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a, **kw): pass
    def do_POST(self):
        n = int(self.headers.get('Content-Length','0'))
        body = self.rfile.read(n)
        with open(BODY_PATH, 'wb') as f: f.write(body)
        self.send_response(200)
        self.send_header('Content-Type','application/json')
        self.end_headers()
        self.wfile.write(b'{"received":true}')
srv = http.server.HTTPServer(('127.0.0.1', PORT), H)
srv.serve_forever()
PY
LISTENER_PID=$!
sleep 1
if ! kill -0 "$LISTENER_PID" 2>/dev/null; then
  echo "✗ FAIL: listener did not start. log:"; cat "$LISTENER_LOG" >&2
  exit 1
fi

# 2. Start daemon.
export FOCUSA_BIND="127.0.0.1:$PORT_DAEMON"
export FOCUSA_DATA_DIR="$DATA_DIR"
export RUST_LOG="warn,focusa_api=info"
nohup "$DAEMON_BIN" >"$LOG_DIR/daemon.log" 2>&1 &
DAEMON_PID=$!
trap 'rm -rf "$LOG_DIR"; kill $LISTENER_PID 2>/dev/null; kill $DAEMON_PID 2>/dev/null; true' EXIT

for i in $(seq 1 20); do
  if curl -fs -m 1 "http://127.0.0.1:$PORT_DAEMON/v1/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done
if ! curl -fs -m 1 "http://127.0.0.1:$PORT_DAEMON/v1/health" >/dev/null 2>&1; then
  echo "✗ FAIL: daemon not healthy. log tail:"; tail -10 "$LOG_DIR/daemon.log" >&2
  exit 1
fi

# 3. Create + join + approve.
ROOM=$(curl -fs -m 5 -X POST "http://127.0.0.1:$PORT_DAEMON/v1/connect/room/create" \
  -H 'Content-Type: application/json' \
  -d "{\"server_url\":\"http://127.0.0.1:$PORT_DAEMON\"}" \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)["connect_id"])')
echo "  room=$ROOM"

curl -fs -m 5 -X POST "http://127.0.0.1:$PORT_DAEMON/v1/connect/room/$ROOM/join" \
  -H 'Content-Type: application/json' \
  -d "{\"mac_name\":\"op-mac\",\"mac_nonce\":\"deadbeef0001\",\"mac_pubkey\":null,\"mac_callback\":\"http://127.0.0.1:$PORT_LISTENER/focusa-phone-bridge/test-nonce\"}" \
  >/dev/null
echo "  joined (with mac_callback)"

APPROVE_JSON=$(curl -fs -m 5 -X POST "http://127.0.0.1:$PORT_DAEMON/v1/connect/approve" \
  -H 'Content-Type: application/json' \
  -d "{\"connect_id\":\"$ROOM\",\"host\":\"op-vps\"}")
echo "  approve response: $APPROVE_JSON"

# 4. Wait briefly for the server-side dispatch (best-effort fire-and-forget).
for i in $(seq 1 20); do
  if [ -s "$LISTENER_BODY" ]; then break; fi
  sleep 0.25
done

# 5. Assertions.
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
[ -s "$LISTENER_BODY" ] || fail "listener did not receive any POST body. daemon log:"; tail -15 "$LOG_DIR/daemon.log" >&2

BODY=$(cat "$LISTENER_BODY")
echo "  callback body: $BODY"
echo "$BODY" | python3 -c "
import json, sys
b = json.load(sys.stdin)
assert b.get('protocol') == 'focusa-connect-v1', 'protocol mismatch: ' + str(b.get('protocol'))
assert b.get('role') == 'mac_completion_payload', 'role mismatch: ' + str(b.get('role'))
assert b.get('token') and len(b['token']) > 20, 'token missing or short'
assert b.get('device_id'), 'device_id missing'
assert b.get('connect_id'), 'connect_id missing'
assert b.get('token_expires_at'), 'token_expires_at missing'
print('  callback payload schema: OK')
" || fail "callback payload failed schema assertions"

# 6. mac_callback_dispatched should be true in the /approve response.
echo "$APPROVE_JSON" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('mac_callback_dispatched') == True, 'mac_callback_dispatched not true: ' + str(d.get('mac_callback_dispatched'))
assert d.get('mac_receives_token_via') == 'mac_callback', 'unexpected mac_receives_token_via: ' + str(d.get('mac_receives_token_via'))
print('  approve response: mac_callback_dispatched=true OK')
" || fail "approve response did not report callback dispatched"

echo
echo "✓ ALL V2 CALLBACK E2E ASSERTIONS PASSED"
echo "  - ephemeral listener received POST with role=mac_completion_payload"
echo "  - body contained non-empty token + device_id + connect_id + token_expires_at"
echo "  - /approve response reported mac_callback_dispatched=true"
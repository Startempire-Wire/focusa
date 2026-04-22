#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR=$(mktemp -d)
HITS_FILE="$TMP_DIR/hits.jsonl"
PORT_FILE="$TMP_DIR/port.txt"
SERVER_LOG="$TMP_DIR/server.log"
OUT1="$TMP_DIR/pi-head.out"
OUT2="$TMP_DIR/pi-diff.out"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

HITS_FILE="$HITS_FILE" PORT_FILE="$PORT_FILE" python3 - <<'PY' >"$SERVER_LOG" 2>&1 &
import json, os
from http.server import BaseHTTPRequestHandler, HTTPServer

hits_file = os.environ['HITS_FILE']
port_file = os.environ['PORT_FILE']

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        return
    def _write(self, payload, status=200):
        body = json.dumps(payload).encode('utf-8')
        self.send_response(status)
        self.send_header('content-type', 'application/json')
        self.send_header('content-length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def do_GET(self):
        with open(hits_file, 'a', encoding='utf-8') as f:
            f.write(json.dumps({'method':'GET','path':self.path}) + '\n')
        path = self.path.split('?',1)[0]
        if path == '/v1/lineage/head':
            return self._write({'head':'clt-head-eval','branch_id':'main','session_id':'sess-eval'})
        if path == '/v1/work-loop/status':
            return self._write({'status':'running','writer_id':'writer-test'})
        return self._write({'status':'error','code':'NOT_FOUND'}, 404)
    def do_POST(self):
        length = int(self.headers.get('content-length','0'))
        raw = self.rfile.read(length).decode('utf-8') if length else ''
        try:
            body = json.loads(raw) if raw else None
        except Exception:
            body = raw
        with open(hits_file, 'a', encoding='utf-8') as f:
            f.write(json.dumps({'method':'POST','path':self.path,'body':body}) + '\n')
        if self.path == '/v1/focus/snapshots/diff':
            return self._write({
                'status':'ok',
                'from_snapshot_id': body.get('from_snapshot_id'),
                'to_snapshot_id': body.get('to_snapshot_id'),
                'checksum_changed': True,
                'clt_node_changed': False,
                'version_delta': 2,
                'decisions_delta': {'changed': True},
                'constraints_delta': {'changed': True},
            })
        return self._write({'status':'error','code':'NOT_FOUND'}, 404)

server = HTTPServer(('127.0.0.1', 0), Handler)
with open(port_file, 'w', encoding='utf-8') as f:
    f.write(str(server.server_port))
server.serve_forever()
PY
SERVER_PID=$!

for _ in {1..50}; do
  [[ -s "$PORT_FILE" ]] && break
  sleep 0.1
done
PORT=$(cat "$PORT_FILE")
BASE_URL="http://127.0.0.1:${PORT}/v1"

cd "$ROOT_DIR"

FOCUSA_PI_API_BASE_URL="$BASE_URL" timeout 90 /usr/local/bin/pi \
  --no-session --print --no-context-files --no-prompt-templates --no-skills --no-extensions \
  -e "$ROOT_DIR/apps/pi-extension/src/index.ts" \
  -p "Use the focusa_tree_head tool and answer with only the current lineage head id." \
  >"$OUT1" 2>/dev/null || true

FOCUSA_PI_API_BASE_URL="$BASE_URL" timeout 90 /usr/local/bin/pi \
  --no-session --print --no-context-files --no-prompt-templates --no-skills --no-extensions \
  -e "$ROOT_DIR/apps/pi-extension/src/index.ts" \
  -p "Use the focusa_tree_diff_context tool to compare snapshots snap-a and snap-b. Answer in one short sentence." \
  >"$OUT2" 2>/dev/null || true

if ! rg -n '"path": "/v1/lineage/head"' "$HITS_FILE" >/dev/null 2>&1; then
  echo "SPEC81 pickup smoke failed: tree_head tool was not called" >&2
  cat "$SERVER_LOG" >&2 || true
  exit 1
fi

if ! rg -n '"path": "/v1/focus/snapshots/diff"' "$HITS_FILE" >/dev/null 2>&1; then
  echo "SPEC81 pickup smoke failed: tree_diff tool was not called" >&2
  cat "$SERVER_LOG" >&2 || true
  exit 1
fi

if ! rg -n 'clt-head-eval' "$OUT1" >/dev/null 2>&1; then
  echo "SPEC81 pickup smoke failed: head answer did not include expected id" >&2
  echo '--- output 1 ---' >&2
  cat "$OUT1" >&2 || true
  exit 1
fi

if ! rg -n 'changed|different|diff' "$OUT2" >/dev/null 2>&1; then
  echo "SPEC81 pickup smoke failed: diff answer looked wrong" >&2
  echo '--- output 2 ---' >&2
  cat "$OUT2" >&2 || true
  exit 1
fi

echo "SPEC81 tool pickup smoke: PASS"

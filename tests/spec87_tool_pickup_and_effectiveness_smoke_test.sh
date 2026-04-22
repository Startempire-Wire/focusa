#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR=$(mktemp -d)
HITS_FILE="$TMP_DIR/hits.jsonl"
PORT_FILE="$TMP_DIR/port.txt"
SERVER_LOG="$TMP_DIR/server.log"
OUT_REFL="$TMP_DIR/pi-refl.out"
OUT_DOC="$TMP_DIR/pi-doc.out"

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
        if path == '/v1/metacognition/reflections/recent':
            return self._write({'status':'ok','total':1,'reflections':[{'reflection_id':'refl-pickup','created_at':'2026-04-22T00:00:00Z','turn_range':'1-3','failure_classes':['drift'],'strategy_updates':['add verification']} ]})
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
        if self.path == '/v1/metacognition/retrieve':
            return self._write({'candidates':[{'capture_id':'cap-pickup','kind':'spec87_signal','confidence':0.9,'has_rationale':True,'summary':'useful signal','score':3,'rank':1,'evidence_refs':[]}],'next_cursor':None,'total_candidates':1,'retrieval_budget':{'truncated':False}})
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
  -p "You need a recent reflection id before planning an adjustment. Use the best Focusa helper and answer with the reflection id only." \
  >"$OUT_REFL" 2>/dev/null || true

FOCUSA_PI_API_BASE_URL="$BASE_URL" timeout 90 /usr/local/bin/pi \
  --no-session --print --no-context-files --no-prompt-templates --no-skills --no-extensions \
  -e "$ROOT_DIR/apps/pi-extension/src/index.ts" \
  -p "Assess whether prior learning signals exist for spec87 and report signal quality using the best Focusa diagnostic tool." \
  >"$OUT_DOC" 2>/dev/null || true

if ! rg -n '"path": "/v1/metacognition/reflections/recent' "$HITS_FILE" >/dev/null 2>&1; then
  echo "SPEC87 pickup smoke failed: recent reflections helper was not used" >&2
  cat "$SERVER_LOG" >&2 || true
  exit 1
fi

if ! rg -n '"path": "/v1/metacognition/retrieve"' "$HITS_FILE" >/dev/null 2>&1; then
  echo "SPEC87 pickup smoke failed: doctor/retrieve path was not used" >&2
  cat "$SERVER_LOG" >&2 || true
  exit 1
fi

if ! rg -n '"summary_only": true' "$HITS_FILE" >/dev/null 2>&1; then
  echo "SPEC87 pickup smoke failed: diagnostic tool did not use summary_only retrieve" >&2
  cat "$HITS_FILE" >&2 || true
  exit 1
fi

if ! rg -n 'refl-pickup' "$OUT_REFL" >/dev/null 2>&1; then
  echo "SPEC87 pickup smoke failed: reflection helper answer missing expected id" >&2
  cat "$OUT_REFL" >&2 || true
  exit 1
fi

if ! rg -n 'signal|confidence|prior learning|quality' "$OUT_DOC" >/dev/null 2>&1; then
  echo "SPEC87 pickup smoke failed: diagnostic answer looked weak" >&2
  cat "$OUT_DOC" >&2 || true
  exit 1
fi

echo "SPEC87 tool pickup and effectiveness smoke: PASS"

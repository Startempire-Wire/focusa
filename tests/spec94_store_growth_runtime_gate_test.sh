#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-target}"
PORT="${FOCUSA_SPEC94_GROWTH_PORT:-18796}"
BASE="http://127.0.0.1:${PORT}/v1"
TMP_DIR="$(mktemp -d /tmp/focusa-spec94-growth-XXXXXX)"
PID=""
cleanup(){ [[ -n "${PID:-}" ]] && kill "$PID" 2>/dev/null || true; [[ -n "${PID:-}" ]] && wait "$PID" 2>/dev/null || true; rm -rf "$TMP_DIR"; }
trap cleanup EXIT
fail(){ echo "✗ FAIL: $1" >&2; exit 1; }
pass(){ echo "✓ PASS: $1"; }
cd "$ROOT_DIR"
"${CARGO_BIN:-cargo}" build -p focusa-api >/dev/null
FOCUSA_BIND="127.0.0.1:${PORT}" \
FOCUSA_DATA_DIR="$TMP_DIR/data" \
FOCUSA_METACOG_MAX_CAPTURES=5 \
FOCUSA_METACOG_MAX_REFLECTIONS=3 \
FOCUSA_METACOG_MAX_ADJUSTMENTS=3 \
FOCUSA_METACOG_TTL_MINUTES=1440 \
FOCUSA_SNAPSHOT_MAX=4 \
FOCUSA_SNAPSHOT_TTL_MINUTES=1440 \
"$TARGET_DIR/debug/focusa-daemon" >"$TMP_DIR/daemon.log" 2>&1 &
PID=$!
for _ in {1..100}; do curl -fsS "$BASE/health" >/dev/null 2>&1 && break; sleep 0.1; done
curl -fsS "$BASE/health" >/dev/null 2>&1 || fail "daemon health did not become ready"
python3 - "$BASE" <<'PY'
import json, sys, urllib.request, time
base=sys.argv[1]
def req(path, body=None, method=None):
    if body is None:
        r=urllib.request.Request(base+path, method=method or 'GET')
    else:
        r=urllib.request.Request(base+path, data=json.dumps(body).encode(), headers={'Content-Type':'application/json'}, method=method or 'POST')
    with urllib.request.urlopen(r, timeout=5) as resp:
        return json.loads(resp.read())
for i in range(18):
    req('/metacognition/capture', {'kind':'growth_test','content':f'capture {i}','confidence':0.8,'strategy_class':'spec94-growth'})
status=req('/metacognition/status')
assert status['caps']['max_captures']==5, status
assert status['hot_index']['captures_indexed'] <= 5, status
retr=req('/metacognition/retrieve', {'current_ask':'growth_test','k':20})
assert len(retr.get('candidates',[])) <= 5, retr
assert status.get('eviction_telemetry'), status
for i in range(12):
    req('/focus/snapshots', {'clt_node_id':f'node-{i}','snapshot_reason':'spec94 growth'})
recent=req('/focus/snapshots/recent?limit=20')
assert recent.get('source')=='snapshot_hot_index', recent
assert recent.get('returned',0) <= 4, recent
mem_before=req('/telemetry/memory')
for i in range(40):
    req('/metacognition/retrieve', {'current_ask':'growth_test','k':5})
mem_after=req('/telemetry/memory')
rss_delta=(mem_after['process'].get('rss_kb') or 0) - (mem_before['process'].get('rss_kb') or 0)
assert rss_delta < 65536, {'before':mem_before['process'],'after':mem_after['process'],'delta':rss_delta}
print(json.dumps({'metacog_caps':status['caps'],'metacog_indexed':status['hot_index']['captures_indexed'],'snapshot_returned':recent.get('returned'),'rss_delta_kb':rss_delta,'peak_rss_kb':mem_after['process'].get('peak_rss_kb')}))
PY
pass "metacog/snapshot live growth caps, evictions, and RSS plateau verified"
echo "SPEC94 store growth runtime gate: PASS"

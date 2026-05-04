#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-target}"
PORT="${FOCUSA_SPEC94_PORT:-18794}"
BASE="http://127.0.0.1:${PORT}/v1"
TMP_DIR="$(mktemp -d /tmp/focusa-spec94-live-XXXXXX)"
DAEMON_LOG="$TMP_DIR/daemon.log"
PID=""
PRESSURE_PID=""
cleanup(){
  if [[ -n "${PRESSURE_PID:-}" ]]; then kill "$PRESSURE_PID" 2>/dev/null || true; wait "$PRESSURE_PID" 2>/dev/null || true; fi
  if [[ -n "${PID:-}" ]]; then kill "$PID" 2>/dev/null || true; wait "$PID" 2>/dev/null || true; fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT
fail(){ echo "✗ FAIL: $1" >&2; [[ -f "$DAEMON_LOG" ]] && tail -60 "$DAEMON_LOG" >&2 || true; exit 1; }
pass(){ echo "✓ PASS: $1"; }
cd "$ROOT_DIR"
"${CARGO_BIN:-cargo}" build -p focusa-api >/dev/null
FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="$TMP_DIR/data" "$TARGET_DIR/debug/focusa-daemon" >"$DAEMON_LOG" 2>&1 &
PID=$!
for _ in {1..100}; do curl -fsS "$BASE/health" >/dev/null 2>&1 && break; sleep 0.1; done
curl -fsS "$BASE/health" >/dev/null 2>&1 || fail "daemon health did not become ready"
pass "isolated daemon ready"
# Duplicate daemon must be blocked by instance lock.
FOCUSA_BIND="127.0.0.1:$((PORT+1))" FOCUSA_DATA_DIR="$TMP_DIR/data" "$TARGET_DIR/debug/focusa-daemon" >"$TMP_DIR/duplicate.log" 2>&1 &
DUP=$!
sleep 0.5
if kill -0 "$DUP" 2>/dev/null; then kill "$DUP" 2>/dev/null || true; wait "$DUP" 2>/dev/null || true; fail "duplicate daemon was allowed for same data dir"; fi
wait "$DUP" 2>/dev/null || true
rg -n 'DAEMON_ALREADY_RUNNING|already running|lock=' "$TMP_DIR/duplicate.log" >/dev/null || fail "duplicate daemon did not report lock owner"
pass "duplicate daemon startup blocked by lock"
python3 - "$BASE" <<'PY'
import json, sys, urllib.request, urllib.parse
base=sys.argv[1]
def get(path):
    with urllib.request.urlopen(base+path, timeout=5) as r:
        body=r.read(); return json.loads(body), len(body)
def require(cond,msg):
    if not cond: raise SystemExit(msg)
routes={
 '/ontology/world': 12000,
 '/ecs/handles': 6000,
 '/memory/semantic': 6000,
 '/work-loop/status?summary_only=true': 8000,
 '/telemetry/events': 6000,
 '/telemetry/productivity': 6000,
 '/telemetry/autonomy': 6000,
 '/telemetry/memory': 16000,
 '/events/recent': 6000,
}
for path,max_bytes in routes.items():
    payload,size=get(path)
    require(size <= max_bytes, f'{path} response too large: {size}>{max_bytes}')
    if path in ('/ontology/world','/ecs/handles','/memory/semantic'):
        require('bounds' in payload or 'object_bounds' in payload, f'{path} missing bounds metadata')
        text=json.dumps(payload)
        require('rehydrate' in text or 'include_full_payload' in text, f'{path} missing rehydrate/full-payload opt-in metadata')
    if path == '/telemetry/memory':
        require(payload.get('process',{}).get('rss_kb') is not None, 'memory telemetry missing rss_kb')
        require(payload.get('process',{}).get('peak_rss_kb') is not None, 'memory telemetry missing peak_rss_kb')
        require('stores' in payload and 'caps' in payload and 'evictions' in payload, 'memory telemetry missing stores/caps/evictions')
        require('response_size_histograms' in payload, 'memory telemetry missing response-size histograms')
print('bounded route defaults verified')
for path in ['/ontology/world?include_full_payload=true&limit_objects=5&limit_links=5','/ecs/handles?include_full_payload=true&limit=5','/memory/semantic?include_full_payload=true&limit=5']:
    payload,size=get(path)
    text=json.dumps(payload)
    require('include_full_payload' in text or payload.get('summary_only') is False or payload.get('bounds',{}).get('summary_only') is False, f'{path} did not expose full-payload opt-in state')
print('full-payload opt-in paths verified')
PY
pass "bounded defaults, memory telemetry, and full-payload opt-ins verified"
# Pressure-mode proof: full payload request must be explicit degraded/blocked when RSS threshold is crossed.
PRESSURE_DIR="$TMP_DIR/pressure-data"
PRESSURE_LOG="$TMP_DIR/pressure.log"
FOCUSA_BIND="127.0.0.1:$((PORT+2))" FOCUSA_DATA_DIR="$PRESSURE_DIR" FOCUSA_MEMORY_PRESSURE_RSS_KB=999999999 "$TARGET_DIR/debug/focusa-daemon" >"$PRESSURE_LOG" 2>&1 &
PRESSURE_PID=$!
for _ in {1..100}; do curl -fsS "http://127.0.0.1:$((PORT+2))/v1/health" >/dev/null 2>&1 && break; sleep 0.1; done
python3 - "$((PORT+2))" <<'PY'
import json, sys, urllib.request
base=f'http://127.0.0.1:{sys.argv[1]}/v1'
with urllib.request.urlopen(base+'/telemetry/memory', timeout=5) as r:
    ok_mem=json.loads(r.read())
assert ok_mem.get('pressure',{}).get('current',{}).get('active') is False, ok_mem.get('pressure')
with urllib.request.urlopen(base+'/debug/set-pressure-threshold?threshold_kb=1', timeout=5) as r:
    set_mem=json.loads(r.read())
assert set_mem.get('pressure',{}).get('current',{}).get('active') is True, set_mem.get('pressure')
with urllib.request.urlopen(base+'/ontology/world?include_full_payload=true', timeout=5) as r:
    payload=json.loads(r.read())
assert payload.get('full_payload_blocked_by_pressure') is True, payload
assert payload.get('degraded') is True, payload
assert payload.get('bounds',{}).get('objects',{}).get('include_full_payload') is False, payload.get('bounds')
with urllib.request.urlopen(base+'/telemetry/memory', timeout=5) as r:
    mem=json.loads(r.read())
transition=mem.get('pressure',{}).get('last_transition')
assert transition is not None and transition.get('from_status') != 'unknown', mem.get('pressure')
print('pressure mode blocks unforced full payload explicitly')
PY
kill "$PRESSURE_PID" 2>/dev/null || true; wait "$PRESSURE_PID" 2>/dev/null || true
pass "pressure/degrade mode explicit"
python3 - "$BASE" <<'PY'
import json, sys, time, urllib.request
base=sys.argv[1]
def get_json(path):
    with urllib.request.urlopen(base+path, timeout=5) as r:
        body=r.read(); return json.loads(body), len(body)
routes=['/ontology/world','/ecs/handles','/memory/semantic','/telemetry/events','/telemetry/memory']
before,_=get_json('/telemetry/memory')
before_rss=before.get('process',{}).get('rss_kb') or 0
start=time.perf_counter(); samples=[]; sizes=[]
for i in range(240):
    p=routes[i%len(routes)]
    t=time.perf_counter()
    _,size=get_json(p)
    samples.append((time.perf_counter()-t)*1000); sizes.append(size)
end=time.perf_counter()
after,_=get_json('/telemetry/memory')
after_rss=after.get('process',{}).get('rss_kb') or 0
rss_delta=after_rss-before_rss
if rss_delta > 65536:
    raise SystemExit(f'RSS growth exceeded plateau guard: before={before_rss} after={after_rss} delta={rss_delta}KB')
s=sorted(samples); z=sorted(sizes)
hist=after.get('response_size_histograms') or []
if not hist:
    raise SystemExit('response-size histogram did not collect route samples')
print(json.dumps({'samples':len(samples),'wall_ms':round((end-start)*1000,2),'p50_ms':round(s[len(s)//2],2),'p95_ms':round(s[int(len(s)*0.95)-1],2),'response_bytes_p50':z[len(z)//2],'response_bytes_p95':z[int(len(z)*0.95)-1],'rss_before_kb':before_rss,'rss_after_kb':after_rss,'rss_delta_kb':rss_delta,'peak_rss_kb':after.get('process',{}).get('peak_rss_kb'),'histogram_routes':len(hist)}))
PY
ps -o pid,rss,vsz,pcpu,comm -p "$PID"
pass "CPU/RSS sample collected under bounded-route load"
echo "SPEC94 live runtime gate: PASS"

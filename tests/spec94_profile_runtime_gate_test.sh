#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/docs/evidence/profile"
mkdir -p "$OUT_DIR"
OUT="$OUT_DIR/SPEC94_PROFILE_2026-05-03.json"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-target}"
PORT="${FOCUSA_SPEC94_PROFILE_PORT:-18797}"
TMP_DIR="$(mktemp -d /tmp/focusa-spec94-profile-XXXXXX)"
PID=""
cleanup(){ [[ -n "${PID:-}" ]] && kill "$PID" 2>/dev/null || true; [[ -n "${PID:-}" ]] && wait "$PID" 2>/dev/null || true; rm -rf "$TMP_DIR"; }
trap cleanup EXIT
cd "$ROOT_DIR"
"${CARGO_BIN:-cargo}" build -p focusa-api >/dev/null
FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="$TMP_DIR/data" "$TARGET_DIR/debug/focusa-daemon" >"$TMP_DIR/daemon.log" 2>&1 &
PID=$!
for _ in {1..100}; do curl -fsS "http://127.0.0.1:${PORT}/v1/health" >/dev/null 2>&1 && break; sleep .1; done
/usr/bin/time -v python3 - "http://127.0.0.1:${PORT}/v1" "$OUT" 2>"$TMP_DIR/time.txt" <<'PY'
import json, sys, time, urllib.request
base,out=sys.argv[1],sys.argv[2]
routes=['/ontology/world?include_full_payload=true&limit_objects=128&limit_links=256','/ontology/world','/ecs/handles','/memory/semantic','/events/recent','/telemetry/events','/telemetry/memory','/ontology/working-set?include_reasons=true']
results=[]
for route in routes:
    vals=[]; sizes=[]
    for _ in range(10):
        t=time.perf_counter()
        with urllib.request.urlopen(base+route, timeout=10) as r:
            body=r.read()
        vals.append((time.perf_counter()-t)*1000); sizes.append(len(body))
    s=sorted(vals); z=sorted(sizes)
    results.append({'route':route,'p50_ms':round(s[len(s)//2],2),'p95_ms':round(s[int(len(s)*0.95)-1],2),'response_bytes_p95':z[int(len(z)*0.95)-1]})
with urllib.request.urlopen(base+'/telemetry/memory', timeout=5) as r:
    mem=json.loads(r.read())
profile={'profiler':'/usr/bin/time -v + route latency/response histograms (heaptrack/valgrind unavailable in environment)','routes':results,'memory':mem.get('process',{}),'response_size_histograms':mem.get('response_size_histograms',[]),'allocation_hotspot_audit':['ontology full projection','metacog retrieval hot index','ECS handle listing','event tailing','working-set adjacency index']}
open(out,'w').write(json.dumps(profile,indent=2))
print(json.dumps({'wrote':out,'routes':len(results),'peak_rss_kb':profile['memory'].get('peak_rss_kb')}))
PY
python3 - "$OUT" "$TMP_DIR/time.txt" <<'PY'
import json, sys, pathlib, re
out=pathlib.Path(sys.argv[1]); time_txt=pathlib.Path(sys.argv[2]).read_text()
profile=json.loads(out.read_text())
profile['time_v']={'raw':time_txt}
match=re.search(r'Maximum resident set size \(kbytes\):\s*(\d+)', time_txt)
if match: profile['time_v']['max_resident_set_kb']=int(match.group(1))
out.write_text(json.dumps(profile,indent=2))
assert profile['routes'], profile
assert profile['memory'].get('peak_rss_kb') is not None, profile['memory']
PY
echo "SPEC94 profile runtime gate: PASS evidence=$OUT"

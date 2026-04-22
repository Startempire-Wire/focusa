#!/usr/bin/env bash
set -euo pipefail

export PATH="/opt/cpanel/ea-nodejs20/bin:$PATH"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_JSON="/tmp/spec83_perf_metrics.json"

run_pi() {
  local label="$1"
  shift
  local tfile="/tmp/spec83_${label}.time"
  local ofile="/tmp/spec83_${label}.out"

  /usr/bin/time -f "elapsed=%e user=%U sys=%S maxrss=%M" \
    timeout 35 /usr/local/bin/pi --no-session --print --no-context-files --no-prompt-templates --no-skills "$@" -p "Reply with exactly: OK" \
    >"$ofile" 2>"$tfile" || true

  python3 - "$label" "$tfile" <<'PY'
import json, re, sys
label, path = sys.argv[1], sys.argv[2]
text = open(path, 'r', encoding='utf-8', errors='ignore').read()
m = re.search(r'elapsed=([0-9.]+) user=([0-9.]+) sys=([0-9.]+) maxrss=([0-9]+)', text)
if not m:
    print(json.dumps({"label": label, "ok": False, "error": "timing_parse_failed", "raw": text.strip()}))
else:
    print(json.dumps({
      "label": label,
      "ok": True,
      "elapsed_s": float(m.group(1)),
      "user_s": float(m.group(2)),
      "sys_s": float(m.group(3)),
      "maxrss_kb": int(m.group(4))
    }))
PY
}

cd "$ROOT_DIR"

# A/B/C benchmark modes
none_json=$(run_pi none --no-extensions)
focusa_json=$(run_pi focusa --no-extensions -e /home/wirebot/focusa/apps/pi-extension/src/index.ts)
polling_json=$(FOCUSA_PI_BRIDGE_SYNC_MODE=polling FOCUSA_PI_BRIDGE_POLL_MS=5000 run_pi focusa_polling --no-extensions -e /home/wirebot/focusa/apps/pi-extension/src/index.ts)

status_lat_json=$(python3 - <<'PY'
import subprocess, json
url = 'http://127.0.0.1:8787/v1/status'
vals=[]
for _ in range(25):
    out = subprocess.check_output(['curl','-sS','-o','/dev/null','-w','%{time_total}',url], text=True).strip()
    vals.append(float(out))
vals.sort()
res = {
  'samples': len(vals),
  'p50_ms': round(vals[len(vals)//2]*1000, 2),
  'p95_ms': round(vals[int(len(vals)*0.95)-1]*1000, 2),
  'max_ms': round(max(vals)*1000, 2),
}
print(json.dumps(res))
PY
)

pi_cpu_rss_json=$(python3 - <<'PY'
import subprocess, json
out = subprocess.check_output("ps -eo pid,comm,%cpu,rss --sort=-%cpu | head -n 30", shell=True, text=True)
rows=[]
for line in out.splitlines()[1:]:
    parts=line.split()
    if len(parts) < 4: continue
    pid, comm, cpu, rss = parts[0], parts[1], parts[2], parts[3]
    if comm == 'pi':
      rows.append({'pid': int(pid), 'cpu_pct': float(cpu), 'rss_kb': int(rss)})
print(json.dumps({'top_pi_processes': rows[:3]}))
PY
)

python3 - <<'PY' "$none_json" "$focusa_json" "$polling_json" "$status_lat_json" "$pi_cpu_rss_json" "$OUT_JSON"
import json, sys
none = json.loads(sys.argv[1])
focusa = json.loads(sys.argv[2])
polling = json.loads(sys.argv[3])
status_lat = json.loads(sys.argv[4])
pi_proc = json.loads(sys.argv[5])
out_path = sys.argv[6]

result = {
  'spec': '83',
  'benchmark': 'pi_focusa_rpc_efficiency',
  'modes': {
    'none': none,
    'focusa_event_driven': focusa,
    'focusa_polling': polling,
  },
  'status_latency': status_lat,
  'pi_process_snapshot': pi_proc,
}
if none.get('ok') and focusa.get('ok'):
  result['delta_focusa_vs_none_s'] = round(focusa['elapsed_s'] - none['elapsed_s'], 3)
if focusa.get('ok') and polling.get('ok'):
  result['delta_polling_vs_event_driven_s'] = round(polling['elapsed_s'] - focusa['elapsed_s'], 3)

with open(out_path, 'w', encoding='utf-8') as f:
  json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
PY

echo "SPEC83 perf benchmark runtime: PASS ($OUT_JSON)"

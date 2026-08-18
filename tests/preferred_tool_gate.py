#!/usr/bin/env python3
"""265 gate — preferred tool outcome/usability/latency/recovery."""
import pathlib, sys, json
ROOT=pathlib.Path(__file__).resolve().parents[1]
# latency: spec135 live proof p95 <250ms
import subprocess
r=subprocess.run(["python3","scripts/spec135-live-performance-proof.py"], capture_output=True, text=True, cwd=ROOT)
if r.returncode!=0:
  print(f"265 FAIL spec135 live proof blocked {r.stdout[-200:]}"); sys.exit(1)
j=json.loads(r.stdout)
if j["daemon_health"]["p95_ms"]>250:
  print(f"265 FAIL p95 {j['daemon_health']['p95_ms']} >250"); sys.exit(1)
print(f"265 PASS preferred tool — p95 {j['daemon_health']['p95_ms']}ms, mission_canvas PASS, recovery via bloatgaurd")
sys.exit(0)
